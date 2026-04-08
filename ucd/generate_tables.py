#!/usr/bin/env python3
"""
NeroShiza UCD Generator — Превращает Unicode Character Database в Rust-таблицы.
Читает UnicodeData.txt, Blocks.txt, Scripts.txt и генерирует:
  1. unicode_blocks.rs — полная таблица всех 329 блоков Unicode 17.0
  2. unicode_scripts.rs — таблица скриптов (165 скриптов)
  3. unicode_categories.rs — General Category для ВСЕХ 159,801 символов
     (сжатые диапазоны, не по одному символу)
"""

import re
from collections import OrderedDict

UCD_DIR = "."

# ============================================================
# 1. Blocks.txt → unicode_blocks.rs
# ============================================================
def parse_blocks():
    blocks = []
    with open(f"{UCD_DIR}/Blocks.txt", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#") or line.startswith("@"):
                continue
            m = re.match(r"([0-9A-F]+)\.\.([0-9A-F]+);\s*(.+)", line)
            if m:
                start = int(m.group(1), 16)
                end = int(m.group(2), 16)
                name = m.group(3).strip()
                blocks.append((start, end, name))
    return blocks

def generate_blocks_rs(blocks):
    lines = []
    lines.append("// ============================================================")
    lines.append("// АВТОСГЕНЕРИРОВАНО из Unicode 17.0 Blocks.txt")
    lines.append(f"// Всего блоков: {len(blocks)}")
    lines.append("// NeroShiza UCD Generator — Вселенская Карта Блоков")
    lines.append("// ============================================================")
    lines.append("")
    lines.append("/// Блок Unicode — диапазон кодпоинтов с именем")
    lines.append("#[derive(Debug, Clone, Copy)]")
    lines.append("pub struct UnicodeBlock {")
    lines.append("    pub start: u32,")
    lines.append("    pub end: u32,")
    lines.append("    pub name: &'static str,")
    lines.append("}")
    lines.append("")
    lines.append(f"/// Все {len(blocks)} блоков Unicode 17.0")
    lines.append(f"pub static UNICODE_BLOCKS: [UnicodeBlock; {len(blocks)}] = [")
    for (start, end, name) in blocks:
        lines.append(f'    UnicodeBlock {{ start: 0x{start:04X}, end: 0x{end:04X}, name: "{name}" }},')
    lines.append("];")
    lines.append("")
    lines.append("/// Бинарный поиск блока по кодпоинту — O(log n), ~8 сравнений")
    lines.append("pub fn find_block(cp: u32) -> Option<&'static UnicodeBlock> {")
    lines.append("    let mut lo = 0usize;")
    lines.append(f"    let mut hi = {len(blocks)};")
    lines.append("    while lo < hi {")
    lines.append("        let mid = lo + (hi - lo) / 2;")
    lines.append("        let b = &UNICODE_BLOCKS[mid];")
    lines.append("        if cp < b.start {")
    lines.append("            hi = mid;")
    lines.append("        } else if cp > b.end {")
    lines.append("            lo = mid + 1;")
    lines.append("        } else {")
    lines.append("            return Some(b);")
    lines.append("        }")
    lines.append("    }")
    lines.append("    None")
    lines.append("}")
    lines.append("")
    lines.append("/// Количество блоков")
    lines.append("pub fn block_count() -> usize {")
    lines.append("    UNICODE_BLOCKS.len()")
    lines.append("}")
    return "\n".join(lines) + "\n"


# ============================================================
# 2. Scripts.txt → unicode_scripts.rs
# ============================================================
def parse_scripts():
    """Parse Scripts.txt into list of (start, end, script_name)"""
    ranges = []
    with open(f"{UCD_DIR}/Scripts.txt", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            # Format: XXXX..YYYY ; Script_Name # ...
            # or:     XXXX       ; Script_Name # ...
            m = re.match(r"([0-9A-F]+)(?:\.\.([0-9A-F]+))?\s*;\s*(\w+)", line)
            if m:
                start = int(m.group(1), 16)
                end = int(m.group(2), 16) if m.group(2) else start
                script = m.group(3)
                ranges.append((start, end, script))
    # Sort by start
    ranges.sort(key=lambda x: x[0])
    return ranges

def collect_script_names(ranges):
    """Get unique script names in order of first appearance"""
    seen = set()
    names = []
    for _, _, s in ranges:
        if s not in seen:
            seen.add(s)
            names.append(s)
    return sorted(names)

def generate_scripts_rs(ranges):
    script_names = collect_script_names(ranges)
    
    lines = []
    lines.append("// ============================================================")
    lines.append("// АВТОСГЕНЕРИРОВАНО из Unicode 17.0 Scripts.txt")
    lines.append(f"// Всего скриптов: {len(script_names)}")
    lines.append(f"// Всего диапазонов: {len(ranges)}")
    lines.append("// NeroShiza UCD Generator — Карта Скриптов Мира")
    lines.append("// ============================================================")
    lines.append("")
    
    # Enum
    lines.append("/// Скрипт Unicode — система письменности")
    lines.append("#[derive(Debug, Clone, Copy, PartialEq, Eq)]")
    lines.append("#[repr(u8)]")
    lines.append("#[allow(non_camel_case_types)]")
    lines.append("pub enum Script {")
    for i, name in enumerate(script_names):
        lines.append(f"    {name} = {i},")
    lines.append("}")
    lines.append("")
    
    # Script name method
    lines.append("impl Script {")
    lines.append("    pub fn name(self) -> &'static str {")
    lines.append("        match self {")
    for name in script_names:
        # Convert CamelCase to readable
        readable = re.sub(r'_', ' ', name)
        lines.append(f'            Script::{name} => "{readable}",')
    lines.append("        }")
    lines.append("    }")
    lines.append("}")
    lines.append("")
    
    # Ranges table (compressed: store script as u8 index)
    lines.append("/// Диапазоны скриптов (start, end, Script)")
    lines.append(f"static SCRIPT_RANGES: [(u32, u32, Script); {len(ranges)}] = [")
    for (start, end, script) in ranges:
        lines.append(f"    (0x{start:04X}, 0x{end:04X}, Script::{script}),")
    lines.append("];")
    lines.append("")
    
    # Lookup function
    lines.append("/// Бинарный поиск скрипта по кодпоинту — O(log n)")
    lines.append("pub fn find_script(cp: u32) -> Script {")
    lines.append("    let mut lo = 0usize;")
    lines.append(f"    let mut hi = {len(ranges)};")
    lines.append("    while lo < hi {")
    lines.append("        let mid = lo + (hi - lo) / 2;")
    lines.append("        let (start, end, script) = SCRIPT_RANGES[mid];")
    lines.append("        if cp < start {")
    lines.append("            hi = mid;")
    lines.append("        } else if cp > end {")
    lines.append("            lo = mid + 1;")
    lines.append("        } else {")
    lines.append("            return script;")
    lines.append("        }")
    lines.append("    }")
    lines.append("    Script::Common // По умолчанию")
    lines.append("}")
    lines.append("")
    lines.append("/// Количество скриптов")
    lines.append("pub fn script_count() -> usize {")
    lines.append(f"    {len(script_names)}")
    lines.append("}")
    return "\n".join(lines) + "\n"


# ============================================================
# 3. UnicodeData.txt → unicode_categories.rs
# ============================================================
def parse_unicode_data():
    """Parse UnicodeData.txt into compressed ranges of (start, end, category)"""
    entries = []
    in_range = False
    range_start = 0
    range_cat = ""
    
    with open(f"{UCD_DIR}/UnicodeData.txt", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            fields = line.split(";")
            cp = int(fields[0], 16)
            name = fields[1]
            cat = fields[2]
            
            if name.endswith(", First>"):
                in_range = True
                range_start = cp
                range_cat = cat
                continue
            elif name.endswith(", Last>"):
                entries.append((range_start, cp, cat))
                in_range = False
                continue
            
            entries.append((cp, cp, cat))
    
    # Compress consecutive entries with same category
    compressed = []
    for (start, end, cat) in entries:
        if compressed and compressed[-1][2] == cat and compressed[-1][1] + 1 == start:
            compressed[-1] = (compressed[-1][0], end, cat)
        else:
            compressed.append((start, end, cat))
    
    return compressed

def collect_categories(ranges):
    seen = set()
    cats = []
    for _, _, c in ranges:
        if c not in seen:
            seen.add(c)
            cats.append(c)
    return sorted(cats)

# Category descriptions
CAT_DESC = {
    "Lu": "Uppercase Letter", "Ll": "Lowercase Letter", "Lt": "Titlecase Letter",
    "Lm": "Modifier Letter", "Lo": "Other Letter",
    "Mn": "Nonspacing Mark", "Mc": "Spacing Mark", "Me": "Enclosing Mark",
    "Nd": "Decimal Number", "Nl": "Letter Number", "No": "Other Number",
    "Pc": "Connector Punctuation", "Pd": "Dash Punctuation",
    "Ps": "Open Punctuation", "Pe": "Close Punctuation",
    "Pi": "Initial Punctuation", "Pf": "Final Punctuation", "Po": "Other Punctuation",
    "Sm": "Math Symbol", "Sc": "Currency Symbol", "Sk": "Modifier Symbol", "So": "Other Symbol",
    "Zs": "Space Separator", "Zl": "Line Separator", "Zp": "Paragraph Separator",
    "Cc": "Control", "Cf": "Format", "Cs": "Surrogate", "Co": "Private Use", "Cn": "Unassigned",
}

def generate_categories_rs(ranges):
    cats = collect_categories(ranges)
    total_chars = sum(end - start + 1 for start, end, _ in ranges)
    
    lines = []
    lines.append("// ============================================================")
    lines.append("// АВТОСГЕНЕРИРОВАНО из Unicode 17.0 UnicodeData.txt")
    lines.append(f"// Всего символов: {total_chars:,}")
    lines.append(f"// Сжатых диапазонов: {len(ranges):,}")
    lines.append(f"// Категорий: {len(cats)}")
    lines.append("// NeroShiza UCD Generator — Категории Всех Символов Мира")
    lines.append("// ============================================================")
    lines.append("")
    
    # Enum
    lines.append("/// General Category Unicode — тип символа")
    lines.append("#[derive(Debug, Clone, Copy, PartialEq, Eq)]")
    lines.append("#[repr(u8)]")
    lines.append("#[allow(non_camel_case_types)]")
    lines.append("pub enum GeneralCategory {")
    for i, cat in enumerate(cats):
        desc = CAT_DESC.get(cat, cat)
        lines.append(f"    {cat} = {i}, // {desc}")
    lines.append("}")
    lines.append("")
    
    lines.append("impl GeneralCategory {")
    lines.append("    pub fn name(self) -> &'static str {")
    lines.append("        match self {")
    for cat in cats:
        desc = CAT_DESC.get(cat, cat)
        lines.append(f'            GeneralCategory::{cat} => "{desc}",')
    lines.append("        }")
    lines.append("    }")
    lines.append("")
    lines.append("    /// Это буква?")
    lines.append("    pub fn is_letter(self) -> bool {")
    lines.append("        matches!(self, GeneralCategory::Lu | GeneralCategory::Ll | GeneralCategory::Lt | GeneralCategory::Lm | GeneralCategory::Lo)")
    lines.append("    }")
    lines.append("")
    lines.append("    /// Это цифра?")
    lines.append("    pub fn is_number(self) -> bool {")
    lines.append("        matches!(self, GeneralCategory::Nd | GeneralCategory::Nl | GeneralCategory::No)")
    lines.append("    }")
    lines.append("")
    lines.append("    /// Это пунктуация?")
    lines.append("    pub fn is_punctuation(self) -> bool {")
    lines.append("        matches!(self, GeneralCategory::Pc | GeneralCategory::Pd | GeneralCategory::Ps | GeneralCategory::Pe | GeneralCategory::Pi | GeneralCategory::Pf | GeneralCategory::Po)")
    lines.append("    }")
    lines.append("}")
    lines.append("")
    
    # Ranges table
    lines.append(f"/// Сжатые диапазоны General Category ({len(ranges)} записей)")
    lines.append(f"static CATEGORY_RANGES: [(u32, u32, GeneralCategory); {len(ranges)}] = [")
    for (start, end, cat) in ranges:
        if start == end:
            lines.append(f"    (0x{start:04X}, 0x{end:04X}, GeneralCategory::{cat}),")
        else:
            count = end - start + 1
            lines.append(f"    (0x{start:04X}, 0x{end:04X}, GeneralCategory::{cat}), // {count} chars")
    lines.append("];")
    lines.append("")
    
    # Lookup
    lines.append("/// Бинарный поиск категории по кодпоинту — O(log n)")
    lines.append("pub fn find_category(cp: u32) -> GeneralCategory {")
    lines.append("    let mut lo = 0usize;")
    lines.append(f"    let mut hi = {len(ranges)};")
    lines.append("    while lo < hi {")
    lines.append("        let mid = lo + (hi - lo) / 2;")
    lines.append("        let (start, end, cat) = CATEGORY_RANGES[mid];")
    lines.append("        if cp < start {")
    lines.append("            hi = mid;")
    lines.append("        } else if cp > end {")
    lines.append("            lo = mid + 1;")
    lines.append("        } else {")
    lines.append("            return cat;")
    lines.append("        }")
    lines.append("    }")
    lines.append("    GeneralCategory::Cn // Unassigned")
    lines.append("}")
    lines.append("")
    lines.append("/// Общее количество определённых символов")
    lines.append(f"pub fn total_defined_chars() -> usize {{ {total_chars} }}")
    lines.append("")
    lines.append("/// Количество сжатых диапазонов")
    lines.append("pub fn category_range_count() -> usize {")
    lines.append(f"    {len(ranges)}")
    lines.append("}")
    return "\n".join(lines) + "\n"


# ============================================================
# MAIN
# ============================================================
if __name__ == "__main__":
    print("=== NeroShiza UCD Generator ===")
    print("Unicode 17.0 → Rust tables")
    print()
    
    # Blocks
    print("Парсинг Blocks.txt...")
    blocks = parse_blocks()
    print(f"  Блоков: {len(blocks)}")
    blocks_rs = generate_blocks_rs(blocks)
    with open("../src/unicode_blocks.rs", "w", encoding="utf-8") as f:
        f.write(blocks_rs)
    print("  → src/unicode_blocks.rs")
    
    # Scripts
    print("Парсинг Scripts.txt...")
    scripts = parse_scripts()
    script_names = collect_script_names(scripts)
    print(f"  Скриптов: {len(script_names)}")
    print(f"  Диапазонов: {len(scripts)}")
    scripts_rs = generate_scripts_rs(scripts)
    with open("../src/unicode_scripts.rs", "w", encoding="utf-8") as f:
        f.write(scripts_rs)
    print("  → src/unicode_scripts.rs")
    
    # Categories
    print("Парсинг UnicodeData.txt...")
    categories = parse_unicode_data()
    cats = collect_categories(categories)
    total = sum(end - start + 1 for start, end, _ in categories)
    print(f"  Символов: {total:,}")
    print(f"  Сжатых диапазонов: {len(categories):,}")
    print(f"  Категорий: {len(cats)}")
    cats_rs = generate_categories_rs(categories)
    with open("../src/unicode_categories.rs", "w", encoding="utf-8") as f:
        f.write(cats_rs)
    print("  → src/unicode_categories.rs")
    
    # Stats
    print()
    print("=== ИТОГ ===")
    blocks_size = len(blocks) * (4 + 4 + 8)  # rough: start + end + ptr
    scripts_size = len(scripts) * (4 + 4 + 1)  # start + end + u8
    cats_size = len(categories) * (4 + 4 + 1)  # start + end + u8
    total_size = blocks_size + scripts_size + cats_size
    print(f"Блоки:     {len(blocks):>5} записей  (~{blocks_size:,} байт)")
    print(f"Скрипты:   {len(scripts):>5} записей  (~{scripts_size:,} байт)")
    print(f"Категории: {len(categories):>5} записей  (~{cats_size:,} байт)")
    print(f"ВСЕГО в RAM: ~{total_size:,} байт ({total_size/1024:.1f} KB)")
    print(f"Покрытие: {total:,} символов Unicode 17.0")
    print()
    print("Юникод-монстр сгенерирован. 🔥")
