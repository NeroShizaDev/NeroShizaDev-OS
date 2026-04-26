// ============================================================
// JACKAL TOOLS — Rust-версии Python-скриптов из scripts/
// ============================================================
//
// Соответствие Python → Rust:
//   extract_clean_text.py  → extractor.rs   (чистка текста от мусора)
//   pack.py / unpack.py    → archive.rs      (ZIP-like упаковка .jkl блоков)
//   pipeline.py            → pipeline.rs     (пайплайн: анализ → кодирование → .jkl)
//   validate.py            → validate.rs     (проверка .jkl заголовка)
//   replace_text.py        → replace.rs      (поиск и замена в байтовом срезе)
//
// В ядре нет файловой системы, поэтому все функции работают с &[u8] и &mut [u8].
// ============================================================

pub mod archive;
pub mod extractor;
pub mod pipeline;
pub mod replace;
pub mod validate;
