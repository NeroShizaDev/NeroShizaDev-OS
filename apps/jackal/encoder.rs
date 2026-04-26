// ============================================================
// JACKAL ENCODER — байтовый архиватор с адаптивной стратегией
// ============================================================
// Принимает сырые данные + Report от analyzer, выбирает алгоритм:
//
//   FileKind::Text       → RLE (повторяющиеся байты) + escape-паки
//   FileKind::Structured → Delta-coding (разности между соседними байтами)
//   FileKind::Compressed
//   FileKind::Random     → Store (без сжатия — данные уже несжимаемы)
//   FileKind::Executable
//   FileKind::LossyMedia → Store (энтропия слишком высокая)
//   FileKind::Unknown    → Store (нет гипотезы → не рискуем)
//
// Формат блока .jkl:
//   [0..4]  magic: b"JKL\x01"
//   [4]     FileKind byte (0=Text,1=Exec,2=Comp,3=Rand,4=Strc,5=Media,6=?)
//   [5]     Algorithm byte (0=Store,1=RLE,2=Delta)
//   [6..14] original_size: u64 LE
//   [14..22] encoded_size: u64 LE
//   [22..]  encoded payload
// ============================================================

#![allow(dead_code)]

use super::analyzer::{FileKind, Report};

// ============================================================
// Константы формата
// ============================================================
pub const JKL_MAGIC: [u8; 4] = [b'J', b'K', b'L', 0x01];
pub const JKL_HEADER_SIZE: usize = 22;

pub const ALG_STORE: u8 = 0;
pub const ALG_RLE: u8 = 1;
pub const ALG_DELTA: u8 = 2;

// Максимальный размер выходного буфера (in-kernel: статический)
pub const MAX_ENCODED: usize = 256 * 1024; // 256 КБ

// ============================================================
// EncodedBlock — результат кодирования
// ============================================================
pub struct EncodedBlock {
    /// Алгоритм, которым закодировано
    pub algorithm: u8,
    /// Оригинальный FileKind
    pub kind: FileKind,
    /// Размер исходных данных
    pub original_size: u64,
    /// Размер закодированных данных (payload без заголовка)
    pub encoded_size: u64,
    /// Коэффициент: (encoded_size * 1000) / original_size (меньше = лучше сжатие)
    pub ratio_milli: u32,
    /// Статический буфер с закодированными данными
    pub buf: [u8; MAX_ENCODED],
}

impl EncodedBlock {
    /// Константный конструктор для инициализации static-буфера в ядре.
    pub const fn new() -> Self {
        Self {
            algorithm: ALG_STORE,
            kind: FileKind::Unknown,
            original_size: 0,
            encoded_size: 0,
            ratio_milli: 1000,
            buf: [0u8; MAX_ENCODED],
        }
    }

    fn new_empty(kind: FileKind) -> Self {
        Self {
            algorithm: ALG_STORE,
            kind,
            original_size: 0,
            encoded_size: 0,
            ratio_milli: 1000,
            buf: [0u8; MAX_ENCODED],
        }
    }
}

// ============================================================
// Главная функция кодирования
// ============================================================
/// Кодирует `data` с учётом `report` от analyzer.
/// Возвращает заполненный EncodedBlock.
pub fn encode(data: &[u8], report: &Report) -> EncodedBlock {
    let mut block = EncodedBlock::new_empty(report.classification);
    block.original_size = data.len() as u64;

    if data.is_empty() {
        return block;
    }

    // Выбираем алгоритм по FileKind
    let (algorithm, encoded_len) = match report.classification {
        FileKind::Text => {
            let n = rle_encode(data, &mut block.buf);
            // Если RLE не помогло — используем store
            if n < data.len() {
                (ALG_RLE, n)
            } else {
                store(data, &mut block.buf);
                (ALG_STORE, data.len())
            }
        }
        FileKind::Structured => {
            let n = delta_encode(data, &mut block.buf);
            if n < data.len() {
                (ALG_DELTA, n)
            } else {
                store(data, &mut block.buf);
                (ALG_STORE, data.len())
            }
        }
        // Всё остальное: store без попыток сжатия
        _ => {
            store(data, &mut block.buf);
            (ALG_STORE, data.len())
        }
    };

    block.algorithm = algorithm;
    block.encoded_size = encoded_len as u64;
    block.ratio_milli = if block.original_size > 0 {
        ((block.encoded_size * 1000) / block.original_size) as u32
    } else {
        1000
    };

    block
}

/// Кодирует `data` в существующий `block` (без выделения на стеке).
/// Для использования со static-буферами в ядре.
pub fn encode_into(data: &[u8], report: &Report, block: &mut EncodedBlock) {
    block.kind = report.classification;
    block.original_size = data.len() as u64;
    block.algorithm = ALG_STORE;
    block.encoded_size = 0;
    block.ratio_milli = 1000;

    if data.is_empty() {
        return;
    }

    let (algorithm, encoded_len) = match report.classification {
        FileKind::Text => {
            let n = rle_encode(data, &mut block.buf);
            if n < data.len() {
                (ALG_RLE, n)
            } else {
                store(data, &mut block.buf);
                (ALG_STORE, data.len())
            }
        }
        FileKind::Structured => {
            let n = delta_encode(data, &mut block.buf);
            if n < data.len() {
                (ALG_DELTA, n)
            } else {
                store(data, &mut block.buf);
                (ALG_STORE, data.len())
            }
        }
        _ => {
            store(data, &mut block.buf);
            (ALG_STORE, data.len())
        }
    };

    block.algorithm = algorithm;
    block.encoded_size = encoded_len as u64;
    block.ratio_milli = if block.original_size > 0 {
        ((block.encoded_size * 1000) / block.original_size) as u32
    } else {
        1000
    };
}

/// Записывает полный .jkl-заголовок в начало буфера `out` (22 байта).
/// Возвращает смещение после заголовка.
pub fn write_jkl_header(out: &mut [u8], block: &EncodedBlock) -> usize {
    if out.len() < JKL_HEADER_SIZE {
        return 0;
    }
    out[0..4].copy_from_slice(&JKL_MAGIC);
    out[4] = kind_byte(block.kind);
    out[5] = block.algorithm;
    out[6..14].copy_from_slice(&block.original_size.to_le_bytes());
    out[14..22].copy_from_slice(&block.encoded_size.to_le_bytes());
    JKL_HEADER_SIZE
}

fn kind_byte(k: FileKind) -> u8 {
    match k {
        FileKind::Text => 0,
        FileKind::Executable => 1,
        FileKind::Compressed => 2,
        FileKind::Random => 3,
        FileKind::Structured => 4,
        FileKind::LossyMedia => 5,
        FileKind::Unknown => 6,
    }
}

// ============================================================
// ALG_STORE — просто копируем байты
// ============================================================
fn store(data: &[u8], out: &mut [u8; MAX_ENCODED]) -> usize {
    let n = data.len().min(MAX_ENCODED);
    out[..n].copy_from_slice(&data[..n]);
    n
}

// ============================================================
// ALG_RLE — Run-Length Encoding
// ============================================================
// Формат: [count: u8][byte: u8] для серий >= 3
// Для одиночных байтов: [0x01][byte]
// Escape: count=0xFF значит «следующие 255 байтов — литералы»
// ============================================================
fn rle_encode(data: &[u8], out: &mut [u8; MAX_ENCODED]) -> usize {
    let mut pos = 0usize;
    let mut out_pos = 0usize;

    while pos < data.len() && out_pos + 2 <= MAX_ENCODED {
        let b = data[pos];
        let mut run = 1usize;

        while pos + run < data.len() && data[pos + run] == b && run < 255 {
            run += 1;
        }

        if run >= 3 || out_pos + 2 <= MAX_ENCODED {
            if out_pos + 2 > MAX_ENCODED {
                break;
            }
            out[out_pos] = run as u8;
            out[out_pos + 1] = b;
            out_pos += 2;
            pos += run;
        }
    }

    out_pos
}

// ============================================================
// ALG_DELTA — дельта-кодирование
// ============================================================
// Первый байт — без изменений, остальные = data[i] - data[i-1] (mod 256).
// Хорошо для BMP, WAV, raw sensor logs с плавными рядами.
// ============================================================
fn delta_encode(data: &[u8], out: &mut [u8; MAX_ENCODED]) -> usize {
    let n = data.len().min(MAX_ENCODED);
    if n == 0 {
        return 0;
    }
    out[0] = data[0];
    for i in 1..n {
        out[i] = data[i].wrapping_sub(data[i - 1]);
    }
    n
}

// ============================================================
// Декодирование (симметричное, нужно для верификации и shell)
// ============================================================
pub fn decode_rle(encoded: &[u8], out: &mut [u8], out_len: usize) -> usize {
    let mut src = 0usize;
    let mut dst = 0usize;
    while src + 1 < encoded.len() && dst < out_len {
        let run = encoded[src] as usize;
        let b = encoded[src + 1];
        src += 2;
        let put = run.min(out_len - dst);
        for i in 0..put {
            out[dst + i] = b;
        }
        dst += put;
    }
    dst
}

pub fn decode_delta(encoded: &[u8], out: &mut [u8], out_len: usize) -> usize {
    let n = encoded.len().min(out_len);
    if n == 0 {
        return 0;
    }
    out[0] = encoded[0];
    for i in 1..n {
        out[i] = encoded[i].wrapping_add(out[i - 1]);
    }
    n
}
