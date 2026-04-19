#!/usr/bin/env python3
"""
JACKAL HOST PIPELINE — NeroShizaDev-OS
======================================
Подготавливает данные на хосте и упаковывает их в .jkl-контейнер
для последующего анализа Jackal-analyzer'ом в ядре.

Пайплайн:
  1. Извлечь чистый текст из любого формата (JSON/FB2/HTML/PDF/DOCX/raw)
     через extract_clean_text.py
  2. Проанализировать байты локально (эвристика: энтропия, magic-bytes)
  3. Применить адаптивное кодирование (RLE/Delta/Store)
  4. Упаковать в .jkl-файл с заголовком
  5. Опционально: отправить через serial-порт или записать в образ диска

Использование:
    python pipeline.py input.[json|fb2|html|pdf|docx|txt|bin] output.jkl
    python pipeline.py --demo                # тестовый прогон на встроенных данных
    python pipeline.py --analyze input.bin   # только анализ, без упаковки

Опции:
    --mode auto|json|fb2|html|pdf|raw  Режим извлечения текста (для text-форматов)
    --keys k1 k2 ...                   Ключи JSON
    --serial /dev/ttyS0                Отправить .jkl в ядро через serial
    --serial-baud 115200               Скорость порта (по умолчанию 115200)
    --no-encode                        Не сжимать, только store
    -v / --verbose                     Подробный вывод
"""

import sys
import os
import struct
import math
import argparse
import subprocess
import tempfile
from pathlib import Path


# ============================================================
# .jkl формат (должен совпадать с encoder.rs)
# ============================================================
JKL_MAGIC     = b'JKL\x01'
JKL_HEADER_SZ = 22

ALG_STORE = 0
ALG_RLE   = 1
ALG_DELTA = 2

KIND_TEXT   = 0
KIND_EXEC   = 1
KIND_COMP   = 2
KIND_RAND   = 3
KIND_STRC   = 4
KIND_MEDIA  = 5
KIND_UNKNWN = 6

KIND_NAMES = {
    KIND_TEXT:   'TEXT',
    KIND_EXEC:   'EXECUTABLE',
    KIND_COMP:   'COMPRESSED',
    KIND_RAND:   'RANDOM',
    KIND_STRC:   'STRUCTURED',
    KIND_MEDIA:  'LOSSY MEDIA',
    KIND_UNKNWN: 'UNKNOWN',
}

ALG_NAMES = {ALG_STORE: 'STORE', ALG_RLE: 'RLE', ALG_DELTA: 'DELTA'}

MAGIC_TABLE = [
    (0, b'MZ',                      KIND_EXEC,  'DOS/PE'),
    (0, b'\x7fELF',                 KIND_EXEC,  'ELF'),
    (0, b'PK\x03\x04',             KIND_COMP,  'ZIP'),
    (0, b'\x1f\x8b',               KIND_COMP,  'GZIP'),
    (0, b'\xfd7zXZ',               KIND_COMP,  'XZ'),
    (0, b'Rar!',                    KIND_COMP,  'RAR'),
    (0, b'\x89PNG\r\n\x1a\n',      KIND_STRC,  'PNG'),
    (0, b'\xff\xd8\xff',           KIND_MEDIA, 'JPEG'),
    (0, b'GIF8',                    KIND_STRC,  'GIF'),
    (0, b'BM',                      KIND_STRC,  'BMP'),
    (0, b'RIFF',                    KIND_STRC,  'RIFF'),
    (0, b'%PDF',                    KIND_STRC,  'PDF'),
    (0, b'\xef\xbb\xbf',          KIND_TEXT,  'UTF-8 BOM'),
]


# ============================================================
# Локальный анализатор (Python-версия Jackal analyzer)
# ============================================================

def sniff_magic(data: bytes):
    for offset, sig, kind, name in MAGIC_TABLE:
        if len(data) >= offset + len(sig):
            if data[offset:offset + len(sig)] == sig:
                return kind, name
    return None, None


def shannon_entropy(data: bytes) -> float:
    if not data:
        return 0.0
    counts = [0] * 256
    for b in data:
        counts[b] += 1
    n = len(data)
    h = 0.0
    for c in counts:
        if c > 0:
            p = c / n
            h -= p * math.log2(p)
    return h


def printable_ratio(data: bytes) -> float:
    if not data:
        return 0.0
    printable = sum(1 for b in data if 0x20 <= b <= 0x7E or b in (9, 10, 13))
    return printable / len(data)


def classify(data: bytes, verbose=False) -> tuple[int, str, float]:
    """Возвращает (kind_byte, kind_name, confidence 0..1)."""
    kind_m, magic_name = sniff_magic(data)
    h = shannon_entropy(data)
    pr = printable_ratio(data)

    if verbose:
        print(f"  [ANALYZE] entropy={h:.3f}  printable={pr:.3f}  magic={magic_name}", file=sys.stderr)

    if kind_m is not None:
        return kind_m, KIND_NAMES[kind_m], 0.85

    if pr > 0.90 and h < 5.5:
        return KIND_TEXT, 'TEXT', 0.85

    if h > 7.5:
        return KIND_COMP, 'COMPRESSED/RANDOM', 0.70

    if h > 3.0 and pr < 0.3:
        return KIND_STRC, 'STRUCTURED', 0.60

    return KIND_UNKNWN, 'UNKNOWN', 0.30


# ============================================================
# Кодирование (Python-версия Jackal encoder)
# ============================================================

def encode_rle(data: bytes) -> bytes:
    out = bytearray()
    i = 0
    while i < len(data):
        b = data[i]
        run = 1
        while i + run < len(data) and data[i + run] == b and run < 255:
            run += 1
        out.append(run)
        out.append(b)
        i += run
    return bytes(out)


def encode_delta(data: bytes) -> bytes:
    if not data:
        return b''
    out = bytearray(len(data))
    out[0] = data[0]
    for i in range(1, len(data)):
        out[i] = (data[i] - data[i - 1]) & 0xFF
    return bytes(out)


def encode_adaptive(data: bytes, kind: int, no_encode=False) -> tuple[int, bytes]:
    if no_encode:
        return ALG_STORE, data

    if kind == KIND_TEXT:
        rle = encode_rle(data)
        if len(rle) < len(data):
            return ALG_RLE, rle
        return ALG_STORE, data

    if kind == KIND_STRC:
        delta = encode_delta(data)
        if len(delta) < len(data):
            return ALG_DELTA, delta
        return ALG_STORE, data

    return ALG_STORE, data


# ============================================================
# .jkl упаковщик
# ============================================================

def pack_jkl(data: bytes, kind: int, algorithm: int, encoded: bytes) -> bytes:
    header = (
        JKL_MAGIC
        + bytes([kind, algorithm])
        + struct.pack('<Q', len(data))
        + struct.pack('<Q', len(encoded))
    )
    assert len(header) == JKL_HEADER_SZ
    return header + encoded


def unpack_jkl(jkl: bytes) -> dict:
    if len(jkl) < JKL_HEADER_SZ:
        raise ValueError('JKL too short')
    magic = jkl[0:4]
    if magic != JKL_MAGIC:
        raise ValueError(f'Bad JKL magic: {magic!r}')
    kind      = jkl[4]
    alg       = jkl[5]
    orig_sz   = struct.unpack('<Q', jkl[6:14])[0]
    enc_sz    = struct.unpack('<Q', jkl[14:22])[0]
    payload   = jkl[22:]
    return {
        'kind': kind, 'kind_name': KIND_NAMES.get(kind, '?'),
        'algorithm': alg, 'alg_name': ALG_NAMES.get(alg, '?'),
        'original_size': orig_sz, 'encoded_size': enc_sz,
        'payload': payload,
    }


# ============================================================
# Извлечение текста через extract_clean_text.py
# ============================================================

SCRIPTS_DIR = Path(__file__).parent


def extract_text(input_path: Path, mode='auto', keys=None) -> bytes:
    """Вызывает extract_clean_text.py, возвращает чистый текст как bytes."""
    script = SCRIPTS_DIR / 'extract_clean_text.py'
    if not script.exists():
        raise FileNotFoundError(f'extract_clean_text.py not found at {script}')

    cmd = [sys.executable, str(script), '--mode', mode, str(input_path)]
    if keys:
        cmd += ['--keys'] + list(keys)

    result = subprocess.run(cmd, capture_output=True)
    if result.returncode != 0:
        print(result.stderr.decode(errors='replace'), file=sys.stderr)
        raise RuntimeError(f'extract_clean_text.py failed (rc={result.returncode})')
    return result.stdout


# ============================================================
# Serial отправка (опционально)
# ============================================================

def send_serial(jkl_data: bytes, port: str, baud: int, verbose=False):
    try:
        import serial  # pyserial
    except ImportError:
        print('[WARN] pyserial не установлен. Установите: pip install pyserial', file=sys.stderr)
        return
    with serial.Serial(port, baud, timeout=5) as s:
        # Простой протокол: сначала 4 байта размера LE, потом данные
        s.write(struct.pack('<I', len(jkl_data)))
        s.write(jkl_data)
        if verbose:
            print(f'[SERIAL] Отправлено {len(jkl_data)} байт → {port}@{baud}', file=sys.stderr)


# ============================================================
# CLI
# ============================================================

def main():
    parser = argparse.ArgumentParser(
        description='Jackal Host Pipeline — NeroShizaDev-OS data packer'
    )
    parser.add_argument('input', nargs='?', help='Входной файл')
    parser.add_argument('output', nargs='?', help='Выходной .jkl файл')
    parser.add_argument('--demo',      action='store_true', help='Запустить встроенный demo')
    parser.add_argument('--analyze',   action='store_true', help='Только анализ без упаковки')
    parser.add_argument('--mode',      default='auto', choices=['auto','json','fb2','html','pdf','raw'])
    parser.add_argument('--keys',      nargs='+')
    parser.add_argument('--serial',    help='Serial-порт для отправки в ядро (напр. COM3)')
    parser.add_argument('--serial-baud', type=int, default=115200)
    parser.add_argument('--no-encode', action='store_true', help='Не сжимать (store)')
    parser.add_argument('-v', '--verbose', action='store_true')
    args = parser.parse_args()

    if args.demo:
        data = (
            b"Hello NeroShizaDev-OS! This is a host-side Jackal pipeline demo.\n"
            b"The quick brown fox jumps over the lazy dog. 1234567890\n"
            b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR"
        )
        print("[DEMO] Встроенные тестовые данные:", len(data), "байт", file=sys.stderr)
    elif args.input:
        input_path = Path(args.input)
        if not input_path.exists():
            print(f'[ERROR] Файл не найден: {args.input}', file=sys.stderr)
            sys.exit(1)

        # Для текстовых форматов — извлекаем через extract_clean_text
        text_exts = {'.json', '.fb2', '.html', '.htm', '.pdf', '.docx'}
        if input_path.suffix.lower() in text_exts or args.mode != 'auto':
            print(f'[INFO] Извлечение текста из {input_path.name}...', file=sys.stderr)
            data = extract_text(input_path, args.mode, args.keys)
        else:
            data = input_path.read_bytes()

        print(f'[INFO] Прочитано {len(data)} байт', file=sys.stderr)
    else:
        parser.print_help()
        sys.exit(0)

    # Анализ
    kind, kind_name, conf = classify(data, verbose=args.verbose)
    h = shannon_entropy(data)
    pr = printable_ratio(data)

    print(f'[JACKAL] Классификация : {kind_name}  (уверенность {conf*100:.0f}%)', file=sys.stderr)
    print(f'[JACKAL] Энтропия      : {h:.3f} бит/байт', file=sys.stderr)
    print(f'[JACKAL] Печатных      : {pr*100:.1f}%', file=sys.stderr)

    if args.analyze:
        return

    # Кодирование
    algorithm, encoded = encode_adaptive(data, kind, no_encode=args.no_encode)
    ratio = len(encoded) / len(data) * 100 if data else 100
    print(f'[JACKAL] Алгоритм      : {ALG_NAMES[algorithm]}', file=sys.stderr)
    print(f'[JACKAL] Размер до     : {len(data)} байт', file=sys.stderr)
    print(f'[JACKAL] Размер после  : {len(encoded)} байт  ({ratio:.1f}%)', file=sys.stderr)

    # Упаковка в .jkl
    jkl = pack_jkl(data, kind, algorithm, encoded)
    print(f'[JACKAL] JKL размер    : {len(jkl)} байт  (с заголовком {JKL_HEADER_SZ} байт)', file=sys.stderr)

    # Запись
    if args.output:
        Path(args.output).write_bytes(jkl)
        print(f'[OK] Сохранено → {args.output}', file=sys.stderr)
    else:
        sys.stdout.buffer.write(jkl)

    # Serial отправка
    if args.serial:
        send_serial(jkl, args.serial, args.serial_baud, verbose=args.verbose)


if __name__ == '__main__':
    main()

