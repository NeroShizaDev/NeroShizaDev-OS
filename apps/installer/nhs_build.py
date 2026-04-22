#!/usr/bin/env python3
"""
NHS Builder — NeroShizaDev-OS Package Builder
=============================================
Собирает .nhs пакет из скрипта + ресурсов.

Цикл разработки БЕЗ пересборки ядра:
  1. Написал скрипт hello.nss (NeroShizaScript)
  2. python nhs_build.py hello.nss -o hello.nhs
  3. В QEMU: Shell> install hello.nhs
  4. Готово. Ядро не перекомпилировалось.

Формат .nhs (64-byte header):
  [0..4]   MAGIC "NHS\x1A"
  [4..6]   version_major: u16 LE
  [6..8]   version_minor: u16 LE
  [8..12]  flags: u32 LE
  [12..16] entry_point: u32 LE
  [16..20] code_offset: u32 LE
  [20..24] code_size: u32 LE
  [24..28] rodata_offset: u32 LE
  [28..32] rodata_size: u32 LE
  [32..36] data_offset: u32 LE
  [36..40] data_size: u32 LE
  [40..44] bss_size: u32 LE
  [44..48] lump_dir_offset: u32 LE
  [48..52] lump_count: u32 LE
  [52..56] manifest_offset: u32 LE
  [56..60] manifest_size: u32 LE
  [60..64] crc32: u32 LE

Использование:
  python nhs_build.py script.nss -o app.nhs
  python nhs_build.py script.nss -o app.nhs --name "My App" --author "NeroShizaDev"
  python nhs_build.py script.nss -o app.nhs --lump logo.txt:TEX --lump sound.bin:SND
  python nhs_build.py --demo -o demo.nhs    # встроенный тестовый пакет
  python nhs_build.py --info app.nhs        # показать содержимое .nhs
"""

import sys
import struct
import argparse
import zlib
from pathlib import Path

# ============================================================
# Константы формата (должны совпадать с header.rs)
# ============================================================

NHS_MAGIC = b'NHS\x1a'
HEADER_SIZE = 64
MANIFEST_SIZE = 128
LUMP_ENTRY_SIZE = 16

# Флаги
FLAG_NEEDS_FPU     = 1 << 0
FLAG_NEEDS_PS2     = 1 << 1
FLAG_NEEDS_SPEAKER = 1 << 2
FLAG_NEEDS_GFX     = 1 << 3
FLAG_HAS_SCRIPT    = 1 << 4
FLAG_HAS_NATIVE    = 1 << 5

# Категории
CAT_GAME    = 0
CAT_TOOL    = 1
CAT_SCIENCE = 2
CAT_SYSTEM  = 3
CAT_SCRIPT  = 4
CAT_OTHER   = 5

CAT_NAMES = {
    'game': CAT_GAME, 'tool': CAT_TOOL, 'science': CAT_SCIENCE,
    'system': CAT_SYSTEM, 'script': CAT_SCRIPT, 'other': CAT_OTHER,
}

# Lump type tags
LUMP_TAGS = {
    'TEX': b'TEX\x00', 'SND': b'SND\x00', 'CFG': b'CFG\x00',
    'GFX': b'GFX\x00', 'SCR': b'SCR\x00', 'DAT': b'DAT\x00',
}

SLOT_SIZE = 64 * 1024  # 64KB макс


# ============================================================
# CRC32 (IEEE, совпадает с crc32.rs)
# ============================================================

def nhs_crc32(data: bytes) -> int:
    """CRC32 IEEE — тот же алгоритм что в crc32.rs."""
    return zlib.crc32(data) & 0xFFFFFFFF


# ============================================================
# Сборка .nhs
# ============================================================

def build_nhs(
    code: bytes,
    rodata: bytes = b'',
    data_section: bytes = b'',
    bss_size: int = 0,
    lumps: list = None,      # [(tag: str, name: str, data: bytes), ...]
    app_name: str = "Unnamed",
    author: str = "NeroShizaDev",
    version: tuple = (1, 0, 0),
    flags: int = FLAG_HAS_SCRIPT,
    category: int = CAT_SCRIPT,
    entry_point: int = 0,
) -> bytes:
    """Собирает полный .nhs файл."""

    if lumps is None:
        lumps = []

    # ── Расчёт смещений ──────────────────────────────────────

    # Секции идут после header (64 байт):
    #   CODE → RODATA → LUMP_DIR → MANIFEST → LUMP_DATA

    code_offset = HEADER_SIZE
    code_size = len(code)

    rodata_offset = code_offset + code_size
    rodata_size = len(rodata)

    # Lump directory (WAD-style, 16 байт на запись)
    lump_dir_offset = rodata_offset + rodata_size
    lump_count = len(lumps)
    lump_dir_size = lump_count * LUMP_ENTRY_SIZE

    # Manifest (128 байт)
    manifest_offset = lump_dir_offset + lump_dir_size

    # Lump data (после manifest)
    lump_data_offset = manifest_offset + MANIFEST_SIZE

    # DATA section (после lump data)
    total_lump_data = sum(len(l[2]) for l in lumps)
    data_offset = lump_data_offset + total_lump_data
    data_size = len(data_section)

    # ── Собираем lump directory + data ────────────────────────

    lump_dir_bytes = bytearray()
    lump_data_bytes = bytearray()
    current_lump_offset = lump_data_offset

    for tag_str, name, lump_data in lumps:
        tag = LUMP_TAGS.get(tag_str.upper(), tag_str.encode('ascii')[:4].ljust(4, b'\x00'))
        name_hash = nhs_crc32(name.encode('utf-8'))

        # Lump entry: [4] tag, [4] offset, [4] size, [4] name_hash
        lump_dir_bytes += tag
        lump_dir_bytes += struct.pack('<I', current_lump_offset)
        lump_dir_bytes += struct.pack('<I', len(lump_data))
        lump_dir_bytes += struct.pack('<I', name_hash)

        lump_data_bytes += lump_data
        current_lump_offset += len(lump_data)

    # ── Собираем manifest (128 байт) ─────────────────────────

    manifest = bytearray(MANIFEST_SIZE)
    # app_name [0..32]
    name_bytes = app_name.encode('utf-8')[:31]
    manifest[0:len(name_bytes)] = name_bytes
    # author [32..64]
    author_bytes = author.encode('utf-8')[:31]
    manifest[32:32+len(author_bytes)] = author_bytes
    # version [64..68]
    manifest[64] = version[0] & 0xFF
    manifest[65] = version[1] & 0xFF
    manifest[66] = version[2] & 0xFF
    manifest[67] = 0
    # min_os_version [68..72]
    manifest[68:72] = b'\x00\x03\x00\x00'  # v0.3
    # required_flags [72..76]
    struct.pack_into('<I', manifest, 72, flags)
    # stack_size [76..80]
    struct.pack_into('<I', manifest, 76, 4096)
    # reserved [80..128] — оставляем нулями

    # ── Собираем payload (всё после header) ───────────────────

    payload = bytearray()
    payload += code                              # CODE
    payload += rodata                            # RODATA
    payload += lump_dir_bytes                    # LUMP_DIR
    payload += bytes(manifest)                   # MANIFEST
    payload += lump_data_bytes                   # LUMP_DATA
    payload += data_section                      # DATA

    # ── CRC32 payload ─────────────────────────────────────────

    checksum = nhs_crc32(bytes(payload))

    # ── Собираем header (64 байта) ────────────────────────────

    header = bytearray(HEADER_SIZE)
    # magic [0..4]
    header[0:4] = NHS_MAGIC
    # version_major [4..6]
    struct.pack_into('<H', header, 4, 1)
    # version_minor [6..8]
    struct.pack_into('<H', header, 6, 0)
    # flags [8..12]
    struct.pack_into('<I', header, 8, flags)
    # entry_point [12..16]
    struct.pack_into('<I', header, 12, entry_point)
    # code_offset [16..20]
    struct.pack_into('<I', header, 16, code_offset)
    # code_size [20..24]
    struct.pack_into('<I', header, 20, code_size)
    # rodata_offset [24..28]
    struct.pack_into('<I', header, 24, rodata_offset)
    # rodata_size [28..32]
    struct.pack_into('<I', header, 28, rodata_size)
    # data_offset [32..36]
    struct.pack_into('<I', header, 32, data_offset)
    # data_size [36..40]
    struct.pack_into('<I', header, 36, data_size)
    # bss_size [40..44]
    struct.pack_into('<I', header, 40, bss_size)
    # lump_dir_offset [44..48]
    struct.pack_into('<I', header, 44, lump_dir_offset)
    # lump_count [48..52]
    struct.pack_into('<I', header, 48, lump_count)
    # manifest_offset [52..56]
    struct.pack_into('<I', header, 52, manifest_offset)
    # manifest_size [56..60]
    struct.pack_into('<I', header, 56, MANIFEST_SIZE)
    # crc32 [60..64]
    struct.pack_into('<I', header, 60, checksum)

    return bytes(header) + bytes(payload)


# ============================================================
# Чтение / информация
# ============================================================

def read_nhs_info(data: bytes) -> dict:
    """Разбирает .nhs файл и возвращает словарь с метаданными."""
    if len(data) < HEADER_SIZE:
        return {'error': 'Too short'}
    if data[0:4] != NHS_MAGIC:
        return {'error': f'Bad magic: {data[0:4].hex()}'}

    info = {}
    info['magic'] = data[0:4].hex()
    info['version'] = f"{struct.unpack_from('<H', data, 4)[0]}.{struct.unpack_from('<H', data, 6)[0]}"
    info['flags'] = struct.unpack_from('<I', data, 8)[0]
    info['entry_point'] = struct.unpack_from('<I', data, 12)[0]
    info['code_offset'] = struct.unpack_from('<I', data, 16)[0]
    info['code_size'] = struct.unpack_from('<I', data, 20)[0]
    info['rodata_offset'] = struct.unpack_from('<I', data, 24)[0]
    info['rodata_size'] = struct.unpack_from('<I', data, 28)[0]
    info['data_offset'] = struct.unpack_from('<I', data, 32)[0]
    info['data_size'] = struct.unpack_from('<I', data, 36)[0]
    info['bss_size'] = struct.unpack_from('<I', data, 40)[0]
    info['lump_dir_offset'] = struct.unpack_from('<I', data, 44)[0]
    info['lump_count'] = struct.unpack_from('<I', data, 48)[0]
    info['manifest_offset'] = struct.unpack_from('<I', data, 52)[0]
    info['manifest_size'] = struct.unpack_from('<I', data, 56)[0]
    info['checksum'] = struct.unpack_from('<I', data, 60)[0]
    info['file_size'] = len(data)

    # Memory footprint
    info['memory_footprint'] = (
        info['code_size'] + info['rodata_size'] +
        info['data_size'] + info['bss_size']
    )

    # Verify CRC32
    if len(data) > HEADER_SIZE:
        payload = data[HEADER_SIZE:]
        computed = nhs_crc32(payload)
        info['crc32_valid'] = computed == info['checksum']
        info['crc32_computed'] = computed
    else:
        info['crc32_valid'] = False

    # Parse manifest
    mo = info['manifest_offset']
    if mo + MANIFEST_SIZE <= len(data):
        m = data[mo:]
        info['app_name'] = m[0:32].split(b'\x00')[0].decode('utf-8', errors='replace')
        info['author'] = m[32:64].split(b'\x00')[0].decode('utf-8', errors='replace')
        info['app_version'] = f"{m[64]}.{m[65]}.{m[66]}"

    # Parse lumps
    info['lumps'] = []
    ldo = info['lump_dir_offset']
    for i in range(info['lump_count']):
        base = ldo + i * LUMP_ENTRY_SIZE
        if base + LUMP_ENTRY_SIZE > len(data):
            break
        tag = data[base:base+4].split(b'\x00')[0].decode('ascii', errors='replace')
        offset = struct.unpack_from('<I', data, base + 4)[0]
        size = struct.unpack_from('<I', data, base + 8)[0]
        name_hash = struct.unpack_from('<I', data, base + 12)[0]
        info['lumps'].append({
            'tag': tag, 'offset': offset, 'size': size, 'name_hash': f'{name_hash:08X}'
        })

    # Flags description
    flag_names = []
    f = info['flags']
    if f & FLAG_NEEDS_FPU:     flag_names.append('FPU')
    if f & FLAG_NEEDS_PS2:     flag_names.append('PS2')
    if f & FLAG_NEEDS_SPEAKER: flag_names.append('SPEAKER')
    if f & FLAG_NEEDS_GFX:     flag_names.append('GFX')
    if f & FLAG_HAS_SCRIPT:    flag_names.append('SCRIPT')
    if f & FLAG_HAS_NATIVE:    flag_names.append('NATIVE')
    info['flags_desc'] = ', '.join(flag_names) if flag_names else 'NONE'

    # Fits in slot?
    info['fits_in_slot'] = info['memory_footprint'] <= SLOT_SIZE

    return info


def print_nhs_info(info: dict):
    """Красиво печатает информацию о .nhs файле."""
    if 'error' in info:
        print(f'[ERROR] {info["error"]}', file=sys.stderr)
        return

    print('╔══════════════════════════════════════════════╗')
    print('║          NHS PACKAGE INSPECTOR               ║')
    print('╠══════════════════════════════════════════════╣')
    print(f'║  Name:      {info.get("app_name", "?"):32s} ║')
    print(f'║  Author:    {info.get("author", "?"):32s} ║')
    print(f'║  Version:   {info.get("app_version", "?"):32s} ║')
    print(f'║  Format:    {info["version"]:32s}  ║')
    print(f'║  Flags:     {info["flags_desc"]:32s} ║')
    print('╠══════════════════════════════════════════════╣')
    print(f'║  File size:    {info["file_size"]:>8d} bytes               ║')
    print(f'║  Code:         {info["code_size"]:>8d} bytes @ {info["code_offset"]:#010x}  ║')
    print(f'║  RoData:       {info["rodata_size"]:>8d} bytes @ {info["rodata_offset"]:#010x}  ║')
    print(f'║  Data:         {info["data_size"]:>8d} bytes @ {info["data_offset"]:#010x}  ║')
    print(f'║  BSS:          {info["bss_size"]:>8d} bytes (zeroed)       ║')
    print(f'║  Memory:       {info["memory_footprint"]:>8d} bytes total         ║')
    print(f'║  Entry point:  {info["entry_point"]:#010x}                  ║')
    print(f'║  CRC32:        {info["checksum"]:08X} {"OK" if info["crc32_valid"] else "MISMATCH!":>17s}  ║')
    print(f'║  Fits slot:    {"YES (< 64KB)" if info["fits_in_slot"] else "NO (> 64KB)":>32s}  ║')

    if info['lumps']:
        print('╠══════════════════════════════════════════════╣')
        print(f'║  Lumps: {info["lump_count"]:>3d}                                  ║')
        for l in info['lumps']:
            print(f'║    [{l["tag"]:4s}] {l["size"]:>6d} bytes  hash={l["name_hash"]}  ║')

    print('╚══════════════════════════════════════════════╝')


# ============================================================
# Демо-пакет
# ============================================================

def build_demo_nhs() -> bytes:
    """Создаёт тестовый .nhs с NeroShizaScript-заглушкой."""
    # Простой "скрипт" — пока просто ASCII текст как placeholder
    code = b'# NeroShizaScript Demo\n'
    code += b'# This runs on x87 FPU stack machine\n'
    code += b'PUSH 42\n'
    code += b'PUSH 13\n'
    code += b'ADD\n'
    code += b'PRINT\n'
    code += b'# Result: 55\n'
    code += b'HALT\n'

    rodata = b'Hello from NHS demo!\x00Psychotown greets you!\x00'

    lumps = [
        ('TEX', 'banner.txt', b'=== NeroShizaDev-OS NHS Demo ===\n'),
        ('CFG', 'settings.cfg', b'color=0x0E\nspeed=fast\n'),
    ]

    return build_nhs(
        code=code,
        rodata=rodata,
        lumps=lumps,
        app_name='NHS Demo',
        author='NeroShizaDev',
        version=(1, 0, 0),
        flags=FLAG_HAS_SCRIPT | FLAG_NEEDS_PS2,
        category=CAT_SCRIPT,
    )


# ============================================================
# CLI
# ============================================================

def main():
    parser = argparse.ArgumentParser(
        description='NHS Builder — NeroShizaDev-OS Package Builder',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s script.nss -o app.nhs
  %(prog)s script.nss -o app.nhs --name "My Game" --author "Kotlew" --cat game
  %(prog)s script.nss -o app.nhs --lump logo.txt:TEX --lump music.bin:SND
  %(prog)s --demo -o demo.nhs
  %(prog)s --info app.nhs
        """,
    )

    parser.add_argument('input', nargs='?', help='Input script or binary file')
    parser.add_argument('-o', '--output', help='Output .nhs file')
    parser.add_argument('--name', default='Unnamed', help='App name (max 31 chars)')
    parser.add_argument('--author', default='NeroShizaDev', help='Author name')
    parser.add_argument('--version', default='1.0.0', help='Version (X.Y.Z)')
    parser.add_argument('--cat', choices=CAT_NAMES.keys(), default='script', help='Category')
    parser.add_argument('--lump', action='append', default=[],
                        help='Add lump: file.ext:TAG (TAG = TEX/SND/CFG/GFX/SCR/DAT)')
    parser.add_argument('--rodata', help='File to include as RODATA section')
    parser.add_argument('--data', help='File to include as DATA section')
    parser.add_argument('--bss', type=int, default=0, help='BSS size (bytes)')
    parser.add_argument('--fpu', action='store_true', help='Set FLAG_NEEDS_FPU')
    parser.add_argument('--gfx', action='store_true', help='Set FLAG_NEEDS_GFX')
    parser.add_argument('--speaker', action='store_true', help='Set FLAG_NEEDS_SPEAKER')
    parser.add_argument('--native', action='store_true', help='Set FLAG_HAS_NATIVE (no script)')
    parser.add_argument('--demo', action='store_true', help='Build demo package')
    parser.add_argument('--info', action='store_true', help='Inspect .nhs file')
    parser.add_argument('-v', '--verbose', action='store_true')

    args = parser.parse_args()

    # ── --info mode ───────────────────────────────────────────
    if args.info:
        if not args.input:
            parser.error('--info requires an input .nhs file')
        data = Path(args.input).read_bytes()
        info = read_nhs_info(data)
        print_nhs_info(info)
        return

    # ── --demo mode ───────────────────────────────────────────
    if args.demo:
        nhs = build_demo_nhs()
        if args.output:
            Path(args.output).write_bytes(nhs)
            print(f'[OK] Demo package → {args.output} ({len(nhs)} bytes)', file=sys.stderr)
            # Auto-inspect
            print_nhs_info(read_nhs_info(nhs))
        else:
            sys.stdout.buffer.write(nhs)
        return

    # ── Normal build ──────────────────────────────────────────
    if not args.input:
        parser.print_help()
        sys.exit(1)

    input_path = Path(args.input)
    if not input_path.exists():
        print(f'[ERROR] File not found: {args.input}', file=sys.stderr)
        sys.exit(1)

    code = input_path.read_bytes()

    # Parse version
    try:
        v_parts = [int(x) for x in args.version.split('.')]
        version = (v_parts[0], v_parts[1] if len(v_parts) > 1 else 0,
                   v_parts[2] if len(v_parts) > 2 else 0)
    except (ValueError, IndexError):
        version = (1, 0, 0)

    # Flags
    flags = FLAG_HAS_SCRIPT | FLAG_NEEDS_PS2
    if args.fpu:     flags |= FLAG_NEEDS_FPU
    if args.gfx:     flags |= FLAG_NEEDS_GFX
    if args.speaker: flags |= FLAG_NEEDS_SPEAKER
    if args.native:
        flags = (flags & ~FLAG_HAS_SCRIPT) | FLAG_HAS_NATIVE

    # Rodata
    rodata = b''
    if args.rodata:
        rodata = Path(args.rodata).read_bytes()

    # Data
    data_section = b''
    if args.data:
        data_section = Path(args.data).read_bytes()

    # Lumps
    lumps = []
    for lump_spec in args.lump:
        if ':' not in lump_spec:
            print(f'[WARN] Bad lump spec "{lump_spec}", expected file:TAG', file=sys.stderr)
            continue
        fpath, tag = lump_spec.rsplit(':', 1)
        lump_path = Path(fpath)
        if not lump_path.exists():
            print(f'[WARN] Lump file not found: {fpath}', file=sys.stderr)
            continue
        lumps.append((tag.upper(), lump_path.name, lump_path.read_bytes()))

    # Auto-detect name from filename if not specified
    name = args.name
    if name == 'Unnamed':
        name = input_path.stem.replace('_', ' ').title()

    # Build
    nhs = build_nhs(
        code=code,
        rodata=rodata,
        data_section=data_section,
        bss_size=args.bss,
        lumps=lumps,
        app_name=name,
        author=args.author,
        version=version,
        flags=flags,
        category=CAT_NAMES[args.cat],
    )

    # Check size
    footprint = len(code) + len(rodata) + len(data_section) + args.bss
    if footprint > SLOT_SIZE:
        print(f'[WARN] Memory footprint {footprint} > {SLOT_SIZE} (slot overflow!)', file=sys.stderr)

    # Output
    if args.output:
        Path(args.output).write_bytes(nhs)
        print(f'[OK] {args.output} ({len(nhs)} bytes)', file=sys.stderr)
        if args.verbose:
            print_nhs_info(read_nhs_info(nhs))
    else:
        sys.stdout.buffer.write(nhs)

    if args.verbose:
        print(f'[NHS] code={len(code)}  rodata={len(rodata)}  data={len(data_section)}  '
              f'bss={args.bss}  lumps={len(lumps)}  total={len(nhs)}', file=sys.stderr)


if __name__ == '__main__':
    main()
