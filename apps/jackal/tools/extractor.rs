// ============================================================
// EXTRACTOR — Rust-версия extract_clean_text.py
// ============================================================
// Очищает сырые байты от HTML-тегов, Markdown-разметки,
// JSON-метаполей и нормализует пробелы.
//
// Python-оригинал: apps/jackal/scripts/extract_clean_text.py
//
// API:
//   extract(src, dst, mode, opts) -> usize   — кол-во записанных байт
//   detect_format(data) -> Format            — автоопределение формата
// ============================================================

#![allow(dead_code)]

/// Режим извлечения (аналог --mode в Python-скрипте)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Auto,
    Json,
    Html,
    Xml, // включает FB2
    Raw,
}

/// Опции очистки (аналог DEFAULT_OPTS)
pub struct ExtractOpts {
    pub remove_urls: bool,
    pub remove_html_tags: bool,
    pub remove_markdown: bool,
    pub remove_json_meta: bool,
    pub remove_brackets: bool,
    pub collapse_spaces: bool,
    pub preserve_newlines: bool,
}

impl ExtractOpts {
    /// Настройки по умолчанию (совпадают с Python DEFAULT_OPTS)
    pub const fn default() -> Self {
        Self {
            remove_urls: true,
            remove_html_tags: true,
            remove_markdown: true,
            remove_json_meta: true,
            remove_brackets: false,
            collapse_spaces: true,
            preserve_newlines: true,
        }
    }
}

// ============================================================
// Автоопределение формата
// ============================================================

/// Пытается угадать формат по первым байтам данных.
pub fn detect_format(data: &[u8]) -> Format {
    let head = &data[..data.len().min(64)];

    // JSON: начинается с { или [
    if head.first().copied() == Some(b'{') || head.first().copied() == Some(b'[') {
        return Format::Json;
    }
    // HTML
    if starts_with_str(head, "<!DOCTYPE") || contains_bytes(head, b"<html") {
        return Format::Html;
    }
    // XML / FB2
    if starts_with_str(head, "<?xml") || contains_bytes(head, b"<FictionBook") {
        return Format::Xml;
    }
    Format::Raw
}

// ============================================================
// Главная функция извлечения
// ============================================================

/// Извлекает «чистый» текст из `src` и записывает в `dst`.
/// Возвращает количество записанных байт.
///
/// Алгоритм: сначала format-specific разбор, затем general cleaner.
pub fn extract<'a>(src: &[u8], dst: &'a mut [u8], mode: Format, opts: &ExtractOpts) -> usize {
    let fmt = if mode == Format::Auto {
        detect_format(src)
    } else {
        mode
    };

    // Промежуточный буфер для format-specific прохода
    let mut stage: [u8; 8192] = [0u8; 8192];
    let stage_len = match fmt {
        Format::Json => strip_json_keys(src, &mut stage, opts),
        Format::Html => strip_html_tags(src, &mut stage),
        Format::Xml => strip_xml_tags(src, &mut stage),
        _ => {
            let n = src.len().min(stage.len());
            stage[..n].copy_from_slice(&src[..n]);
            n
        }
    };

    // Общая очистка поверх format-specific результата
    clean_text(&stage[..stage_len], dst, opts)
}

// ============================================================
// Format-specific: JSON — убираем ключи-метаданные
// ============================================================

/// Копирует строковые значения из JSON, пропуская ключи-мета.
/// Упрощённый вариант (не полный парсер): ищет `"key": "value"` паттерны.
fn strip_json_keys(src: &[u8], dst: &mut [u8], opts: &ExtractOpts) -> usize {
    // Ключи которые считаем мусором (аналог Python meta_keys)
    const META_KEYS: &[&[u8]] = &[
        b"role",
        b"model",
        b"id",
        b"object",
        b"created",
        b"index",
        b"finish_reason",
        b"logprobs",
        b"type",
        b"usage",
        b"prompt_tokens",
        b"completion_tokens",
        b"total_tokens",
    ];

    let mut dst_pos = 0usize;
    let mut i = 0usize;

    while i < src.len() && dst_pos < dst.len() {
        // Ищем начало JSON-строки
        if src[i] == b'"' {
            // Читаем ключ или значение
            let (s, end) = read_json_string(src, i + 1);
            i = end;
            // Проверяем: это ключ (за ним ':') или значение
            let after = skip_whitespace(src, i);
            if after < src.len() && src[after] == b':' {
                // Это ключ — проверяем нужно ли пропустить
                let is_meta = META_KEYS.iter().any(|k| s == *k) && opts.remove_json_meta;
                if is_meta {
                    // Пропускаем ':' и значение
                    i = skip_json_value(src, after + 1);
                }
                // Ключи не копируем в вывод в любом случае
            } else {
                // Это значение — копируем
                for &b in s {
                    if dst_pos >= dst.len() {
                        break;
                    }
                    // Экранирование: \n → реальный перенос
                    if b == b'\\' {
                        continue;
                    }
                    dst[dst_pos] = b;
                    dst_pos += 1;
                }
                if dst_pos < dst.len() {
                    dst[dst_pos] = b'\n';
                    dst_pos += 1;
                }
            }
        } else {
            i += 1;
        }
    }
    dst_pos
}

/// Читает JSON-строку начиная с позиции `start` (после открывающей `"`).
/// Возвращает (&[u8] содержимое, позиция после закрывающей `"`).
fn read_json_string(src: &[u8], start: usize) -> (&[u8], usize) {
    let mut i = start;
    while i < src.len() {
        if src[i] == b'\\' {
            i += 2;
            continue;
        }
        if src[i] == b'"' {
            return (&src[start..i], i + 1);
        }
        i += 1;
    }
    (&src[start..i], i)
}

fn skip_whitespace(src: &[u8], start: usize) -> usize {
    let mut i = start;
    while i < src.len() && (src[i] == b' ' || src[i] == b'\t' || src[i] == b'\n' || src[i] == b'\r')
    {
        i += 1;
    }
    i
}

/// Пропускает одно JSON-значение (строку, число, null, bool, объект, массив).
fn skip_json_value(src: &[u8], start: usize) -> usize {
    let i = skip_whitespace(src, start);
    if i >= src.len() {
        return i;
    }
    match src[i] {
        b'"' => {
            let (_, end) = read_json_string(src, i + 1);
            end
        }
        b'{' | b'[' => {
            // Ищем соответствующую закрывающую скобку (упрощённо)
            let close = if src[i] == b'{' { b'}' } else { b']' };
            let mut depth = 1usize;
            let mut j = i + 1;
            while j < src.len() && depth > 0 {
                if src[j] == src[i] {
                    depth += 1;
                } else if src[j] == close {
                    depth -= 1;
                } else if src[j] == b'"' {
                    let (_, end) = read_json_string(src, j + 1);
                    j = end;
                    continue;
                }
                j += 1;
            }
            // Пропускаем запятую если есть
            let j = skip_whitespace(src, j);
            if j < src.len() && src[j] == b',' {
                j + 1
            } else {
                j
            }
        }
        _ => {
            // Число / null / true / false — до запятой или пробела
            let mut j = i;
            while j < src.len()
                && src[j] != b','
                && src[j] != b'}'
                && src[j] != b']'
                && src[j] != b'\n'
            {
                j += 1;
            }
            if j < src.len() && src[j] == b',' {
                j + 1
            } else {
                j
            }
        }
    }
}

// ============================================================
// Format-specific: HTML — убираем теги
// ============================================================

fn strip_html_tags(src: &[u8], dst: &mut [u8]) -> usize {
    let mut dst_pos = 0usize;
    let mut i = 0usize;
    let mut in_tag = false;
    let mut in_script = false;

    while i < src.len() && dst_pos < dst.len() {
        if in_script {
            // Пропускаем до </script>
            if i + 8 < src.len() && &src[i..i + 9] == b"</script>" {
                in_script = false;
                i += 9;
            } else {
                i += 1;
            }
            continue;
        }

        if src[i] == b'<' {
            in_tag = true;
            // Проверяем <script
            if i + 6 < src.len() {
                let tag = &src[i..i + 7];
                if tag == b"<script" || tag == b"<SCRIPT" {
                    in_script = true;
                }
            }
            // Блочные теги → перенос строки
            let block = is_block_tag(src, i);
            if block && dst_pos < dst.len() {
                dst[dst_pos] = b'\n';
                dst_pos += 1;
            }
        } else if src[i] == b'>' {
            in_tag = false;
        } else if !in_tag {
            dst[dst_pos] = src[i];
            dst_pos += 1;
        }
        i += 1;
    }
    dst_pos
}

fn is_block_tag(src: &[u8], pos: usize) -> bool {
    const BLOCK: &[&[u8]] = &[
        b"<p", b"<div", b"<h1", b"<h2", b"<h3", b"<h4", b"<h5", b"<h6", b"<li", b"<tr", b"<br",
        b"<P", b"<DIV", b"<BR",
    ];
    for &t in BLOCK {
        if pos + t.len() <= src.len() && &src[pos..pos + t.len()] == t {
            return true;
        }
    }
    false
}

// ============================================================
// Format-specific: XML/FB2
// ============================================================

fn strip_xml_tags(src: &[u8], dst: &mut [u8]) -> usize {
    // Для XML стратегия та же что и для HTML — убираем теги
    strip_html_tags(src, dst)
}

// ============================================================
// Общая очистка текста (apply opts)
// ============================================================

fn clean_text(src: &[u8], dst: &mut [u8], opts: &ExtractOpts) -> usize {
    let mut dst_pos = 0usize;
    let mut i = 0usize;
    let mut prev_space = false;
    let mut prev_newline = false;

    while i < src.len() && dst_pos < dst.len() {
        let b = src[i];

        // Убираем URL: http... до пробела
        if opts.remove_urls
            && i + 4 < src.len()
            && (&src[i..i + 7] == b"http://" || &src[i..i + 8] == b"https://")
        {
            while i < src.len() && src[i] != b' ' && src[i] != b'\n' && src[i] != b'\t' {
                i += 1;
            }
            continue;
        }

        // Убираем Markdown **bold** и *italic* (упрощённо — пропускаем * и #)
        if opts.remove_markdown && (b == b'*' || b == b'#' || b == b'`') {
            i += 1;
            continue;
        }

        // Убираем квадратные скобки
        if opts.remove_brackets && (b == b'[' || b == b']' || b == b'{' || b == b'}') {
            i += 1;
            continue;
        }

        // Пробелы / переносы
        if b == b'\n' || b == b'\r' {
            if opts.preserve_newlines && !prev_newline {
                dst[dst_pos] = b'\n';
                dst_pos += 1;
                prev_newline = true;
                prev_space = false;
            }
            i += 1;
            continue;
        }

        if b == b' ' || b == b'\t' {
            if opts.collapse_spaces && !prev_space {
                dst[dst_pos] = b' ';
                dst_pos += 1;
                prev_space = true;
            }
            i += 1;
            continue;
        }

        // Обычный байт
        dst[dst_pos] = b;
        dst_pos += 1;
        prev_space = false;
        prev_newline = false;
        i += 1;
    }
    dst_pos
}

// ============================================================
// Вспомогательные функции
// ============================================================

fn starts_with_str(data: &[u8], pat: &str) -> bool {
    let pb = pat.as_bytes();
    data.len() >= pb.len() && &data[..pb.len()] == pb
}

fn contains_bytes(data: &[u8], pat: &[u8]) -> bool {
    if pat.len() > data.len() {
        return false;
    }
    for i in 0..=(data.len() - pat.len()) {
        if &data[i..i + pat.len()] == pat {
            return true;
        }
    }
    false
}
