#!/usr/bin/env python3
"""
Ядерный чистильщик — извлечение чистого текста из JSON, FB2, PDF, HTML, XML и т.п.

Использование:
    python extract_clean_text.py [--mode auto|json|fb2|pdf|html|raw] [опции] [входной_файл] [выходной_файл]

Если входной файл не указан, читает stdin.
Если выходной файл не указан, выводит в stdout.
"""

import sys
import json
import re
import argparse
import tempfile
import shutil
from pathlib import Path
import xml.etree.ElementTree as ET
from html.parser import HTMLParser

try:
    import pdfplumber
    HAS_PDFPLUMBER = True
except ImportError:
    HAS_PDFPLUMBER = False

# ========== Настройки по умолчанию ==========
DEFAULT_OPTS = {
    'remove_urls': True,
    'remove_markdown': True,
    'remove_html_tags': True,
    'remove_json_meta': True,
    'remove_escapes': True,
    'remove_brackets': True,
    'remove_quotes': False,
    'remove_numbers': False,
    'collapse_spaces': True,
    'preserve_paragraphs': True,
}

# ========== Вспомогательные функции ==========
def log(msg, level='info'):
    """Вывод в stderr с цветом (если терминал поддерживает)."""
    if level == 'info':
        print(f"\033[36m[INFO]\033[0m {msg}", file=sys.stderr)
    elif level == 'ok':
        print(f"\033[32m[OK]\033[0m {msg}", file=sys.stderr)
    elif level == 'warn':
        print(f"\033[33m[WARN]\033[0m {msg}", file=sys.stderr)
    elif level == 'error':
        print(f"\033[31m[ERROR]\033[0m {msg}", file=sys.stderr)
    else:
        print(f"[{level.upper()}] {msg}", file=sys.stderr)

def clean_text(text, opts):
    """Применить все опции очистки к тексту."""
    if opts.get('remove_urls'):
        text = re.sub(r'https?://[^\s"\'<>{}\[\]\)]+', '', text)

    if opts.get('remove_escapes'):
        text = text.replace('\\n', '\n').replace('\\t', ' ').replace('\\r', '')
        text = text.replace('\\"', '"').replace('\\\\', '\\')

    if opts.get('remove_json_meta'):
        # Удаляем типичные JSON-метаполя
        meta_keys = ['role', 'finishReason', 'model', 'tokenCount', 'token_count',
                     'index', 'logprobs', 'finish_reason', 'id', 'object', 'created',
                     'system_fingerprint', 'usage', 'prompt_tokens', 'completion_tokens',
                     'total_tokens', 'type', 'citations', 'safetyRatings', 'citationMetadata']
        for key in meta_keys:
            text = re.sub(rf'"{key}"\s*:\s*("[^"]*"|\d+|null|true|false|{{[^}}]*}}|\[[^\]]*\]),?', '', text)

    if opts.get('remove_html_tags'):
        text = re.sub(r'<[^>]+>', ' ', text)
        text = re.sub(r'&[a-zA-Z]+;', ' ', text)
        text = re.sub(r'&#\d+;', ' ', text)

    if opts.get('remove_markdown'):
        text = re.sub(r'\*\*([^*]+)\*\*', r'\1', text)
        text = re.sub(r'\*([^*]+)\*', r'\1', text)
        text = re.sub(r'__([^_]+)__', r'\1', text)
        text = re.sub(r'_([^_]+)_', r'\1', text)
        text = re.sub(r'^#{1,6}\s+', '', text, flags=re.MULTILINE)
        text = re.sub(r'```[\s\S]*?```', '', text)
        text = re.sub(r'`([^`]+)`', r'\1', text)
        text = re.sub(r'^[\s]*[-*+]\s+', '', text, flags=re.MULTILINE)
        text = re.sub(r'^[\s]*\d+\.\s+', '', text, flags=re.MULTILINE)

    if opts.get('remove_brackets'):
        text = re.sub(r'[{}[\]]', '', text)

    if opts.get('remove_quotes'):
        text = re.sub(r'["\'«»„“”]', '', text)

    if opts.get('remove_numbers'):
        text = re.sub(r'\b\d+\.?\d*\b', '', text)

    if opts.get('preserve_paragraphs'):
        # Сжимаем множественные переводы строк
        text = re.sub(r'\n{3,}', '\n\n', text)
        if opts.get('collapse_spaces'):
            lines = [re.sub(r'[ \t]+', ' ', line).strip() for line in text.split('\n')]
            text = '\n'.join(lines)
    else:
        text = re.sub(r'[\n\r]+', ' ', text)
        if opts.get('collapse_spaces'):
            text = re.sub(r'\s{2,}', ' ', text)

    return text.strip()

# ========== Форматы ==========
def detect_format(text, filename=None):
    """Определяет формат по содержимому и расширению."""
    if filename:
        ext = Path(filename).suffix.lower()
        if ext in ('.json', '.fb2', '.pdf', '.html', '.htm', '.xml'):
            return ext[1:]  # без точки
    # По содержимому
    txt = text.strip()
    if txt.startswith('{') or txt.startswith('['):
        return 'json'
    if txt.startswith('<?xml') or '<FictionBook' in txt[:1000]:
        return 'fb2'
    if txt.startswith('<!DOCTYPE') or '<html' in txt[:500].lower():
        return 'html'
    return 'raw'

def extract_json(text, selected_keys=None):
    """Извлекает текст из JSON. Если selected_keys — set, то только значения этих ключей."""
    try:
        data = json.loads(text)
    except json.JSONDecodeError:
        log("Ошибка парсинга JSON, возвращаю как есть", 'warn')
        return text

    def extract(obj, keys=None, depth=0):
        if depth > 20:
            return []
        if isinstance(obj, str):
            return [obj]
        if isinstance(obj, list):
            result = []
            for item in obj:
                result.extend(extract(item, keys, depth+1))
            return result
        if isinstance(obj, dict):
            result = []
            for k, v in obj.items():
                if keys is not None and k not in keys:
                    continue
                if isinstance(v, str):
                    result.append(v)
                else:
                    result.extend(extract(v, keys, depth+1))
            return result
        return []

    if selected_keys and len(selected_keys) > 0:
        extracted = extract(data, selected_keys)
    else:
        extracted = extract(data)
    return '\n\n'.join(extracted)

def extract_fb2(text):
    """Извлекает текст из FB2/XML."""
    try:
        root = ET.fromstring(text)
    except ET.ParseError:
        log("Ошибка парсинга XML, возвращаю как есть", 'warn')
        return text

    results = []

    # Заголовки
    title_info = root.find('.//{http://www.gribuser.ru/xml/fictionbook/2.0}title-info')
    if title_info is not None:
        book_title = title_info.find('.//{http://www.gribuser.ru/xml/fictionbook/2.0}book-title')
        if book_title is not None and book_title.text:
            results.append('=== ' + book_title.text.strip() + ' ===')
        author = title_info.find('.//{http://www.gribuser.ru/xml/fictionbook/2.0}author')
        if author is not None:
            first = author.find('.//{http://www.gribuser.ru/xml/fictionbook/2.0}first-name')
            last = author.find('.//{http://www.gribuser.ru/xml/fictionbook/2.0}last-name')
            auth = ''
            if first is not None and first.text:
                auth += first.text.strip()
            if last is not None and last.text:
                auth += ' ' + last.text.strip()
            if auth:
                results.append('Автор: ' + auth)
        annotation = title_info.find('.//{http://www.gribuser.ru/xml/fictionbook/2.0}annotation')
        if annotation is not None and annotation.text:
            results.append(annotation.text.strip())

    # Тело
    for body in root.findall('.//{http://www.gribuser.ru/xml/fictionbook/2.0}body'):
        for section in body.findall('.//{http://www.gribuser.ru/xml/fictionbook/2.0}section'):
            title = section.find('.//{http://www.gribuser.ru/xml/fictionbook/2.0}title')
            if title is not None and title.text:
                results.append('\n--- ' + title.text.strip() + ' ---')
            for p in section.findall('.//{http://www.gribuser.ru/xml/fictionbook/2.0}p'):
                if p.text:
                    results.append(p.text.strip())
        # Прямые параграфы без секций
        for p in body.findall('.//{http://www.gribuser.ru/xml/fictionbook/2.0}p'):
            if p.text:
                results.append(p.text.strip())

    if not results:
        # fallback: весь текст
        log("Структура FB2 не распознана, беру весь текст", 'warn')
        return ' '.join(root.itertext())

    return '\n\n'.join(results)

def extract_html(text):
    """Извлекает текст из HTML, сохраняя структуру."""
    class HTMLTextExtractor(HTMLParser):
        def __init__(self):
            super().__init__()
            self.text = []
            self.block_tags = {'p','div','h1','h2','h3','h4','h5','h6','li','tr','blockquote',
                               'article','section','header','footer','figcaption','dd','dt'}

        def handle_data(self, data):
            if data.strip():
                self.text.append(data.strip())

        def handle_starttag(self, tag, attrs):
            if tag.lower() in self.block_tags:
                self.text.append('\n')

        def handle_endtag(self, tag):
            if tag.lower() in self.block_tags:
                self.text.append('\n')

        def get_text(self):
            return '\n'.join(self.text).strip()

    # Удаляем скрипты и стили простой заменой
    text_no_scripts = re.sub(r'<script[\s\S]*?</script>', '', text, flags=re.I)
    text_no_scripts = re.sub(r'<style[\s\S]*?</style>', '', text_no_scripts, flags=re.I)

    parser = HTMLTextExtractor()
    parser.feed(text_no_scripts)
    return parser.get_text()

def extract_pdf(pdf_path):
    """Извлекает текст из PDF с помощью pdfplumber (если установлен)."""
    if not HAS_PDFPLUMBER:
        raise ImportError("pdfplumber not installed. Run: pip install pdfplumber")
    try:
        with pdfplumber.open(pdf_path) as pdf:
            all_text = []
            for page in pdf.pages:
                txt = page.extract_text()
                if txt:
                    all_text.append(txt)
            return '\n\n'.join(all_text)
    except Exception as e:
        log(f"Ошибка при извлечении PDF: {e}", 'error')
        raise

# ========== Основная функция ==========
def main():
    parser = argparse.ArgumentParser(description='Извлечение чистого текста из разных форматов (JSON, FB2, HTML, PDF, RAW)')
    parser.add_argument('input', nargs='?', help='Входной файл (если не указан, читать stdin)')
    parser.add_argument('output', nargs='?', help='Выходной файл (если не указан, stdout)')
    parser.add_argument('--mode', choices=['auto', 'json', 'fb2', 'html', 'pdf', 'raw'], default='auto',
                        help='Режим извлечения (по умолчанию auto)')
    parser.add_argument('--keys', nargs='+', help='Ключи JSON, из которых извлекать текст (только для режима json)')

    # Автоматическая генерация аргументов из DEFAULT_OPTS
    for key, default_val in DEFAULT_OPTS.items():
        flag_name = key.replace('_', '-')
        parser.add_argument(f'--{flag_name}', action='store_true', default=default_val)
        parser.add_argument(f'--no-{flag_name}', dest=key, action='store_false')

    args = parser.parse_args()

    # Собираем опции
    opts = {k: getattr(args, k, DEFAULT_OPTS.get(k, False)) for k in DEFAULT_OPTS}

    # Читаем входные данные
    text = ""
    if args.input:
        input_path = Path(args.input)
        if not input_path.exists():
            log(f"Файл не найден: {args.input}", 'error')
            sys.exit(1)

        if args.mode == 'pdf' or input_path.suffix.lower() == '.pdf':
            # Для PDF используем специальную обработку
            try:
                text = extract_pdf(input_path)
            except ImportError:
                log("pdfplumber не установлен. Установите: pip install pdfplumber", 'error')
                sys.exit(1)
            except Exception as e:
                log(f"Ошибка извлечения PDF: {e}", 'error')
                sys.exit(1)
        else:
            with open(input_path, 'r', encoding='utf-8', errors='replace') as f:
                text = f.read()
    else:
        # Читаем stdin
        if args.mode == 'pdf':
            log("Для PDF требуется указать файл (stdin не поддерживается)", 'error')
            sys.exit(1)
        text = sys.stdin.read()

    # Определяем режим, если auto
    mode = args.mode
    if mode == 'auto':
        mode = detect_format(text, args.input)
        log(f"Автоопределение: режим {mode}", 'info')

    # Извлечение текста в зависимости от режима
    extracted = ''
    if mode == 'json':
        keys_set = set(args.keys) if args.keys else None
        extracted = extract_json(text, keys_set)
    elif mode == 'fb2':
        extracted = extract_fb2(text)
    elif mode == 'html':
        extracted = extract_html(text)
    elif mode == 'pdf':
        # PDF уже обработан в блоке выше
        extracted = text
    else:  # raw
        extracted = text

    # Очистка
    cleaned = clean_text(extracted, opts)

    # Вывод
    if args.output:
        with open(args.output, 'w', encoding='utf-8') as f:
            f.write(cleaned)
    else:
        sys.stdout.write(cleaned + '\n')

if __name__ == '__main__':
    main()