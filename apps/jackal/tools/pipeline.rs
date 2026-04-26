// ============================================================
// PIPELINE — Rust-версия pipeline.py
// ============================================================
// Python-оригинал: apps/jackal/scripts/pipeline.py
//
// Полный пайплайн: байты → анализ → кодирование → .jkl
// В ядре работает целиком in-memory без subprocess.
//
// Использование:
//   let result = Pipeline::run(data);
//   // result.jkl_buf содержит готовый .jkl-блок
//
// Дополнительно: extract_then_pack — принимает «грязный» формат,
// сначала очищает через extractor, потом кодирует.
// ============================================================

#![allow(dead_code)]

use super::extractor::{ExtractOpts, Format, extract};
use crate::apps::jackal::analyzer;
use crate::apps::jackal::encoder::{self, EncodedBlock};
use crate::apps::jackal::tools::archive;
use crate::apps::jackal::tools::validate::{self, ValidateResult};

// Статический буфер для пайплайна (два слота: stage + output)
const PIPE_BUF: usize = 256 * 1024;

/// Результат пайплайна
pub struct PipelineResult {
    /// Готовый .jkl с полным заголовком (JKL_HEADER_SIZE + payload)
    pub jkl_buf: [u8; PIPE_BUF],
    pub jkl_len: usize,
    /// Метрики для отладки / shell вывода
    pub original_size: u64,
    pub encoded_size: u64,
    pub ratio_milli: u32,
    pub algorithm: u8,
    pub kind_name: &'static str,
}

pub struct Pipeline;

impl Pipeline {
    // ============================================================
    // Основной метод: сырые байты → .jkl
    // ============================================================
    /// Аналог: `python pipeline.py input.bin output.jkl`
    pub fn run(data: &[u8]) -> PipelineResult {
        let report = analyzer::analyze(data);
        let block = encoder::encode(data, &report);

        Self::build_result(data.len() as u64, &block)
    }

    /// Пайплайн + валидация собранного JKL (сквозная связка tools).
    pub fn run_and_validate(data: &[u8]) -> (PipelineResult, ValidateResult) {
        let result = Self::run(data);
        let status = validate::validate(&result.jkl_buf[..result.jkl_len]);
        (result, status)
    }

    /// Сжатие в JKL + упаковка в JKLA (один блок), аналог связки pipeline.py + pack.py.
    /// Возвращает длину записанного архива в `out`, 0 при ошибке.
    pub fn run_to_archive(data: &[u8], block_name: &[u8], out: &mut [u8]) -> usize {
        let result = Self::run(data);
        if result.jkl_len == 0 {
            return 0;
        }

        let mut arch = archive::Archive::new();
        if !arch.add_jkl_block(block_name, &result.jkl_buf[..result.jkl_len]) {
            return 0;
        }

        archive::pack(&arch, out)
    }

    // ============================================================
    // Извлечение текста + упаковка (аналог text_exts ветки pipeline.py)
    // ============================================================
    /// Аналог: `python pipeline.py doc.html output.jkl`
    /// Сначала очищает текст через extractor, потом кодирует.
    pub fn extract_then_pack(raw: &[u8], mode: Format) -> PipelineResult {
        // Буфер для чистого текста
        let mut clean: [u8; 16 * 1024] = [0u8; 16 * 1024];
        let opts = ExtractOpts::default();
        let clean_len = extract(raw, &mut clean, mode, &opts);

        let clean_slice = &clean[..clean_len];
        let report = analyzer::analyze(clean_slice);
        let block = encoder::encode(clean_slice, &report);

        Self::build_result(raw.len() as u64, &block)
    }

    // ============================================================
    // Только анализ (аналог --analyze флага)
    // ============================================================
    /// Аналог: `python pipeline.py --analyze input.bin`
    /// Возвращает строку-отчёт в `dst`, длину.
    pub fn analyze_only(data: &[u8], dst: &mut [u8]) -> usize {
        let report = analyzer::analyze(data);
        let kind = match report.classification {
            analyzer::FileKind::Text => "TEXT",
            analyzer::FileKind::Executable => "EXECUTABLE",
            analyzer::FileKind::Compressed => "COMPRESSED",
            analyzer::FileKind::Random => "RANDOM",
            analyzer::FileKind::Structured => "STRUCTURED",
            analyzer::FileKind::LossyMedia => "LOSSY_MEDIA",
            analyzer::FileKind::Unknown => "UNKNOWN",
        };
        // Форматируем как строку (без alloc)
        let mut pos = 0usize;
        pos += write_str(dst, pos, "[PIPE] size=");
        pos += write_u64(dst, pos, data.len() as u64);
        pos += write_str(dst, pos, " kind=");
        pos += write_str(dst, pos, kind);
        pos += write_str(dst, pos, " H=");
        pos += write_u64(dst, pos, report.global_h_milli / 1000);
        pos += write_str(dst, pos, ".");
        pos += write_u64(dst, pos, report.global_h_milli % 1000);
        pos += write_str(dst, pos, " conf=");
        pos += write_u64(dst, pos, report.confidence_milli as u64);
        pos += write_str(dst, pos, "\n");
        pos
    }

    // ============================================================
    // Внутренний builder
    // ============================================================
    fn build_result(original_size: u64, block: &EncodedBlock) -> PipelineResult {
        let mut result = PipelineResult {
            jkl_buf: [0u8; PIPE_BUF],
            jkl_len: 0,
            original_size,
            encoded_size: block.encoded_size,
            ratio_milli: block.ratio_milli,
            algorithm: block.algorithm,
            kind_name: kind_str(block.kind),
        };

        // Записываем заголовок
        let hdr_len = encoder::write_jkl_header(&mut result.jkl_buf, block);
        // Payload
        let pay_len = block.encoded_size as usize;
        if hdr_len + pay_len <= PIPE_BUF {
            result.jkl_buf[hdr_len..hdr_len + pay_len].copy_from_slice(&block.buf[..pay_len]);
            result.jkl_len = hdr_len + pay_len;
        }

        result
    }
}

// ============================================================
// Вспомогательные форматтеры (без alloc/format!)
// ============================================================

fn kind_str(k: analyzer::FileKind) -> &'static str {
    match k {
        analyzer::FileKind::Text => "TEXT",
        analyzer::FileKind::Executable => "EXECUTABLE",
        analyzer::FileKind::Compressed => "COMPRESSED",
        analyzer::FileKind::Random => "RANDOM",
        analyzer::FileKind::Structured => "STRUCTURED",
        analyzer::FileKind::LossyMedia => "LOSSY_MEDIA",
        analyzer::FileKind::Unknown => "UNKNOWN",
    }
}

fn write_str(dst: &mut [u8], pos: usize, s: &str) -> usize {
    let b = s.as_bytes();
    let n = b.len().min(dst.len().saturating_sub(pos));
    dst[pos..pos + n].copy_from_slice(&b[..n]);
    n
}

fn write_u64(dst: &mut [u8], pos: usize, v: u64) -> usize {
    // Пишем u64 как ASCII цифры
    if pos >= dst.len() {
        return 0;
    }
    let mut buf = [0u8; 20];
    let mut n = 0usize;
    let mut x = v;
    if x == 0 {
        buf[0] = b'0';
        n = 1;
    } else {
        while x > 0 {
            buf[n] = b'0' + (x % 10) as u8;
            x /= 10;
            n += 1;
        }
        // Разворачиваем
        buf[..n].reverse();
    }
    let copy = n.min(dst.len() - pos);
    dst[pos..pos + copy].copy_from_slice(&buf[..copy]);
    copy
}
