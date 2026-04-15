// ============================================================
// VGA UNICODE — единственное место, которое знает как превратить
// Unicode codepoint в VGA-байт.
// ============================================================
// Слоты 0..127   — встроенный VGA-шрифт (ASCII, не трогаем)
// Слоты 128..191 — статические глифы кириллицы (загружаем при старте)
// Слоты 192..255 — динамический LRU-кеш для арабского, CJK и др.
// ============================================================

// --- VGA dimensions (runtime, not const) ---
fn vga_columns() -> usize { crate::vga_hw::get_columns() }
fn vga_rows() -> usize { crate::vga_hw::get_rows() }

// ============================================================
// КИРИЛЛИЦА — глифы хранятся в fonts::bios_font::CYRILLIC_GLYPHS
// ============================================================

/// Преобразует UTF-8 кириллический символ в наш VGA-код (128+).
/// Возвращает None если символ не кириллический.
pub fn cyrillic_to_vga(c: char) -> Option<u8> {
    match c {
        // Заглавные: А=128, Б=129, ... Я=159
        'А' => Some(128), 'Б' => Some(129), 'В' => Some(130), 'Г' => Some(131),
        'Д' => Some(132), 'Е' => Some(133), 'Ж' => Some(134), 'З' => Some(135),
        'И' => Some(136), 'Й' => Some(137), 'К' => Some(138), 'Л' => Some(139),
        'М' => Some(140), 'Н' => Some(141), 'О' => Some(142), 'П' => Some(143),
        'Р' => Some(144), 'С' => Some(145), 'Т' => Some(146), 'У' => Some(147),
        'Ф' => Some(148), 'Х' => Some(149), 'Ц' => Some(150), 'Ч' => Some(151),
        'Ш' => Some(152), 'Щ' => Some(153), 'Ъ' => Some(154), 'Ы' => Some(155),
        'Ь' => Some(156), 'Э' => Some(157), 'Ю' => Some(158), 'Я' => Some(159),
        // Строчные: а=160, б=161, ... я=191
        'а' => Some(160), 'б' => Some(161), 'в' => Some(162), 'г' => Some(163),
        'д' => Some(164), 'е' => Some(165), 'ж' => Some(166), 'з' => Some(167),
        'и' => Some(168), 'й' => Some(169), 'к' => Some(170), 'л' => Some(171),
        'м' => Some(172), 'н' => Some(173), 'о' => Some(174), 'п' => Some(175),
        'р' => Some(176), 'с' => Some(177), 'т' => Some(178), 'у' => Some(179),
        'ф' => Some(180), 'х' => Some(181), 'ц' => Some(182), 'ч' => Some(183),
        'ш' => Some(184), 'щ' => Some(185), 'ъ' => Some(186), 'ы' => Some(187),
        'ь' => Some(188), 'э' => Some(189), 'ю' => Some(190), 'я' => Some(191),
        // Ё/ё — маппим на Е/е
        'Ё' => Some(133), 'ё' => Some(165),
        _ => None,
    }
}

/// Загружает кириллический шрифт в VGA plane 2 (слоты 128..191).
/// Вызывать при старте до любого вывода кириллического текста.
///
/// # Safety
/// Переключает VGA в font mode (plane 2), пишет глифы в 0xA0000, восстанавливает
/// текстовый режим. Требует: VGA identity-mapped, вызывать вне активного font mode
/// (не реентрантно). Без прерываний во время выполнения.
pub unsafe fn load_static_glyphs() {
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
    0x00, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
    0x18, 0x18, 0x1C, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_ALEF_HAMZA_ABOVE: [u8; 16] = [
    0x0C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
    0x18, 0x18, 0x1C, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_ALEF_HAMZA_BELOW: [u8; 16] = [
    0x00, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
    0x18, 0x18, 0x1C, 0x0C, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_BA: [u8; 16] = [
    0x00, 0x00, 0x00, 0x00, 0x7E, 0x42, 0x40, 0x7E,
    0x00, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_TA_MARBUTA: [u8; 16] = [
    0x00, 0x24, 0x00, 0x3C, 0x42, 0x42, 0x3C, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_TA: [u8; 16] = [
    0x00, 0x24, 0x00, 0x7E, 0x42, 0x40, 0x7E, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_THA: [u8; 16] = [
    0x00, 0x2A, 0x00, 0x7E, 0x42, 0x40, 0x7E, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_JIM: [u8; 16] = [
    0x00, 0x00, 0x3E, 0x22, 0x22, 0x18, 0x22, 0x1E,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_HA: [u8; 16] = [
    0x00, 0x00, 0x3E, 0x22, 0x02, 0x02, 0x22, 0x1E,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_KHA: [u8; 16] = [
    0x18, 0x00, 0x3E, 0x22, 0x02, 0x02, 0x22, 0x1E,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_DAL: [u8; 16] = [
    0x00, 0x00, 0x3C, 0x24, 0x24, 0x24, 0x1E, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_DHAL: [u8; 16] = [
    0x18, 0x00, 0x3C, 0x24, 0x24, 0x24, 0x1E, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_RA: [u8; 16] = [
    0x00, 0x00, 0x30, 0x30, 0x18, 0x18, 0x0C, 0x04,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_ZAY: [u8; 16] = [
    0x18, 0x00, 0x30, 0x30, 0x18, 0x18, 0x0C, 0x04,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_SIN: [u8; 16] = [
    0x00, 0x00, 0x00, 0x6A, 0x02, 0x02, 0x7E, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_SHIN: [u8; 16] = [
    0x54, 0x00, 0x00, 0x6A, 0x02, 0x02, 0x7E, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_SAD: [u8; 16] = [
    0x00, 0x00, 0x3C, 0x42, 0x42, 0x42, 0x7E, 0x02,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_DAD: [u8; 16] = [
    0x18, 0x00, 0x3C, 0x42, 0x42, 0x42, 0x7E, 0x02,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_TA_EMPHATIC: [u8; 16] = [
    0x00, 0x00, 0x3E, 0x22, 0x22, 0x3E, 0x08, 0x08,
    0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_ZHA: [u8; 16] = [
    0x18, 0x00, 0x3E, 0x22, 0x22, 0x3E, 0x08, 0x08,
    0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_AYN: [u8; 16] = [
    0x00, 0x00, 0x3C, 0x42, 0x42, 0x3C, 0x08, 0x04,
    0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_GHAYN: [u8; 16] = [
    0x18, 0x00, 0x3C, 0x42, 0x42, 0x3C, 0x08, 0x04,
    0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_FA: [u8; 16] = [
    0x00, 0x18, 0x3C, 0x42, 0x42, 0x7E, 0x02, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_QAF: [u8; 16] = [
    0x00, 0x00, 0x3C, 0x42, 0x42, 0x7E, 0x00, 0x24,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_KAF: [u8; 16] = [
    0x00, 0x00, 0x7E, 0x04, 0x08, 0x0C, 0x08, 0x04,
    0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_LAM: [u8; 16] = [
    0x00, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
    0x0C, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_MIM: [u8; 16] = [
    0x00, 0x00, 0x3C, 0x42, 0x42, 0x3C, 0x10, 0x08,
    0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_NUN: [u8; 16] = [
    0x00, 0x18, 0x00, 0x3C, 0x02, 0x02, 0x3E, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_HA_SMALL: [u8; 16] = [
    0x00, 0x00, 0x3C, 0x42, 0x42, 0x3C, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_WAW: [u8; 16] = [
    0x00, 0x00, 0x1C, 0x22, 0x22, 0x1C, 0x08, 0x04,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_YA: [u8; 16] = [
    0x00, 0x00, 0x7E, 0x42, 0x40, 0x7C, 0x00, 0x24,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_ALEF_MAQSURA: [u8; 16] = [
    0x00, 0x00, 0x7E, 0x42, 0x40, 0x7C, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_ARABIC_COMMA: [u8; 16] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x10,
    0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_ARABIC_SEMICOLON: [u8; 16] = [
    0x00, 0x00, 0x18, 0x18, 0x00, 0x18, 0x18, 0x10,
    0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_ARABIC_QUESTION: [u8; 16] = [
    0x00, 0x3C, 0x02, 0x02, 0x04, 0x08, 0x10, 0x00,
    0x18, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_AR_ZERO: [u8; 16] = [
    0x00, 0x00, 0x00, 0x3C, 0x42, 0x42, 0x42, 0x42,
    0x42, 0x3C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_AR_ONE: [u8; 16] = [
    0x00, 0x00, 0x00, 0x18, 0x18, 0x18, 0x18, 0x18,
    0x18, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_AR_TWO: [u8; 16] = [
    0x00, 0x00, 0x00, 0x3C, 0x42, 0x02, 0x0C, 0x18,
    0x30, 0x7E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_AR_THREE: [u8; 16] = [
    0x00, 0x00, 0x00, 0x7E, 0x04, 0x08, 0x3C, 0x08,
    0x04, 0x7E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_AR_FOUR: [u8; 16] = [
    0x00, 0x00, 0x00, 0x42, 0x42, 0x7E, 0x04, 0x04,
    0x04, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_AR_FIVE: [u8; 16] = [
    0x00, 0x00, 0x18, 0x3C, 0x42, 0x42, 0x42, 0x42,
    0x3C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_AR_SIX: [u8; 16] = [
    0x00, 0x00, 0x00, 0x1C, 0x20, 0x40, 0x7C, 0x42,
    0x42, 0x3C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_AR_SEVEN: [u8; 16] = [
    0x00, 0x00, 0x00, 0x7E, 0x02, 0x04, 0x08, 0x10,
    0x20, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_AR_EIGHT: [u8; 16] = [
    0x00, 0x00, 0x00, 0x3C, 0x42, 0x42, 0x3C, 0x42,
    0x42, 0x3C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static GLYPH_AR_NINE: [u8; 16] = [
    0x00, 0x00, 0x00, 0x3C, 0x42, 0x42, 0x3E, 0x02,
    0x04, 0x38, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
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
    // 2. Кириллица — слоты 128-191, загружены load_static_glyphs() при старте
    if let Some(vga_code) = cyrillic_to_vga(char::from_u32(cp).unwrap_or('\0')) {
        return Some(vga_code);
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
const VGA_TEXT_ADDR: usize = 0xB8000;
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
        if SLOT_IS_SECOND_HALF[i] { continue; }
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
    if wide { allocate_wide_slot() } else { allocate_narrow_slot() }
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
        if SLOT_IS_SECOND_HALF[i] { continue; }
        let combined_age = if SLOT_WIDE[i] {
            SLOT_AGE[i] as u64
        } else {
            if SLOT_IS_SECOND_HALF[i + 1] { continue; }
            SLOT_AGE[i] as u64 + SLOT_AGE[i + 1] as u64
        };
        if combined_age < min_combined_age {
            min_combined_age = combined_age;
            min_idx = i;
        }
    }
    let was_wide = SLOT_WIDE[min_idx];
    evict_slot(min_idx);
    if !was_wide { evict_slot(min_idx + 1); }
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
            if SLOT_CODEPOINT[i] == codepoint
                && codepoint != 0
                && !SLOT_IS_SECOND_HALF[i]
            {
                AGE_COUNTER = AGE_COUNTER.wrapping_add(1);
                SLOT_AGE[i] = AGE_COUNTER;
                return Some((GLYPH_CACHE_START + i) as u8);
            }
        }

        let bitmap = get_any_glyph_bitmap(codepoint)?;
        let cache_idx = allocate_slot(false);

        crate::vga_hw::enter_font_mode();
        write_glyph_to_slot(GLYPH_CACHE_START + cache_idx, bitmap);
        crate::vga_hw::exit_font_mode();

        AGE_COUNTER = AGE_COUNTER.wrapping_add(1);
        SLOT_CODEPOINT[cache_idx]      = codepoint;
        SLOT_AGE[cache_idx]            = AGE_COUNTER;
        SLOT_WIDE[cache_idx]           = false;
        SLOT_IS_SECOND_HALF[cache_idx] = false;

        Some((GLYPH_CACHE_START + cache_idx) as u8)
    })
}

/// Dispatches glyph bitmap lookup across all registered font sources.
fn get_any_glyph_bitmap(cp: u32) -> Option<&'static [u8; 16]> {
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
    let byte = codepoint_to_vga_byte(codepoint).unwrap_or(b'?');
    unsafe { write_vga_cell(x, y, byte, color); }
    1
}

/// Write a single (character, attribute) pair into the VGA text buffer.
///
/// # Safety
/// offset = (y*80 + x)*2, при x<80 y<25 → offset < 8000 (< 4000 слов VGA).
/// write_volatile обязателен — 0xB8000 MMIO.
unsafe fn write_vga_cell(x: usize, y: usize, ch: u8, attr: u8) {
    let offset = (y * vga_columns() + x) * 2;
    let base = VGA_TEXT_ADDR as *mut u8;
    core::ptr::write_volatile(base.add(offset), ch);
    core::ptr::write_volatile(base.add(offset + 1), attr);
}
