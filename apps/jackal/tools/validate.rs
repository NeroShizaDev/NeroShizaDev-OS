// ============================================================
// VALIDATE — Rust-версия validate.py
// ============================================================
// Python-оригинал: apps/jackal/scripts/validate.py
// Проверяет что файл/буфер является валидным .jkl или .jkla.
// ============================================================

#![allow(dead_code)]

use crate::apps::jackal::encoder::{JKL_MAGIC, JKL_HEADER_SIZE};
use super::archive::ARCH_MAGIC;

/// Результат валидации
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidateResult {
    /// Валидный .jkl одиночный блок
    JklBlock,
    /// Валидный .jkla многоблочный архив
    JklaArchive,
    /// Плохая магия / слишком короткий
    BadMagic,
    /// Размер payload не совпадает с заголовком
    SizeMismatch,
    /// Неизвестный алгоритм
    UnknownAlgorithm,
}

impl ValidateResult {
    pub fn is_ok(self) -> bool {
        matches!(self, ValidateResult::JklBlock | ValidateResult::JklaArchive)
    }

    pub fn description(self) -> &'static str {
        match self {
            ValidateResult::JklBlock => "valid JKL block",
            ValidateResult::JklaArchive => "valid JKLA archive",
            ValidateResult::BadMagic => "bad magic / too short",
            ValidateResult::SizeMismatch => "size mismatch",
            ValidateResult::UnknownAlgorithm => "unknown algorithm",
        }
    }
}

/// Аналог `python validate.py doc.jkl` — проверяет буфер.
pub fn validate(data: &[u8]) -> ValidateResult {
    if data.len() < 5 {
        return ValidateResult::BadMagic;
    }

    // Проверяем .jkla (многоблочный архив)
    if &data[0..5] == ARCH_MAGIC {
        return validate_jkla(data);
    }

    // Проверяем .jkl (одиночный блок)
    if data.len() >= 4 && &data[0..4] == &JKL_MAGIC {
        return validate_jkl(data);
    }

    ValidateResult::BadMagic
}

fn validate_jkl(data: &[u8]) -> ValidateResult {
    if data.len() < JKL_HEADER_SIZE {
        return ValidateResult::BadMagic;
    }

    let algorithm = data[5];
    if algorithm > 2 {
        return ValidateResult::UnknownAlgorithm;
    }

    let enc_sz = u64::from_le_bytes([
        data[14], data[15], data[16], data[17],
        data[18], data[19], data[20], data[21],
    ]) as usize;

    if JKL_HEADER_SIZE + enc_sz != data.len() {
        return ValidateResult::SizeMismatch;
    }

    ValidateResult::JklBlock
}

fn validate_jkla(data: &[u8]) -> ValidateResult {
    match super::archive::unpack(data) {
        Ok(_) => ValidateResult::JklaArchive,
        Err("unknown algorithm") => ValidateResult::UnknownAlgorithm,
        Err("bad magic") | Err("too short") => ValidateResult::BadMagic,
        Err(_) => ValidateResult::SizeMismatch,
    }
}

