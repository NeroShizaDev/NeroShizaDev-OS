// ============================================================
// JACKAL ANALYZER — RAW bytes → structural hypothesis
// ============================================================
// Задача модуля: дать ядру (и VoodooEngine) «органы чувств» для
// байтовых данных. Analyzer НИЧЕГО не сжимает — он ищет признаки.
// Сжатие/кодирование — следующая фаза (encoder.rs).
//
// Пайплайн:
//   raw bytes
//     → magic sniff        (фиксированные сигнатуры: MZ, PE, PNG, ELF…)
//     → histogram 256      (частоты байтов)
//     → shannon H          (глобальная энтропия)
//     → block profile      (H по окнам 4 КБ — видно границы секций)
//     → top-k digrams      (периодичности 2-байт)
//     → autocorrelation    (периоды 1/2/4/8/16/32/64/128/256)
//     → classifier         (гипотеза: text/exe/compressed/random/structured)
//
// Интеграция с VoodooEngine: FileKind → входной вектор вероятностей
// для байесовского апдейта, автокорреляционный профиль → сигнал
// о том, какой AutomatonMode включить.
// ============================================================

#![allow(dead_code)]

use core::arch::asm;

// Количество байтовых классов (фиксировано: 256 возможных значений байта)
pub const ALPHABET: usize = 256;

// Размер блока для энтропийного профиля
pub const BLOCK_SIZE: usize = 4096;

// Максимум блоков в профиле (для файлов больше MAX_BLOCKS*4КБ =
// обрабатываются семплированием: берём BLOCK_SIZE из начала/середины/конца)
pub const MAX_BLOCKS: usize = 64;

// ============================================================
// FileKind — грубая классификация формата
// ============================================================
// Это «персонажи» для VoodooEngine. Каждый — гипотеза о природе
// данных, которую анализатор даёт на вход байесовскому движку.
// ============================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    /// ASCII/UTF-8 текст (низкая H, много 0x20-0x7E)
    Text,
    /// Исполняемый (PE/ELF/Mach-O): магия + смесь заголовка и кода
    Executable,
    /// Уже сжатое/архив (ZIP/GZ/XZ/7Z): H ≈ 8, без структуры
    Compressed,
    /// Шифрованное/истинно случайное: H ≈ 8, без периодов, без магии
    Random,
    /// Структурированный бинарный (BMP/WAV/TIFF/raw sensor log):
    /// периодичность есть, H средняя
    Structured,
    /// Картинка с lossy-компрессией (JPEG/MPEG): H высокая но
    /// есть локальные корреляции
    LossyMedia,
    /// Ещё не определено
    Unknown,
}

impl FileKind {
    pub fn short(self) -> &'static str {
        match self {
            FileKind::Text => "TEXT",
            FileKind::Executable => "EXEC",
            FileKind::Compressed => "COMP",
            FileKind::Random => "RAND",
            FileKind::Structured => "STRC",
            FileKind::LossyMedia => "MEDIA",
            FileKind::Unknown => "?",
        }
    }
}

// ============================================================
// MagicHit — детектированная сигнатура на offset
// ============================================================
#[derive(Debug, Clone, Copy)]
pub struct MagicHit {
    pub offset: usize,
    pub name: &'static str,
    pub kind_hint: FileKind,
}

/// Таблица магических байтов. Офсет = где искать, байты = что искать.
/// Порядок важен: первое совпадение выигрывает в «шапке», остальные
/// ищутся по всему образцу как вторичные маркеры.
static MAGIC_TABLE: &[(usize, &[u8], &str, FileKind)] = &[
    // PE / DOS executable
    (0x00, b"MZ", "DOS/PE header", FileKind::Executable),
    // ELF
    (0x00, &[0x7F, b'E', b'L', b'F'], "ELF", FileKind::Executable),
    // Mach-O (32 и 64 bit, little endian)
    (
        0x00,
        &[0xCE, 0xFA, 0xED, 0xFE],
        "Mach-O 32",
        FileKind::Executable,
    ),
    (
        0x00,
        &[0xCF, 0xFA, 0xED, 0xFE],
        "Mach-O 64",
        FileKind::Executable,
    ),
    // Archives / compressed
    (
        0x00,
        &[b'P', b'K', 0x03, 0x04],
        "ZIP/JAR/DOCX",
        FileKind::Compressed,
    ),
    (0x00, &[0x1F, 0x8B], "GZIP", FileKind::Compressed),
    (
        0x00,
        &[0xFD, b'7', b'z', b'X', b'Z'],
        "XZ",
        FileKind::Compressed,
    ),
    (
        0x00,
        &[b'7', b'z', 0xBC, 0xAF, 0x27, 0x1C],
        "7Z",
        FileKind::Compressed,
    ),
    (0x00, b"Rar!", "RAR", FileKind::Compressed),
    // Images
    (
        0x00,
        &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A],
        "PNG",
        FileKind::Structured,
    ),
    (0x00, &[0xFF, 0xD8, 0xFF], "JPEG", FileKind::LossyMedia),
    (0x00, b"GIF8", "GIF", FileKind::Structured),
    (0x00, b"BM", "BMP", FileKind::Structured),
    // Media containers
    (0x00, b"RIFF", "RIFF (WAV/AVI)", FileKind::Structured),
    (0x00, b"OggS", "OGG", FileKind::LossyMedia),
    (0x04, b"ftyp", "MP4/ISOBMFF", FileKind::LossyMedia),
    // PDF
    (0x00, b"%PDF", "PDF", FileKind::Structured),
    // UTF-8 BOM
    (0x00, &[0xEF, 0xBB, 0xBF], "UTF-8 BOM", FileKind::Text),
];

/// Ищет первое совпадение магии в шапке данных.
pub fn sniff_magic(data: &[u8]) -> Option<MagicHit> {
    for &(offset, bytes, name, kind) in MAGIC_TABLE {
        if data.len() < offset + bytes.len() {
            continue;
        }
        if &data[offset..offset + bytes.len()] == bytes {
            return Some(MagicHit {
                offset,
                name,
                kind_hint: kind,
            });
        }
    }
    None
}

// ============================================================
// Histogram — частоты байтов
// ============================================================
pub struct Histogram {
    pub counts: [u32; ALPHABET],
    pub total: u64,
}

impl Histogram {
    pub fn new() -> Self {
        Self {
            counts: [0; ALPHABET],
            total: 0,
        }
    }

    pub fn feed(&mut self, data: &[u8]) {
        for &b in data {
            self.counts[b as usize] += 1;
        }
        self.total += data.len() as u64;
    }

    /// Shannon entropy × 1000 (чтобы без float на выходе).
    /// Использует x87 FPU fyl2x — в духе всего ядра.
    pub fn shannon_milli(&self) -> u64 {
        if self.total == 0 {
            return 0;
        }
        let len = self.total;

        // H = log2(len) − (1/len) · Σ count·log2(count)
        let mut sum_clog: i64 = 0;
        for i in 0..ALPHABET {
            let c = self.counts[i];
            if c > 0 {
                sum_clog += unsafe { x87_c_log2c(c as u64) };
            }
        }
        let log2_len = unsafe { x87_log2_scaled(len, 1000) };
        let scaled = (sum_clog * 1000) / (len as i64);
        let h = log2_len - scaled;
        if h > 0 { h as u64 } else { 0 }
    }

    /// Топ-k байтов по частоте. Возвращает массив (byte, count).
    pub fn top_k<const K: usize>(&self) -> [(u8, u32); K] {
        let mut out = [(0u8, 0u32); K];
        // Наивный K·N; для K=16 и N=256 это 4096 операций — ок.
        for i in 0..ALPHABET {
            let c = self.counts[i];
            // Вставка в отсортированный список (убывание по count)
            let mut pos = K;
            for (j, slot) in out.iter().enumerate() {
                if c > slot.1 {
                    pos = j;
                    break;
                }
            }
            if pos < K {
                // сдвиг
                for j in (pos + 1..K).rev() {
                    out[j] = out[j - 1];
                }
                out[pos] = (i as u8, c);
            }
        }
        out
    }

    /// Сколько уникальных байтов встречено. Признак «алфавитности».
    /// Text обычно 70–90, random ~256, PE код 200+ но с перекосом.
    pub fn unique_count(&self) -> u32 {
        let mut n = 0u32;
        for &c in &self.counts {
            if c > 0 {
                n += 1;
            }
        }
        n
    }

    /// Доля «печатных» ASCII (0x20..0x7E + \t \n \r).
    /// × 1000 чтобы без float. Text > 950, код < 400, random ~370.
    pub fn printable_ratio_milli(&self) -> u32 {
        if self.total == 0 {
            return 0;
        }
        let mut printable: u64 = 0;
        for i in 0x20..=0x7E {
            printable += self.counts[i] as u64;
        }
        printable += self.counts[b'\t' as usize] as u64;
        printable += self.counts[b'\n' as usize] as u64;
        printable += self.counts[b'\r' as usize] as u64;
        ((printable * 1000) / self.total) as u32
    }

    /// Доля нулевых байтов × 1000. EXE/структурные файлы часто > 100.
    pub fn zero_ratio_milli(&self) -> u32 {
        if self.total == 0 {
            return 0;
        }
        ((self.counts[0] as u64 * 1000) / self.total) as u32
    }
}

// ============================================================
// BlockProfile — энтропия по окнам BLOCK_SIZE
// ============================================================
// Если файл — PE, то профиль покажет:
//   блок 0: низкая H (заголовок, много нулей)
//   блоки 1..N: средняя H (.text — машинный код)
//   блоки M..: низкая H (.rdata — строки)
//   блок X: высокая H (встроенный ресурс/сжатая секция)
// Скачки H = границы секций. Это и есть структурный сигнал.
// ============================================================
pub struct BlockProfile {
    pub milli: [u16; MAX_BLOCKS], // H × 1000, 0..8000
    pub block_count: usize,
    pub file_size: u64,
}

impl BlockProfile {
    pub fn build(data: &[u8]) -> Self {
        let mut prof = Self {
            milli: [0; MAX_BLOCKS],
            block_count: 0,
            file_size: data.len() as u64,
        };
        if data.is_empty() {
            return prof;
        }

        let total_blocks = (data.len() + BLOCK_SIZE - 1) / BLOCK_SIZE;

        if total_blocks <= MAX_BLOCKS {
            // Полный прогон
            for (i, chunk) in data.chunks(BLOCK_SIZE).enumerate() {
                let mut h = Histogram::new();
                h.feed(chunk);
                prof.milli[i] = h.shannon_milli().min(8000) as u16;
            }
            prof.block_count = total_blocks;
        } else {
            // Семплирование: равномерно по файлу
            let step = total_blocks / MAX_BLOCKS;
            for i in 0..MAX_BLOCKS {
                let offset = i * step * BLOCK_SIZE;
                let end = (offset + BLOCK_SIZE).min(data.len());
                if offset >= data.len() {
                    break;
                }
                let mut h = Histogram::new();
                h.feed(&data[offset..end]);
                prof.milli[i] = h.shannon_milli().min(8000) as u16;
            }
            prof.block_count = MAX_BLOCKS;
        }
        prof
    }

    /// Сколько «скачков» H между соседними блоками (порог 1500 ≈ 1.5 бита).
    /// Каждый скачок = вероятная граница секции.
    pub fn transition_count(&self) -> u32 {
        let mut n = 0u32;
        for i in 1..self.block_count {
            let a = self.milli[i - 1] as i32;
            let b = self.milli[i] as i32;
            if (a - b).abs() > 1500 {
                n += 1;
            }
        }
        n
    }

    /// Максимум и минимум H по блокам.
    pub fn range(&self) -> (u16, u16) {
        if self.block_count == 0 {
            return (0, 0);
        }
        let mut mn = u16::MAX;
        let mut mx = 0u16;
        for i in 0..self.block_count {
            let v = self.milli[i];
            if v < mn {
                mn = v;
            }
            if v > mx {
                mx = v;
            }
        }
        (mn, mx)
    }
}

// ============================================================
// Autocorrelation — поиск периодов
// ============================================================
// Для каждого периода p считаем долю позиций где data[i] == data[i+p].
// У структурных форматов выраженные пики на «родных» периодах:
//   BMP 24-bit: пик на 3
//   WAV 16-bit stereo: пик на 4
//   PE Rich header: пик на 8
//   таблицы указателей 64-bit: пик на 8
// Random и Compressed: плоско ≈ 1/256.
// ============================================================
pub const AUTOCORR_PERIODS: &[usize] = &[1, 2, 3, 4, 8, 16, 32, 64, 128, 256];

pub struct AutocorrProfile {
    /// Процент совпадений × 100 (то есть 0..10000, где 10000 = 100%).
    pub match_bp: [u16; 10],
    pub len: usize,
}

impl AutocorrProfile {
    pub fn build(data: &[u8]) -> Self {
        let mut out = Self {
            match_bp: [0; 10],
            len: AUTOCORR_PERIODS.len(),
        };
        if data.len() < 16 {
            return out;
        }

        // Семпл: до 64 КБ — полный, больше — семплируем
        let sample_len = data.len().min(64 * 1024);
        let sample = &data[..sample_len];

        for (i, &p) in AUTOCORR_PERIODS.iter().enumerate() {
            if sample.len() <= p {
                continue;
            }
            let mut hits: u64 = 0;
            let mut total: u64 = 0;
            for j in 0..(sample.len() - p) {
                if sample[j] == sample[j + p] {
                    hits += 1;
                }
                total += 1;
            }
            if total > 0 {
                out.match_bp[i] = ((hits * 10_000) / total).min(10_000) as u16;
            }
        }
        out
    }

    /// Возвращает индекс периода с наибольшей корреляцией.
    pub fn strongest(&self) -> (usize, u16) {
        let mut best_i = 0;
        let mut best_v = 0u16;
        for i in 0..self.len {
            if self.match_bp[i] > best_v {
                best_v = self.match_bp[i];
                best_i = i;
            }
        }
        (AUTOCORR_PERIODS[best_i], best_v)
    }
}

// ============================================================
// Report — то что отдаётся наверх (шеллу, VoodooEngine, encoder)
// ============================================================
pub struct Report {
    pub size: u64,
    pub magic: Option<MagicHit>,
    pub global_h_milli: u64,
    pub unique: u32,
    pub printable_bp: u32,
    pub zero_bp: u32,
    pub profile: BlockProfile,
    pub autocorr: AutocorrProfile,
    pub classification: FileKind,
    pub confidence_milli: u32, // 0..1000
}

/// Главная функция анализа.
pub fn analyze(data: &[u8]) -> Report {
    let magic = sniff_magic(data);

    let mut hist = Histogram::new();
    hist.feed(data);

    let h_milli = hist.shannon_milli();
    let unique = hist.unique_count();
    let printable = hist.printable_ratio_milli();
    let zero = hist.zero_ratio_milli();

    let profile = BlockProfile::build(data);
    let autocorr = AutocorrProfile::build(data);

    let (classification, confidence) = classify(
        &magic, h_milli, unique, printable, zero, &profile, &autocorr,
    );

    Report {
        size: data.len() as u64,
        magic,
        global_h_milli: h_milli,
        unique,
        printable_bp: printable,
        zero_bp: zero,
        profile,
        autocorr,
        classification,
        confidence_milli: confidence,
    }
}

// ============================================================
// Classifier — правила → FileKind + confidence
// ============================================================
// Это не нейросеть, это экспертная система.
// VoodooEngine потом может УЛУЧШАТЬ эти правила,
// но сначала должна быть базовая линия.
// ============================================================
fn classify(
    magic: &Option<MagicHit>,
    h_milli: u64,
    _unique: u32,
    printable_bp: u32,
    zero_bp: u32,
    profile: &BlockProfile,
    autocorr: &AutocorrProfile,
) -> (FileKind, u32) {
    // Магия — сильнейший сигнал, но не абсолют (файл может быть повреждён).
    if let Some(m) = magic {
        // Подтверждаем гипотезу статистикой
        let (mn, mx) = profile.range();
        let profile_span = mx.saturating_sub(mn);
        let transitions = profile.transition_count();

        let base_conf = match m.kind_hint {
            FileKind::Executable => {
                // Для EXE ждём: средняя H 5-7, широкий span, много переходов
                if h_milli > 4000 && profile_span > 1000 && transitions > 0 {
                    900
                } else {
                    700
                }
            }
            FileKind::Compressed => {
                // Для сжатого ждём: H > 7.5, плоский профиль
                if h_milli > 7500 && profile_span < 500 {
                    950
                } else {
                    700
                }
            }
            FileKind::Text => {
                if printable_bp > 900 {
                    900
                } else {
                    600
                }
            }
            _ => 800,
        };
        return (m.kind_hint, base_conf);
    }

    // Магии нет → идём по статистике.
    // Текст: много печатных, H < 5
    if printable_bp > 900 && h_milli < 5500 {
        return (FileKind::Text, 850);
    }

    // Высокая энтропия + плоский профиль + нет сильных периодов = сжатое/random
    let (mn, mx) = profile.range();
    let flat = mx.saturating_sub(mn) < 400;
    let (_best_period, best_corr) = autocorr.strongest();

    if h_milli > 7500 && flat {
        // Различаем compressed от random по автокорреляции на периоде 1
        // (compressed всё же имеет капельку структуры, random — нет)
        if best_corr < 420 {
            // ≈ 1/256 × 10000 + небольшой допуск
            return (FileKind::Random, 700);
        } else {
            return (FileKind::Compressed, 750);
        }
    }

    // Средняя H + выраженный период 2/3/4/8 + много нулей = структурный бинарник
    if h_milli > 3000 && h_milli < 7500 && best_corr > 1500 && zero_bp > 50 {
        return (FileKind::Structured, 700);
    }

    // Средне-высокая H + слабые локальные корреляции + нет магии = lossy media
    if h_milli > 7000 && h_milli < 7800 && best_corr > 500 && best_corr < 1500 {
        return (FileKind::LossyMedia, 600);
    }

    (FileKind::Unknown, 300)
}

// ============================================================
// VoodooEngine bridge — отдаёт вероятности под байесовский апдейт
// ============================================================
/// Конвертирует Report в стартовый вектор вероятностей для VoodooEngine.
/// Порядок соответствует enum FileKind (кроме Unknown).
/// Возвращает [P(Text), P(Executable), P(Compressed), P(Random),
///             P(Structured), P(LossyMedia)]
pub fn to_voodoo_priors(report: &Report) -> [f32; 6] {
    let conf = (report.confidence_milli as f32) / 1000.0;
    let base = (1.0 - conf) / 6.0;
    let mut priors = [base; 6];

    let idx = match report.classification {
        FileKind::Text => Some(0),
        FileKind::Executable => Some(1),
        FileKind::Compressed => Some(2),
        FileKind::Random => Some(3),
        FileKind::Structured => Some(4),
        FileKind::LossyMedia => Some(5),
        FileKind::Unknown => None,
    };
    if let Some(i) = idx {
        priors[i] = base + conf;
    } else {
        // Unknown → равномерно
        for p in &mut priors {
            *p = 1.0 / 6.0;
        }
    }
    priors
}

// ============================================================
// x87 FPU helpers (в стиле модуля fpu)
// ============================================================
/// count · log2(count), результат округлён до i64.
/// SAFETY: fld/fstp балансируют x87-стек; options(nostack) —
/// RSP не трогается. Вход > 0 (проверяется вызывающим).
#[inline(always)]
unsafe fn x87_c_log2c(count: u64) -> i64 {
    let c = count;
    let out: i64;
    unsafe {
        asm!(
            "fld1",                          // ST = 1.0
            "push {c}",
            "fild qword ptr [rsp]",          // ST = count, ST(1) = 1.0
            "add rsp, 8",
            "fyl2x",                         // ST = 1.0 · log2(count) = log2(count)
            "push {c}",
            "fild qword ptr [rsp]",          // ST = count, ST(1) = log2(count)
            "add rsp, 8",
            "fmulp",                         // ST = count · log2(count)
            "push 0",
            "fistp qword ptr [rsp]",
            "pop {out}",
            c = in(reg) c,
            out = out(reg) out,
        );
    }
    out
}

/// log2(n) · scale, округлённо. SAFETY: см. x87_c_log2c.
#[inline(always)]
unsafe fn x87_log2_scaled(n: u64, scale: u64) -> i64 {
    let out: i64;
    unsafe {
        asm!(
            "fld1",
            "push {n}",
            "fild qword ptr [rsp]",
            "add rsp, 8",
            "fyl2x",                         // log2(n)
            "push {k}",
            "fild qword ptr [rsp]",
            "add rsp, 8",
            "fmulp",                         // log2(n) · scale
            "push 0",
            "fistp qword ptr [rsp]",
            "pop {out}",
            n = in(reg) n,
            k = in(reg) scale,
            out = out(reg) out,
        );
    }
    out
}
