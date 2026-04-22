// ============================================================
// NHS HEADER — формат .nhs по спецификации DEVELOPMENT_PLAN.md
// ============================================================
//
// ┌──────────────────────────────────────────┐  offset 0
// │  MAGIC:  4E 48 53 1A  ("NHS\x1A")        │  4 bytes
// │  VERSION: u16 major + u16 minor           │  4 bytes
// │  FLAGS: u32  (bits: needs_fpu|needs_ps2)  │  4 bytes
// │  ENTRY_POINT: u32  (offset от начала)     │  4 bytes
// │  CODE_OFFSET: u32 + CODE_SIZE: u32        │  8 bytes
// │  RODATA_OFFSET: u32 + RODATA_SIZE: u32    │  8 bytes
// │  DATA_OFFSET: u32 + DATA_SIZE: u32        │  8 bytes
// │  BSS_SIZE: u32  (нули при загрузке)       │  4 bytes
// │  LUMP_DIR_OFFSET: u32 + LUMP_COUNT: u32   │  8 bytes
// │  MANIFEST_OFFSET: u32 + MANIFEST_SIZE:u32 │  8 bytes
// │  CRC32: u32  (checksum всего файла)       │  4 bytes
// ├──────────────────────────────────────────┤  offset 64
// │  CODE SECTION  (position-independent)     │
// ├──────────────────────────────────────────┤
// │  RODATA SECTION  (строки, таблицы)        │
// ├──────────────────────────────────────────┤
// │  LUMP DIRECTORY  (WAD-style, 16 bytes ea) │
// │    [4] type_tag ("TEX","SND","CFG","GFX") │
// │    [4] offset (от начала файла)           │
// │    [4] size                               │
// │    [4] name_hash (CRC32 имени)            │
// ├──────────────────────────────────────────┤
// │  MANIFEST  (фиксированные 128 bytes)      │
// │    [32] app_name  (UTF-8, zero-padded)    │
// │    [32] author    (UTF-8)                 │
// │    [4]  version   (major.minor.patch.0)   │
// │    [4]  min_os_version                    │
// │    [4]  required_flags                    │
// │    [4]  stack_size (сколько стека нужно)  │
// │    [48] reserved                          │
// ├──────────────────────────────────────────┤
// │  LUMP DATA (TEX, SND, CFG, GFX блоки)    │
// └──────────────────────────────────────────┘  EOF
// ============================================================

/// Magic bytes: "NHS\x1A"
pub const NHS_MAGIC: [u8; 4] = [b'N', b'H', b'S', 0x1A];

pub const HEADER_SIZE:     usize = 64;
pub const MANIFEST_SIZE:   usize = 128;
pub const LUMP_ENTRY_SIZE: usize = 16;

// ── Флаги ────────────────────────────────────────────────────

pub const FLAG_NEEDS_FPU:     u32 = 1 << 0;
pub const FLAG_NEEDS_PS2:     u32 = 1 << 1;
pub const FLAG_NEEDS_SPEAKER: u32 = 1 << 2;
pub const FLAG_NEEDS_GFX:     u32 = 1 << 3;
pub const FLAG_HAS_SCRIPT:    u32 = 1 << 4;
pub const FLAG_HAS_NATIVE:    u32 = 1 << 5;

// ── Lump type tags ───────────────────────────────────────────

pub const LUMP_TEX: [u8; 4] = *b"TEX\0";
pub const LUMP_SND: [u8; 4] = *b"SND\0";
pub const LUMP_CFG: [u8; 4] = *b"CFG\0";
pub const LUMP_GFX: [u8; 4] = *b"GFX\0";
pub const LUMP_SCR: [u8; 4] = *b"SCR\0";
pub const LUMP_DAT: [u8; 4] = *b"DAT\0";

// ── Категории ────────────────────────────────────────────────

pub const CAT_GAME:    u8 = 0;
pub const CAT_TOOL:    u8 = 1;
pub const CAT_SCIENCE: u8 = 2;
pub const CAT_SYSTEM:  u8 = 3;
pub const CAT_SCRIPT:  u8 = 4;
pub const CAT_OTHER:   u8 = 5;

// ============================================================
// NhsHeader — ровно 64 байта
// ============================================================

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NhsHeader {
    pub magic:           [u8; 4],   //  0..4
    pub version_major:   u16,       //  4..6
    pub version_minor:   u16,       //  6..8
    pub flags:           u32,       //  8..12
    pub entry_point:     u32,       // 12..16
    pub code_offset:     u32,       // 16..20
    pub code_size:       u32,       // 20..24
    pub rodata_offset:   u32,       // 24..28
    pub rodata_size:     u32,       // 28..32
    pub data_offset:     u32,       // 32..36
    pub data_size:       u32,       // 36..40
    pub bss_size:        u32,       // 40..44
    pub lump_dir_offset: u32,       // 44..48
    pub lump_count:      u32,       // 48..52
    pub manifest_offset: u32,       // 52..56
    pub manifest_size:   u32,       // 56..60
    pub checksum:        u32,       // 60..64
}

const _: () = assert!(core::mem::size_of::<NhsHeader>() == HEADER_SIZE);

impl NhsHeader {
    pub fn is_valid_magic(&self) -> bool { self.magic == NHS_MAGIC }

    pub fn memory_footprint(&self) -> u32 {
        self.code_size
            .saturating_add(self.rodata_size)
            .saturating_add(self.data_size)
            .saturating_add(self.bss_size)
    }
}

// ============================================================
// LumpEntry — WAD-style, 16 байт
// ============================================================

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LumpEntry {
    pub type_tag:  [u8; 4],   //  0..4
    pub offset:    u32,       //  4..8   (от начала файла)
    pub size:      u32,       //  8..12
    pub name_hash: u32,       // 12..16  (CRC32 имени)
}

const _: () = assert!(core::mem::size_of::<LumpEntry>() == LUMP_ENTRY_SIZE);

impl LumpEntry {
    pub fn tag_str(&self) -> &str {
        let end = self.type_tag.iter().position(|&b| b == 0).unwrap_or(4);
        core::str::from_utf8(&self.type_tag[..end]).unwrap_or("???")
    }
    pub fn is_type(&self, tag: &[u8; 4]) -> bool { self.type_tag == *tag }
}

// ============================================================
// NhsManifest — 128 байт
// ============================================================

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NhsManifest {
    pub app_name:       [u8; 32],   //  0..32
    pub author:         [u8; 32],   // 32..64
    pub version:        [u8; 4],    // 64..68  (major.minor.patch.0)
    pub min_os_version: [u8; 4],    // 68..72
    pub required_flags: u32,        // 72..76
    pub stack_size:     u32,        // 76..80
    pub _reserved:      [u8; 48],   // 80..128
}

const _: () = assert!(core::mem::size_of::<NhsManifest>() == MANIFEST_SIZE);

impl NhsManifest {
    pub fn name_str(&self) -> &str {
        let end = self.app_name.iter().position(|&b| b == 0).unwrap_or(32);
        core::str::from_utf8(&self.app_name[..end]).unwrap_or("???")
    }
    pub fn author_str(&self) -> &str {
        let end = self.author.iter().position(|&b| b == 0).unwrap_or(32);
        core::str::from_utf8(&self.author[..end]).unwrap_or("???")
    }
    pub fn version_tuple(&self) -> (u8, u8, u8) {
        (self.version[0], self.version[1], self.version[2])
    }
    pub fn category_tag(&self) -> &'static [u8] {
        // category не в структуре по спецификации — используем reserved[0]
        // как расширение (backward-compatible)
        b"[App]   "
    }
}

// ============================================================
// Парсинг из &[u8]
// ============================================================

pub fn parse_header(data: &[u8]) -> Option<NhsHeader> {
    if data.len() < HEADER_SIZE { return None; }
    let h = NhsHeader {
        magic:           [data[0], data[1], data[2], data[3]],
        version_major:   u16_le(data, 4),
        version_minor:   u16_le(data, 6),
        flags:           u32_le(data, 8),
        entry_point:     u32_le(data, 12),
        code_offset:     u32_le(data, 16),
        code_size:       u32_le(data, 20),
        rodata_offset:   u32_le(data, 24),
        rodata_size:     u32_le(data, 28),
        data_offset:     u32_le(data, 32),
        data_size:       u32_le(data, 36),
        bss_size:        u32_le(data, 40),
        lump_dir_offset: u32_le(data, 44),
        lump_count:      u32_le(data, 48),
        manifest_offset: u32_le(data, 52),
        manifest_size:   u32_le(data, 56),
        checksum:        u32_le(data, 60),
    };
    if h.is_valid_magic() { Some(h) } else { None }
}

pub fn parse_manifest(data: &[u8], offset: u32) -> Option<NhsManifest> {
    let off = offset as usize;
    if off + MANIFEST_SIZE > data.len() { return None; }
    let m = &data[off..];
    let mut manifest = NhsManifest {
        app_name: [0; 32], author: [0; 32], version: [0; 4],
        min_os_version: [0; 4], required_flags: 0, stack_size: 0,
        _reserved: [0; 48],
    };
    manifest.app_name.copy_from_slice(&m[0..32]);
    manifest.author.copy_from_slice(&m[32..64]);
    manifest.version.copy_from_slice(&m[64..68]);
    manifest.min_os_version.copy_from_slice(&m[68..72]);
    manifest.required_flags = u32_le(m, 72);
    manifest.stack_size     = u32_le(m, 76);
    Some(manifest)
}

pub const MAX_LUMPS: usize = 64;

pub fn parse_lump_dir(
    data: &[u8], dir_off: u32, count: u32,
) -> ([LumpEntry; MAX_LUMPS], usize) {
    let empty = LumpEntry { type_tag: [0; 4], offset: 0, size: 0, name_hash: 0 };
    let mut lumps = [empty; MAX_LUMPS];
    let n = (count as usize).min(MAX_LUMPS);
    let off = dir_off as usize;
    for i in 0..n {
        let base = off + i * LUMP_ENTRY_SIZE;
        if base + LUMP_ENTRY_SIZE > data.len() { return (lumps, i); }
        lumps[i].type_tag.copy_from_slice(&data[base..base + 4]);
        lumps[i].offset    = u32_le(data, base + 4);
        lumps[i].size      = u32_le(data, base + 8);
        lumps[i].name_hash = u32_le(data, base + 12);
    }
    (lumps, n)
}

pub fn find_lump(lumps: &[LumpEntry], count: usize, tag: &[u8; 4]) -> Option<(u32, u32)> {
    for i in 0..count {
        if lumps[i].is_type(tag) { return Some((lumps[i].offset, lumps[i].size)); }
    }
    None
}

#[inline] fn u16_le(d: &[u8], o: usize) -> u16 {
    (d[o] as u16) | ((d[o+1] as u16) << 8)
}
#[inline] fn u32_le(d: &[u8], o: usize) -> u32 {
    (d[o] as u32) | ((d[o+1] as u32)<<8) | ((d[o+2] as u32)<<16) | ((d[o+3] as u32)<<24)
}
