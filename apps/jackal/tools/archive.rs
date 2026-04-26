// ============================================================
// ARCHIVE — Rust-версия pack.py / unpack.py
// ============================================================
// Python-оригиналы:
//   apps/jackal/scripts/pack.py    — zipfile.ZIP_DEFLATED упаковщик
//   apps/jackal/scripts/unpack.py  — extractall распаковщик
//
// В ядре нет ZIP-библиотеки и файловой системы.
// Поэтому реализуем собственный минимальный контейнер JKL-ARCH:
//
//   Формат JKL-ARCH (многоблочный архив):
//   ┌─────────────────────────────────────┐
//   │  ARCH magic: b"JKLA\x01"  (5 байт) │
//   │  block_count: u16 LE               │
//   ├─────────────────────────────────────┤
//   │  BLOCK[0]:                          │
//   │    name_len: u8                     │
//   │    name: [u8; name_len]  (ASCII)    │
//   │    algorithm: u8  (0=store,1=rle)   │
//   │    original_size: u32 LE            │
//   │    encoded_size: u32 LE             │
//   │    data: [u8; encoded_size]         │
//   ├─────────────────────────────────────┤
//   │  BLOCK[1] ... BLOCK[N-1]            │
//   └─────────────────────────────────────┘
//
// Ограничения (in-kernel):
//   MAX_BLOCKS = 16    (хватит для DOCX / ZIP-like структур)
//   MAX_NAME   = 64    байт
//   MAX_DATA   = 64 КБ на блок
// ============================================================

#![allow(dead_code)]

use crate::apps::jackal::encoder::{JKL_HEADER_SIZE, JKL_MAGIC};

pub const ARCH_MAGIC: &[u8; 5] = b"JKLA\x01";
pub const MAX_BLOCKS: usize = 16;
pub const MAX_NAME: usize = 64;
pub const MAX_DATA: usize = 64 * 1024;

// ============================================================
// Блок архива (в памяти)
// ============================================================
pub struct ArchBlock {
    pub name: [u8; MAX_NAME],
    pub name_len: usize,
    pub algorithm: u8, // 0 = store, 1 = RLE
    pub original_size: u32,
    pub data: [u8; MAX_DATA],
    pub data_len: usize, // encoded_size
}

impl ArchBlock {
    pub const fn empty() -> Self {
        Self {
            name: [0; MAX_NAME],
            name_len: 0,
            algorithm: 0,
            original_size: 0,
            data: [0; MAX_DATA],
            data_len: 0,
        }
    }

    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("?")
    }
}

// ============================================================
// Архив (в памяти)
// ============================================================
pub struct Archive {
    pub blocks: [ArchBlock; MAX_BLOCKS],
    pub count: usize,
}

impl Archive {
    pub const fn new() -> Self {
        const EMPTY: ArchBlock = ArchBlock::empty();
        Self {
            blocks: [EMPTY; MAX_BLOCKS],
            count: 0,
        }
    }

    /// Добавляет блок с именем `name` и данными `data` (store, без сжатия).
    /// Возвращает `false` если архив полон или блок слишком большой.
    pub fn add(&mut self, name: &[u8], data: &[u8]) -> bool {
        if self.count >= MAX_BLOCKS {
            return false;
        }
        if data.len() > MAX_DATA {
            return false;
        }
        if name.len() > MAX_NAME {
            return false;
        }

        let b = &mut self.blocks[self.count];
        let nlen = name.len().min(MAX_NAME);
        b.name[..nlen].copy_from_slice(&name[..nlen]);
        b.name_len = nlen;
        b.algorithm = 0; // store
        b.original_size = data.len() as u32;
        let dlen = data.len().min(MAX_DATA);
        b.data[..dlen].copy_from_slice(&data[..dlen]);
        b.data_len = dlen;

        self.count += 1;
        true
    }

    /// Добавляет блок из готового `.jkl` (header + payload).
    /// Берет algorithm/original/encoded из JKL-заголовка.
    pub fn add_jkl_block(&mut self, name: &[u8], jkl: &[u8]) -> bool {
        if jkl.len() < JKL_HEADER_SIZE {
            return false;
        }
        if jkl[0..4] != JKL_MAGIC {
            return false;
        }

        let algorithm = jkl[5];
        if algorithm > 2 {
            return false;
        }

        let original_size = u64::from_le_bytes([
            jkl[6], jkl[7], jkl[8], jkl[9], jkl[10], jkl[11], jkl[12], jkl[13],
        ]) as usize;
        let encoded_size = u64::from_le_bytes([
            jkl[14], jkl[15], jkl[16], jkl[17], jkl[18], jkl[19], jkl[20], jkl[21],
        ]) as usize;

        if JKL_HEADER_SIZE + encoded_size != jkl.len() {
            return false;
        }
        if original_size > u32::MAX as usize {
            return false;
        }
        if encoded_size > MAX_DATA {
            return false;
        }

        if !self.add(name, &jkl[JKL_HEADER_SIZE..]) {
            return false;
        }

        let b = &mut self.blocks[self.count - 1];
        b.algorithm = algorithm;
        b.original_size = original_size as u32;
        true
    }

    /// Ищет блок по имени. Возвращает &ArchBlock или None.
    pub fn find(&self, name: &[u8]) -> Option<&ArchBlock> {
        for i in 0..self.count {
            if self.blocks[i].name[..self.blocks[i].name_len] == *name {
                return Some(&self.blocks[i]);
            }
        }
        None
    }
}

/// Точный размер сериализованного архива в байтах.
pub fn packed_len(archive: &Archive) -> usize {
    let mut total = 5 + 2;
    for i in 0..archive.count {
        let b = &archive.blocks[i];
        total += 1 + b.name_len + 1 + 4 + 4 + b.data_len;
    }
    total
}

// ============================================================
// PACK — сериализация Archive → байтовый буфер (аналог pack.py)
// ============================================================

/// Сериализует `archive` в `out`. Возвращает количество записанных байт.
/// Аналог: `python pack.py unpacked/ output.jkla`
pub fn pack(archive: &Archive, out: &mut [u8]) -> usize {
    let needed = packed_len(archive);
    if needed > out.len() {
        return 0;
    }

    let mut pos = 0usize;

    // Magic
    if pos + 5 > out.len() {
        return 0;
    }
    out[pos..pos + 5].copy_from_slice(ARCH_MAGIC);
    pos += 5;

    // block_count: u16 LE
    if pos + 2 > out.len() {
        return 0;
    }
    let count = archive.count as u16;
    out[pos] = count as u8;
    out[pos + 1] = (count >> 8) as u8;
    pos += 2;

    for i in 0..archive.count {
        let b = &archive.blocks[i];

        // name_len: u8
        out[pos] = b.name_len as u8;
        pos += 1;

        // name
        out[pos..pos + b.name_len].copy_from_slice(&b.name[..b.name_len]);
        pos += b.name_len;

        // algorithm: u8
        out[pos] = b.algorithm;
        pos += 1;

        // original_size: u32 LE
        let os = b.original_size;
        out[pos] = os as u8;
        out[pos + 1] = (os >> 8) as u8;
        out[pos + 2] = (os >> 16) as u8;
        out[pos + 3] = (os >> 24) as u8;
        pos += 4;

        // encoded_size: u32 LE
        let es = b.data_len as u32;
        out[pos] = es as u8;
        out[pos + 1] = (es >> 8) as u8;
        out[pos + 2] = (es >> 16) as u8;
        out[pos + 3] = (es >> 24) as u8;
        pos += 4;

        // data
        out[pos..pos + b.data_len].copy_from_slice(&b.data[..b.data_len]);
        pos += b.data_len;
    }

    pos
}

// ============================================================
// UNPACK — десериализация байтового буфера → Archive (аналог unpack.py)
// ============================================================

/// Разбирает бинарный буфер в Archive.
/// Аналог: `python unpack.py doc.jkla unpacked/`
/// Возвращает Ok(Archive) или Err(&'static str).
pub fn unpack(data: &[u8]) -> Result<Archive, &'static str> {
    if data.len() < 7 {
        return Err("too short");
    }

    // Magic
    if &data[0..5] != ARCH_MAGIC {
        return Err("bad magic");
    }
    let count = data[5] as usize | ((data[6] as usize) << 8);
    if count > MAX_BLOCKS {
        return Err("too many blocks");
    }

    let mut arch = Archive::new();
    let mut pos = 7usize;

    for _ in 0..count {
        if pos >= data.len() {
            return Err("truncated");
        }

        // name_len
        let name_len = data[pos] as usize;
        pos += 1;
        if name_len > MAX_NAME {
            return Err("name too long");
        }

        // name
        if pos + name_len > data.len() {
            return Err("truncated name");
        }
        let mut b = ArchBlock::empty();
        b.name[..name_len].copy_from_slice(&data[pos..pos + name_len]);
        b.name_len = name_len;
        pos += name_len;

        // algorithm
        if pos >= data.len() {
            return Err("truncated alg");
        }
        b.algorithm = data[pos];
        if b.algorithm > 2 {
            return Err("unknown algorithm");
        }
        pos += 1;

        // original_size
        if pos + 4 > data.len() {
            return Err("truncated orig_sz");
        }
        b.original_size = (data[pos] as u32)
            | ((data[pos + 1] as u32) << 8)
            | ((data[pos + 2] as u32) << 16)
            | ((data[pos + 3] as u32) << 24);
        pos += 4;

        // encoded_size
        if pos + 4 > data.len() {
            return Err("truncated enc_sz");
        }
        let enc_sz = (data[pos] as u32)
            | ((data[pos + 1] as u32) << 8)
            | ((data[pos + 2] as u32) << 16)
            | ((data[pos + 3] as u32) << 24);
        pos += 4;

        // data
        let enc_sz = enc_sz as usize;
        if enc_sz > MAX_DATA {
            return Err("block data too large");
        }
        if pos + enc_sz > data.len() {
            return Err("truncated data");
        }
        b.data[..enc_sz].copy_from_slice(&data[pos..pos + enc_sz]);
        b.data_len = enc_sz;
        pos += enc_sz;

        if arch.count >= MAX_BLOCKS {
            return Err("too many blocks");
        }
        arch.blocks[arch.count] = b;
        arch.count += 1;
    }

    Ok(arch)
}

/// \u0420\u0430\u0437\u0431\u0438\u0440\u0430\u0435\u0442 \u0431\u0438\u043d\u0430\u0440\u043d\u044b\u0439 \u0431\u0443\u0444\u0435\u0440 \u0432 `out` (in-place, \u0431\u0435\u0437 \u0432\u044b\u0434\u0435\u043b\u0435\u043d\u0438\u044f \u043d\u0430 \u0441\u0442\u0435\u043a\u0435).
/// \u0414\u043b\u044f \u0438\u0441\u043f\u043e\u043b\u044c\u0437\u043e\u0432\u0430\u043d\u0438\u044f \u0441\u043e static-\u0431\u0443\u0444\u0435\u0440\u0430\u043c\u0438 \u0432 \u044f\u0434\u0440\u0435.
pub fn unpack_into(data: &[u8], out: &mut Archive) -> Result<(), &'static str> {
    out.count = 0;
    if data.len() < 7 {
        return Err("too short");
    }
    if &data[0..5] != ARCH_MAGIC {
        return Err("bad magic");
    }
    let count = data[5] as usize | ((data[6] as usize) << 8);
    if count > MAX_BLOCKS {
        return Err("too many blocks");
    }
    let mut pos = 7usize;
    for _ in 0..count {
        if pos >= data.len() {
            return Err("truncated");
        }
        let name_len = data[pos] as usize;
        pos += 1;
        if name_len > MAX_NAME {
            return Err("name too long");
        }
        if pos + name_len > data.len() {
            return Err("truncated name");
        }
        let b = &mut out.blocks[out.count];
        b.name_len = name_len;
        b.name[..name_len].copy_from_slice(&data[pos..pos + name_len]);
        pos += name_len;
        if pos >= data.len() {
            return Err("truncated alg");
        }
        b.algorithm = data[pos];
        if b.algorithm > 2 {
            return Err("unknown algorithm");
        }
        pos += 1;
        if pos + 4 > data.len() {
            return Err("truncated orig_sz");
        }
        b.original_size = (data[pos] as u32)
            | ((data[pos + 1] as u32) << 8)
            | ((data[pos + 2] as u32) << 16)
            | ((data[pos + 3] as u32) << 24);
        pos += 4;
        if pos + 4 > data.len() {
            return Err("truncated enc_sz");
        }
        let enc_sz = ((data[pos] as u32)
            | ((data[pos + 1] as u32) << 8)
            | ((data[pos + 2] as u32) << 16)
            | ((data[pos + 3] as u32) << 24)) as usize;
        pos += 4;
        if enc_sz > MAX_DATA {
            return Err("block data too large");
        }
        if pos + enc_sz > data.len() {
            return Err("truncated data");
        }
        b.data[..enc_sz].copy_from_slice(&data[pos..pos + enc_sz]);
        b.data_len = enc_sz;
        pos += enc_sz;
        out.count += 1;
    }
    Ok(())
}
