// ============================================================
// VGA UNICODE — центральный диспетчер шрифтов и Unicode-рендеринга.
// ============================================================
// Framebuffer (linear pixel mode):
//   glyph_bitmap(cp) → прямой рендер из bitmap, без VGA-слотов.
//   ASCII, кириллица, арабский рендерятся нативно через bios_font.
//
// Legacy VGA slot model:
//   Слоты 0..127   — встроенный VGA-шрифт (ASCII)
//   Слоты 128..191 — статические глифы кириллицы (plane 2)
//   Слоты 192..255 — динамический LRU-кеш (арабский, CJK и др.)
// ============================================================

// --- Text grid dimensions ---
// Ordinary rendering is framebuffer-only. Keep the cell grid tied to the
// initialized framebuffer instead of legacy VGA text mode geometry.
fn vga_columns() -> usize {
    crate::fb_buffer::get_cols()
}
fn vga_rows() -> usize {
    crate::fb_buffer::get_rows()
}

// ============================================================
// КИРИЛЛИЦА — глифы хранятся в fonts::bios_font::CYRILLIC_GLYPHS
// ============================================================

/// Преобразует Unicode-символ кириллицы в индекс нашего bitmap-набора.
/// Возвращает индекс 0..63 либо None, если символ не покрыт таблицей.
fn cyrillic_glyph_index(c: char) -> Option<usize> {
    match c {
        // Заглавные: А=0, Б=1, ... Я=31
        'А' => Some(0),
        'Б' => Some(1),
        'В' => Some(2),
        'Г' => Some(3),
        'Д' => Some(4),
        'Е' => Some(5),
        'Ж' => Some(6),
        'З' => Some(7),
        'И' => Some(8),
        'Й' => Some(9),
        'К' => Some(10),
        'Л' => Some(11),
        'М' => Some(12),
        'Н' => Some(13),
        'О' => Some(14),
        'П' => Some(15),
        'Р' => Some(16),
        'С' => Some(17),
        'Т' => Some(18),
        'У' => Some(19),
        'Ф' => Some(20),
        'Х' => Some(21),
        'Ц' => Some(22),
        'Ч' => Some(23),
        'Ш' => Some(24),
        'Щ' => Some(25),
        'Ъ' => Some(26),
        'Ы' => Some(27),
        'Ь' => Some(28),
        'Э' => Some(29),
        'Ю' => Some(30),
        'Я' => Some(31),
        // Строчные: а=32, б=33, ... я=63
        'а' => Some(32),
        'б' => Some(33),
        'в' => Some(34),
        'г' => Some(35),
        'д' => Some(36),
        'е' => Some(37),
        'ж' => Some(38),
        'з' => Some(39),
        'и' => Some(40),
        'й' => Some(41),
        'к' => Some(42),
        'л' => Some(43),
        'м' => Some(44),
        'н' => Some(45),
        'о' => Some(46),
        'п' => Some(47),
        'р' => Some(48),
        'с' => Some(49),
        'т' => Some(50),
        'у' => Some(51),
        'ф' => Some(52),
        'х' => Some(53),
        'ц' => Some(54),
        'ч' => Some(55),
        'ш' => Some(56),
        'щ' => Some(57),
        'ъ' => Some(58),
        'ы' => Some(59),
        'ь' => Some(60),
        'э' => Some(61),
        'ю' => Some(62),
        'я' => Some(63),
        // Ё/ё — пока используем форму Е/е, но остаёмся в Unicode-модели.
        'Ё' => Some(5),
        'ё' => Some(37),
        _ => None,
    }
}

/// Возвращает 8x16 bitmap для codepoint, если у нас есть прямой Unicode-глиф.
pub fn glyph_bitmap(cp: u32) -> Option<&'static [u8; 16]> {
    if cp < 128 {
        return Some(&crate::fonts::bios_font::VGA_FONT_8X16[cp as usize]);
    }
    if let Some(idx) = cyrillic_glyph_index(char::from_u32(cp).unwrap_or('\0')) {
        if idx < crate::fonts::bios_font::CYRILLIC_GLYPHS.len() {
            return Some(&crate::fonts::bios_font::CYRILLIC_GLYPHS[idx]);
        }
    }
    get_any_glyph_bitmap(cp)
}

/// Возвращает runtime-глиф для framebuffer-пути из единого внешнего Unicode-font.
/// Это отделено от bios_font.rs, который остаётся legacy/error-источником.
fn runtime_unifont_bitmap(cp: u32) -> Option<[u8; 16]> {
    let ch = char::from_u32(cp)?;
    let glyph = baremetal_unifont::get_glyph(ch)?;
    if glyph.width() != 8 {
        return None;
    }

    let mut rows = [0u8; 16];
    let mut y = 0usize;
    while y < 16 {
        let mut bits = 0u8;
        let mut x = 0usize;
        while x < 8 {
            if glyph.get(x, y) {
                bits |= 1 << (7 - x);
            }
            x += 1;
        }
        rows[y] = bits;
        y += 1;
    }

    Some(rows)
}

pub fn runtime_glyph_bitmap(cp: u32) -> Option<[u8; 16]> {
    if let Some(rows) = runtime_unifont_bitmap(cp) {
        return Some(rows);
    }

    get_any_glyph_bitmap(cp).copied()
}

static GLYPH_MIDDLE_DOT: [u8; 16] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// Загружает кириллический шрифт в VGA plane 2 (слоты 128..191).
/// Вызывать при старте до любого вывода кириллического текста.
///
/// # Safety
/// Переключает VGA в font mode (plane 2), пишет глифы в 0xA0000, восстанавливает
/// текстовый режим. Требует: VGA identity-mapped, вызывать вне активного font mode
/// (не реентрантно). Без прерываний во время выполнения.
pub unsafe fn load_static_glyphs() {
    // Если активен линейный framebuffer — рендерер использует CYRILLIC_GLYPHS напрямую
    // (get_glyph в fb_buffer.rs), VGA plane 2 ему не нужен. Запись в VGA sequencer
    // при активном VBE переключает дисплей обратно в legacy text path, из-за чего
    // весь вывод Phase 6/7 становится невидимым.
    if crate::fb_buffer::is_initialized() {
        crate::serial_println!(
            "[VGA] load_static_glyphs: framebuffer active, skipping VGA plane 2 writes"
        );
        return;
    }
    // RTX 3060 / современные GPU без legacy VGA: probe_vga() вернёт false (0xFF на 0x3DA).
    // Если VGA не отвечает — enter/exit_font_mode пишут в SEQ/GC регистры впустую или
    // вешают машину. Глифы в plane 2 всё равно недоступны — пропускаем.
    if !crate::validator::probe_vga() {
        crate::serial_println!(
            "[VGA] load_static_glyphs: no VGA controller, skipping font plane writes"
        );
        return;
    }
    x86_64::instructions::interrupts::without_interrupts(|| {
        // SAFETY: vga_hw::enter/exit_font_mode управляют переключением VGA plane 2.
        // 0xA0000 — font plane, identity-mapped. char_offset = (128+i)*32 < 192*32 = 6144
        // что меньше 64KB VGA plane size. write_volatile обязателен для MMIO.
        crate::vga_hw::enter_font_mode();

        let font_base = 0xA0000 as *mut u8;

        for (i, glyph) in crate::fonts::bios_font::CYRILLIC_GLYPHS.iter().enumerate() {
            let char_offset = (128 + i) * 32;
            for (row, &byte) in glyph.iter().enumerate() {
                core::ptr::write_volatile(font_base.add(char_offset + row), byte);
            }
            for row in 16..32 {
                core::ptr::write_volatile(font_base.add(char_offset + row), 0);
            }
        }

        crate::vga_hw::exit_font_mode();
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnicodeRuntimeInitMode {
    FramebufferDirect,
    LegacyVgaLoaded,
    LegacyVgaUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnicodeRuntimeInitReport {
    pub mode: UnicodeRuntimeInitMode,
    pub ready: usize,
    pub checked: usize,
    pub cyrillic_ready: bool,
    pub arabic_ready: bool,
}

const FRAMEBUFFER_WARMUP_GLYPHS: [char; 11] =
    ['Я', 'я', 'Ё', 'ё', 'ش', 'ل', 'ع', 'ة', '؟', '٠', '١'];

pub fn init_runtime_after_shell() -> UnicodeRuntimeInitReport {
    if crate::fb_buffer::is_initialized() {
        let mut ready = 0usize;
        for ch in FRAMEBUFFER_WARMUP_GLYPHS {
            if glyph_bitmap(ch as u32).is_some() {
                ready += 1;
            }
        }

        let report = UnicodeRuntimeInitReport {
            mode: UnicodeRuntimeInitMode::FramebufferDirect,
            ready,
            checked: FRAMEBUFFER_WARMUP_GLYPHS.len(),
            cyrillic_ready: glyph_bitmap('Я' as u32).is_some()
                && glyph_bitmap('ё' as u32).is_some(),
            arabic_ready: glyph_bitmap('ش' as u32).is_some()
                && glyph_bitmap('ة' as u32).is_some()
                && glyph_bitmap('؟' as u32).is_some(),
        };

        crate::serial_println!(
            "[UNICODE] runtime init after shell: mode=framebuffer ready={}/{} cyr={} ar={}",
            report.ready,
            report.checked,
            if report.cyrillic_ready { 1 } else { 0 },
            if report.arabic_ready { 1 } else { 0 }
        );
        return report;
    }

    if !crate::validator::probe_vga() {
        let report = UnicodeRuntimeInitReport {
            mode: UnicodeRuntimeInitMode::LegacyVgaUnavailable,
            ready: 0,
            checked: 0,
            cyrillic_ready: false,
            arabic_ready: false,
        };
        crate::serial_println!("[UNICODE] runtime init after shell: mode=legacy-vga-unavailable");
        return report;
    }

    unsafe { load_static_glyphs() };
    let report = UnicodeRuntimeInitReport {
        mode: UnicodeRuntimeInitMode::LegacyVgaLoaded,
        ready: crate::fonts::bios_font::CYRILLIC_GLYPHS.len(),
        checked: crate::fonts::bios_font::CYRILLIC_GLYPHS.len(),
        cyrillic_ready: true,
        arabic_ready: false,
    };
    crate::serial_println!(
        "[UNICODE] runtime init after shell: mode=legacy-vga loaded={} cyr=1 ar=0",
        report.ready
    );
    report
}

// ============================================================
// СТАТИЧЕСКИЕ ГЛИФЫ — АРАБСКИЙ (U+0600–U+06FF)
// ============================================================
// Isolated forms 8x16. RTL-укладка — в locale::print_rtl.
// Каждый глиф = 16 байт. Бит 7 = левый пиксель.
// ============================================================

fn arabic_get_glyph(cp: u32) -> Option<&'static [u8; 16]> {
    match cp {
        // ==================== ARABIC LETTERS ====================
        0x0623 => Some(&GLYPH_ALEF_HAMZA_ABOVE),
        0x0625 => Some(&GLYPH_ALEF_HAMZA_BELOW),
        0x0627 => Some(&GLYPH_ALEF),
        0x0628 => Some(&GLYPH_BA),
        0x0629 => Some(&GLYPH_TA_MARBUTA),
        0x062A => Some(&GLYPH_TA),
        0x062B => Some(&GLYPH_THA),
        0x062C => Some(&GLYPH_JIM),
        0x062D => Some(&GLYPH_HA),
        0x062E => Some(&GLYPH_KHA),
        0x062F => Some(&GLYPH_DAL),
        0x0630 => Some(&GLYPH_DHAL),
        0x0631 => Some(&GLYPH_RA),
        0x0632 => Some(&GLYPH_ZAY),
        0x0633 => Some(&GLYPH_SIN),
        0x0634 => Some(&GLYPH_SHIN),
        0x0635 => Some(&GLYPH_SAD),
        0x0636 => Some(&GLYPH_DAD),
        0x0637 => Some(&GLYPH_TA_EMPHATIC),
        0x0638 => Some(&GLYPH_ZHA),
        0x0639 => Some(&GLYPH_AYN),
        0x063A => Some(&GLYPH_GHAYN),
        0x0641 => Some(&GLYPH_FA),
        0x0642 => Some(&GLYPH_QAF),
        0x0643 => Some(&GLYPH_KAF),
        0x0644 => Some(&GLYPH_LAM),
        0x0645 => Some(&GLYPH_MIM),
        0x0646 => Some(&GLYPH_NUN),
        0x0647 => Some(&GLYPH_HA_SMALL),
        0x0648 => Some(&GLYPH_WAW),
        0x064A => Some(&GLYPH_YA),
        0x0649 => Some(&GLYPH_ALEF_MAQSURA),
        // ==================== ARABIC PUNCTUATION ====================
        0x060C => Some(&GLYPH_ARABIC_COMMA),
        0x061B => Some(&GLYPH_ARABIC_SEMICOLON),
        0x061F => Some(&GLYPH_ARABIC_QUESTION),
        // ==================== ARABIC-INDIC DIGITS ====================
        0x0660 => Some(&GLYPH_AR_ZERO),
        0x0661 => Some(&GLYPH_AR_ONE),
        0x0662 => Some(&GLYPH_AR_TWO),
        0x0663 => Some(&GLYPH_AR_THREE),
        0x0664 => Some(&GLYPH_AR_FOUR),
        0x0665 => Some(&GLYPH_AR_FIVE),
        0x0666 => Some(&GLYPH_AR_SIX),
        0x0667 => Some(&GLYPH_AR_SEVEN),
        0x0668 => Some(&GLYPH_AR_EIGHT),
        0x0669 => Some(&GLYPH_AR_NINE),
        _ => None,
    }
}

static GLYPH_ALEF: [u8; 16] = [
    0x00, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x1C, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_ALEF_HAMZA_ABOVE: [u8; 16] = [
    0x0C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x1C, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_ALEF_HAMZA_BELOW: [u8; 16] = [
    0x00, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x1C, 0x0C, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_BA: [u8; 16] = [
    0x00, 0x00, 0x00, 0x00, 0x7E, 0x42, 0x40, 0x7E, 0x00, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_TA_MARBUTA: [u8; 16] = [
    0x00, 0x24, 0x00, 0x3C, 0x42, 0x42, 0x3C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_TA: [u8; 16] = [
    0x00, 0x24, 0x00, 0x7E, 0x42, 0x40, 0x7E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_THA: [u8; 16] = [
    0x00, 0x2A, 0x00, 0x7E, 0x42, 0x40, 0x7E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_JIM: [u8; 16] = [
    0x00, 0x00, 0x3E, 0x22, 0x22, 0x18, 0x22, 0x1E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_HA: [u8; 16] = [
    0x00, 0x00, 0x3E, 0x22, 0x02, 0x02, 0x22, 0x1E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_KHA: [u8; 16] = [
    0x18, 0x00, 0x3E, 0x22, 0x02, 0x02, 0x22, 0x1E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_DAL: [u8; 16] = [
    0x00, 0x00, 0x3C, 0x24, 0x24, 0x24, 0x1E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_DHAL: [u8; 16] = [
    0x18, 0x00, 0x3C, 0x24, 0x24, 0x24, 0x1E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_RA: [u8; 16] = [
    0x00, 0x00, 0x30, 0x30, 0x18, 0x18, 0x0C, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_ZAY: [u8; 16] = [
    0x18, 0x00, 0x30, 0x30, 0x18, 0x18, 0x0C, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_SIN: [u8; 16] = [
    0x00, 0x00, 0x00, 0x6A, 0x02, 0x02, 0x7E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_SHIN: [u8; 16] = [
    0x54, 0x00, 0x00, 0x6A, 0x02, 0x02, 0x7E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_SAD: [u8; 16] = [
    0x00, 0x00, 0x3C, 0x42, 0x42, 0x42, 0x7E, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_DAD: [u8; 16] = [
    0x18, 0x00, 0x3C, 0x42, 0x42, 0x42, 0x7E, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_TA_EMPHATIC: [u8; 16] = [
    0x00, 0x00, 0x3E, 0x22, 0x22, 0x3E, 0x08, 0x08, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_ZHA: [u8; 16] = [
    0x18, 0x00, 0x3E, 0x22, 0x22, 0x3E, 0x08, 0x08, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_AYN: [u8; 16] = [
    0x00, 0x00, 0x3C, 0x42, 0x42, 0x3C, 0x08, 0x04, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_GHAYN: [u8; 16] = [
    0x18, 0x00, 0x3C, 0x42, 0x42, 0x3C, 0x08, 0x04, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_FA: [u8; 16] = [
    0x00, 0x18, 0x3C, 0x42, 0x42, 0x7E, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_QAF: [u8; 16] = [
    0x00, 0x00, 0x3C, 0x42, 0x42, 0x7E, 0x00, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_KAF: [u8; 16] = [
    0x00, 0x00, 0x7E, 0x04, 0x08, 0x0C, 0x08, 0x04, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_LAM: [u8; 16] = [
    0x00, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x0C, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_MIM: [u8; 16] = [
    0x00, 0x00, 0x3C, 0x42, 0x42, 0x3C, 0x10, 0x08, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_NUN: [u8; 16] = [
    0x00, 0x18, 0x00, 0x3C, 0x02, 0x02, 0x3E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_HA_SMALL: [u8; 16] = [
    0x00, 0x00, 0x3C, 0x42, 0x42, 0x3C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_WAW: [u8; 16] = [
    0x00, 0x00, 0x1C, 0x22, 0x22, 0x1C, 0x08, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_YA: [u8; 16] = [
    0x00, 0x00, 0x7E, 0x42, 0x40, 0x7C, 0x00, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_ALEF_MAQSURA: [u8; 16] = [
    0x00, 0x00, 0x7E, 0x42, 0x40, 0x7C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_ARABIC_COMMA: [u8; 16] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x10, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_ARABIC_SEMICOLON: [u8; 16] = [
    0x00, 0x00, 0x18, 0x18, 0x00, 0x18, 0x18, 0x10, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_ARABIC_QUESTION: [u8; 16] = [
    0x00, 0x3C, 0x02, 0x02, 0x04, 0x08, 0x10, 0x00, 0x18, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_AR_ZERO: [u8; 16] = [
    0x00, 0x00, 0x00, 0x3C, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_AR_ONE: [u8; 16] = [
    0x00, 0x00, 0x00, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_AR_TWO: [u8; 16] = [
    0x00, 0x00, 0x00, 0x3C, 0x42, 0x02, 0x0C, 0x18, 0x30, 0x7E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_AR_THREE: [u8; 16] = [
    0x00, 0x00, 0x00, 0x7E, 0x04, 0x08, 0x3C, 0x08, 0x04, 0x7E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_AR_FOUR: [u8; 16] = [
    0x00, 0x00, 0x00, 0x42, 0x42, 0x7E, 0x04, 0x04, 0x04, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_AR_FIVE: [u8; 16] = [
    0x00, 0x00, 0x18, 0x3C, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_AR_SIX: [u8; 16] = [
    0x00, 0x00, 0x00, 0x1C, 0x20, 0x40, 0x7C, 0x42, 0x42, 0x3C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_AR_SEVEN: [u8; 16] = [
    0x00, 0x00, 0x00, 0x7E, 0x02, 0x04, 0x08, 0x10, 0x20, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_AR_EIGHT: [u8; 16] = [
    0x00, 0x00, 0x00, 0x3C, 0x42, 0x42, 0x3C, 0x42, 0x42, 0x3C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_AR_NINE: [u8; 16] = [
    0x00, 0x00, 0x00, 0x3C, 0x42, 0x42, 0x3E, 0x02, 0x04, 0x38, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

// ============================================================
// ЕДИНЫЙ ДИСПЕТЧЕР: Unicode codepoint → VGA byte
// ============================================================
// Единственное место в проекте где решается "как показать символ".
// ASCII (0..127)    → встроенный VGA шрифт (слоты 0-127)
// Кириллица         → статические слоты 128-191
// Всё остальное     → LRU кеш слоты 192-255
// ============================================================

/// Возвращает VGA-байт для любого Unicode codepoint.
/// `None` = глиф недоступен (вызывающий пишет b'?').
pub fn codepoint_to_vga_byte(cp: u32) -> Option<u8> {
    // 1. ASCII — слоты 0-127, встроенный VGA шрифт, без обращения к plane 2
    if cp < 128 {
        return Some(cp as u8);
    }
    // 1.5. CP437-совместимые математические символы из VGA ROM.
    match cp {
        0x03C0 => return Some(0xE3),
        0x221E => return Some(0xEC),
        0x2550 => return Some(0xCD),
        0x2551 => return Some(0xBA),
        0x2554 => return Some(0xC9),
        0x2557 => return Some(0xBB),
        0x255A => return Some(0xC8),
        0x255D => return Some(0xBC),
        0x2560 => return Some(0xCC),
        0x2563 => return Some(0xB9),
        0x2588 => return Some(0xDB),
        0x2584 => return Some(0xDC),
        0x2580 => return Some(0xDF),
        0x2591 => return Some(0xB0),
        0x2592 => return Some(0xB1),
        0x2593 => return Some(0xB2),
        _ => {}
    }
    // 2. Кириллица — только здесь переводим Unicode-индекс в legacy VGA slot.
    if let Some(idx) = cyrillic_glyph_index(char::from_u32(cp).unwrap_or('\0')) {
        return Some((128 + idx) as u8);
    }
    // 3. Динамический кеш — слоты 192-255, LRU eviction
    ensure_glyph_cached(cp)
}

// ============================================================
// ДИНАМИЧЕСКИЙ КЕШ (слоты 192-255)
// ============================================================

const GLYPH_CACHE_START: usize = 192;
const GLYPH_CACHE_END: usize = 256;
const GLYPH_CACHE_SIZE: usize = GLYPH_CACHE_END - GLYPH_CACHE_START;

const FONT_PLANE_ADDR: usize = 0xA0000;
const BYTES_PER_GLYPH: usize = 32;

static mut SLOT_CODEPOINT: [u32; GLYPH_CACHE_SIZE] = [0; GLYPH_CACHE_SIZE];
static mut SLOT_AGE: [u32; GLYPH_CACHE_SIZE] = [0; GLYPH_CACHE_SIZE];
static mut AGE_COUNTER: u32 = 0;
static mut SLOT_WIDE: [bool; GLYPH_CACHE_SIZE] = [false; GLYPH_CACHE_SIZE];
static mut SLOT_IS_SECOND_HALF: [bool; GLYPH_CACHE_SIZE] = [false; GLYPH_CACHE_SIZE];

/// Write a 16-row bitmap into the VGA font plane at the given slot index.
///
/// # Safety
/// Требует: enter_font_mode() уже вызван (0xA0000 → plane 2).
/// offset = slot * 32 — слот 192..255 → offset 6144..8160.
/// write_volatile обязателен — MMIO с аппаратными побочными эффектами.
unsafe fn write_glyph_to_slot(slot: usize, bitmap: &[u8; 16]) {
    let base = FONT_PLANE_ADDR as *mut u8;
    let offset = slot * BYTES_PER_GLYPH;
    for (row, &byte) in bitmap.iter().enumerate() {
        core::ptr::write_volatile(base.add(offset + row), byte);
    }
    for row in 16..BYTES_PER_GLYPH {
        core::ptr::write_volatile(base.add(offset + row), 0);
    }
}

/// Find the cache index with the smallest age (least recently used),
/// skipping second-half slots of wide glyphs.
unsafe fn find_lru_slot() -> usize {
    let mut min_age = u32::MAX;
    let mut min_idx = 0;
    for i in 0..GLYPH_CACHE_SIZE {
        if SLOT_IS_SECOND_HALF[i] {
            continue;
        }
        if SLOT_AGE[i] < min_age {
            min_age = SLOT_AGE[i];
            min_idx = i;
        }
    }
    min_idx
}

unsafe fn reset_slot_metadata(idx: usize) {
    SLOT_CODEPOINT[idx] = 0;
    SLOT_AGE[idx] = 0;
    SLOT_IS_SECOND_HALF[idx] = false;
}

unsafe fn evict_slot(cache_idx: usize) {
    if SLOT_WIDE[cache_idx] && cache_idx + 1 < GLYPH_CACHE_SIZE {
        reset_slot_metadata(cache_idx + 1);
    }
    reset_slot_metadata(cache_idx);
    SLOT_WIDE[cache_idx] = false;
}

unsafe fn allocate_slot(wide: bool) -> usize {
    if wide {
        allocate_wide_slot()
    } else {
        allocate_narrow_slot()
    }
}

unsafe fn allocate_narrow_slot() -> usize {
    for i in 0..GLYPH_CACHE_SIZE {
        if SLOT_CODEPOINT[i] == 0 && !SLOT_IS_SECOND_HALF[i] {
            return i;
        }
    }
    let idx = find_lru_slot();
    evict_slot(idx);
    idx
}

unsafe fn allocate_wide_slot() -> usize {
    for i in 0..GLYPH_CACHE_SIZE - 1 {
        if SLOT_CODEPOINT[i] == 0
            && SLOT_CODEPOINT[i + 1] == 0
            && !SLOT_IS_SECOND_HALF[i]
            && !SLOT_IS_SECOND_HALF[i + 1]
        {
            return i;
        }
    }
    let mut min_combined_age = u64::MAX;
    let mut min_idx = 0;
    for i in 0..GLYPH_CACHE_SIZE - 1 {
        if SLOT_IS_SECOND_HALF[i] {
            continue;
        }
        let combined_age = if SLOT_WIDE[i] {
            SLOT_AGE[i] as u64
        } else {
            if SLOT_IS_SECOND_HALF[i + 1] {
                continue;
            }
            SLOT_AGE[i] as u64 + SLOT_AGE[i + 1] as u64
        };
        if combined_age < min_combined_age {
            min_combined_age = combined_age;
            min_idx = i;
        }
    }
    let was_wide = SLOT_WIDE[min_idx];
    evict_slot(min_idx);
    if !was_wide {
        evict_slot(min_idx + 1);
    }
    min_idx
}

/// Looks up or loads a glyph for `codepoint` into the VGA font cache.
/// Returns `Some(slot_byte)` where slot_byte is a VGA character index 192..=255,
/// or `None` if no bitmap is available for this codepoint.
pub fn ensure_glyph_cached(codepoint: u32) -> Option<u8> {
    // SAFETY: Обращается ко всем static mut массивам кеша (SLOT_*).
    // Весь блок выполняется под without_interrupts, чтобы enter/exit_font_mode
    // не был прерван ISR, пишущим в VGA (что повредило бы Plane 2 глифы).
    x86_64::instructions::interrupts::without_interrupts(|| unsafe {
        for i in 0..GLYPH_CACHE_SIZE {
            if SLOT_CODEPOINT[i] == codepoint && codepoint != 0 && !SLOT_IS_SECOND_HALF[i] {
                AGE_COUNTER = AGE_COUNTER.wrapping_add(1);
                SLOT_AGE[i] = AGE_COUNTER;
                return Some((GLYPH_CACHE_START + i) as u8);
            }
        }

        let bitmap = get_any_glyph_bitmap(codepoint)?;
        let cache_idx = allocate_slot(false);

        // RTX 3060: если VGA не отвечает — не трогаем font-mode регистры.
        if !crate::validator::probe_vga() {
            return None;
        }
        crate::vga_hw::enter_font_mode();
        write_glyph_to_slot(GLYPH_CACHE_START + cache_idx, bitmap);
        crate::vga_hw::exit_font_mode();

        AGE_COUNTER = AGE_COUNTER.wrapping_add(1);
        SLOT_CODEPOINT[cache_idx] = codepoint;
        SLOT_AGE[cache_idx] = AGE_COUNTER;
        SLOT_WIDE[cache_idx] = false;
        SLOT_IS_SECOND_HALF[cache_idx] = false;

        Some((GLYPH_CACHE_START + cache_idx) as u8)
    })
}

/// Dispatches glyph bitmap lookup across all registered font sources.
fn get_any_glyph_bitmap(cp: u32) -> Option<&'static [u8; 16]> {
    if cp == 0x00B7 {
        return Some(&GLYPH_MIDDLE_DOT);
    }
    if let Some(bmp) = arabic_get_glyph(cp) {
        return Some(bmp);
    }
    // Future: devanagari_get_glyph(cp), han_get_glyph(cp)
    None
}

// ============================================================
// VGA CELL WRITE & PUBLIC PRINT
// ============================================================

/// Write a Unicode codepoint to the VGA text buffer at (x, y).
/// Returns the number of columns consumed (0 if out of bounds).
pub fn print_char(codepoint: u32, x: usize, y: usize, color: u8) -> usize {
    if x >= vga_columns() || y >= vga_rows() {
        return 0;
    }
    crate::fb_buffer::write_codepoint_at(x, y, codepoint, color);
    1
}
