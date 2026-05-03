// ============================================================
// LOCALE — NeroShizaDev Multilingual Render Pipeline
// ============================================================
// Слои пайплайна:
//   KernelEvent
//     -> get_event_text(locale, ev)        [kernel_messages]
//     -> numeral substitution               [locale::map_digit]
//     -> text direction (LTR / RTL)
//     -> glyph rendering                    [fb_buffer]
//     -> crash-only VGA helpers             [panic/exception paths]
// ============================================================

use core::fmt::{self, Write};

use crate::kernel_messages::{
    KernelEvent, Locale, MessageMode, get_event_text, locale_name, mode_name,
};

// ============================================================
// LOCALE SPEC — параметры системы письма
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
    Ltr, // Left-to-Right: RU, EN
    Rtl, // Right-to-Left: AR, FA, HE
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumeralSystem {
    Western,     // 0 1 2 3 4 5 6 7 8 9
    ArabicIndic, // ٠ ١ ٢ ٣ ٤ ٥ ٦ ٧ ٨ ٩  (U+0660-U+0669)
}

pub struct LocaleSpec {
    pub direction: TextDirection,
    pub numerals: NumeralSystem,
    pub name: &'static str,
}

pub fn locale_spec(locale: Locale) -> LocaleSpec {
    match locale {
        Locale::RuRu => LocaleSpec {
            direction: TextDirection::Ltr,
            numerals: NumeralSystem::Western,
            name: "RU",
        },
        Locale::EnUs => LocaleSpec {
            direction: TextDirection::Ltr,
            numerals: NumeralSystem::Western,
            name: "EN",
        },
        Locale::ArEg => LocaleSpec {
            direction: TextDirection::Rtl,
            numerals: NumeralSystem::ArabicIndic,
            name: "AR",
        },
    }
}

// ============================================================
// NUMERAL MAPPING
// ============================================================

/// Преобразует ASCII цифру в нужную систему счисления.
pub fn map_digit_char(c: char, sys: NumeralSystem) -> char {
    match sys {
        NumeralSystem::Western => c,
        NumeralSystem::ArabicIndic => match c {
            '0' => '٠',
            '1' => '١',
            '2' => '٢',
            '3' => '٣',
            '4' => '٤',
            '5' => '٥',
            '6' => '٦',
            '7' => '٧',
            '8' => '٨',
            '9' => '٩',
            _ => c,
        },
    }
}

fn map_display_char(c: char, sys: NumeralSystem) -> char {
    map_digit_char(c, sys)
}

pub fn map_shell_input_char(c: char, russian_layout: bool) -> char {
    if !russian_layout {
        return c;
    }

    match c {
        'q' => 'й',
        'w' => 'ц',
        'e' => 'у',
        'r' => 'к',
        't' => 'е',
        'y' => 'н',
        'u' => 'г',
        'i' => 'ш',
        'o' => 'щ',
        'p' => 'з',
        '[' => 'х',
        ']' => 'ъ',
        'a' => 'ф',
        's' => 'ы',
        'd' => 'в',
        'f' => 'а',
        'g' => 'п',
        'h' => 'р',
        'j' => 'о',
        'k' => 'л',
        'l' => 'д',
        ';' => 'ж',
        '\'' => 'э',
        'z' => 'я',
        'x' => 'ч',
        'c' => 'с',
        'v' => 'м',
        'b' => 'и',
        'n' => 'т',
        'm' => 'ь',
        ',' => 'б',
        '.' => 'ю',
        'Q' => 'Й',
        'W' => 'Ц',
        'E' => 'У',
        'R' => 'К',
        'T' => 'Е',
        'Y' => 'Н',
        'U' => 'Г',
        'I' => 'Ш',
        'O' => 'Щ',
        'P' => 'З',
        '{' => 'Х',
        '}' => 'Ъ',
        'A' => 'Ф',
        'S' => 'Ы',
        'D' => 'В',
        'F' => 'А',
        'G' => 'П',
        'H' => 'Р',
        'J' => 'О',
        'K' => 'Л',
        'L' => 'Д',
        ':' => 'Ж',
        '"' => 'Э',
        'Z' => 'Я',
        'X' => 'Ч',
        'C' => 'С',
        'V' => 'М',
        'B' => 'И',
        'N' => 'Т',
        'M' => 'Ь',
        '<' => 'Б',
        '>' => 'Ю',
        '`' => 'ё',
        '~' => 'Ё',
        _ => c,
    }
}

struct LocalizedBuffer {
    codepoints: [u32; 256],
    len: usize,
}

impl LocalizedBuffer {
    fn new() -> Self {
        Self {
            codepoints: [0; 256],
            len: 0,
        }
    }

    fn as_slice(&self) -> &[u32] {
        &self.codepoints[..self.len]
    }
}

impl Write for LocalizedBuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for ch in s.chars() {
            if self.len >= self.codepoints.len() {
                break;
            }
            self.codepoints[self.len] = ch as u32;
            self.len += 1;
        }
        Ok(())
    }
}

// ============================================================
// GLOBAL LOCALE STATE
// ============================================================

static mut CURRENT_LOCALE: Locale = Locale::RuRu;
static mut CURRENT_MODE: MessageMode = MessageMode::Lore;

pub fn get_locale() -> Locale {
    // SAFETY: CURRENT_LOCALE — static mut, читается только из однопоточного ядра.
    // Все изменения происходят через set_locale/cycle_locale под отключёнными прерываниями.
    unsafe { CURRENT_LOCALE }
}
pub fn get_mode() -> MessageMode {
    // SAFETY: аналогично CURRENT_LOCALE — однопоточный доступ из ядра.
    unsafe { CURRENT_MODE }
}
pub fn set_locale(locale: Locale) {
    // SAFETY: запись в static mut — вызывается только из команд CLI (однопоточно).
    unsafe {
        CURRENT_LOCALE = locale;
    }
}
pub fn set_mode(mode: MessageMode) {
    // SAFETY: запись в static mut — вызывается только из команд CLI (однопоточно).
    unsafe {
        CURRENT_MODE = mode;
    }
}

/// Циклически переключает локаль RU→EN→AR→RU и возвращает новую.
pub fn cycle_locale() -> Locale {
    unsafe {
        CURRENT_LOCALE = match CURRENT_LOCALE {
            Locale::RuRu => Locale::EnUs,
            Locale::EnUs => Locale::ArEg,
            Locale::ArEg => Locale::RuRu,
        };
        CURRENT_LOCALE
    }
}

/// Переключает режим Lore↔Technical и возвращает новый.
pub fn toggle_mode() -> MessageMode {
    unsafe {
        CURRENT_MODE = match CURRENT_MODE {
            MessageMode::Lore => MessageMode::Technical,
            MessageMode::Technical => MessageMode::Lore,
        };
        CURRENT_MODE
    }
}

// ============================================================
// RENDER PIPELINE (через стандартный println!)
// Использовать для интерактивных команд (не в обработчиках прерываний).
// ============================================================

/// Рендерит событие через текущую глобальную локаль и режим.
pub fn render_event_auto(ev: KernelEvent) {
    render_event(ev, get_mode());
}

/// Рендерит событие через текущую локаль и указанный режим.
pub fn render_event(ev: KernelEvent, mode: MessageMode) {
    let locale = get_locale();
    let entry = get_event_text(locale, ev);
    let msg = match mode {
        MessageMode::Technical => entry.technical,
        MessageMode::Lore => entry.lore,
    };
    print_localized_line(msg, 0x0E);
}

pub fn print_localized_line(text: &str, color: u8) {
    print_localized_fmt(color, format_args!("{}", text));
}

#[inline]
fn boot_text_row() -> usize {
    if crate::fb_buffer::is_initialized() {
        crate::fb_buffer::current_row()
    } else {
        1
    }
}

#[inline]
fn boot_text_last_col() -> usize {
    if crate::fb_buffer::is_initialized() {
        crate::fb_buffer::default_text_last_col()
    } else {
        79
    }
}

#[inline]
fn boot_text_cols() -> usize {
    if crate::fb_buffer::is_initialized() {
        crate::fb_buffer::default_text_cols()
    } else {
        80
    }
}

#[inline]
fn boot_text_left_col() -> usize {
    if crate::fb_buffer::is_initialized() {
        crate::fb_buffer::default_text_left_col()
    } else {
        0
    }
}

pub fn print_localized_fmt(color: u8, args: fmt::Arguments) {
    let mut buf = LocalizedBuffer::new();
    let _ = buf.write_fmt(args);
    let row = boot_text_row();
    let left_col = boot_text_left_col();
    let last_col = boot_text_last_col();

    unsafe {
        let spec = locale_spec(get_locale());
        match spec.direction {
            TextDirection::Ltr => {
                write_codepoints_ltr_direct(buf.as_slice(), row, left_col, color, spec.numerals)
            }
            TextDirection::Rtl => {
                write_codepoints_rtl_direct(buf.as_slice(), row, last_col, color, spec.numerals)
            }
        }
    }

    x86_64::instructions::interrupts::without_interrupts(|| {
        crate::fb_buffer::advance_line();
    });
}

/// Вариант print_boot_status для format_args! — буферизует через LocalizedBuffer, затем вызывает print_boot_status.
pub fn print_boot_status_fmt(args: fmt::Arguments) {
    let mut buf = LocalizedBuffer::new();
    let _ = buf.write_fmt(args);
    // Восстанавливаем строку из codepoints (ASCII-совместимые codepoints → bytes)
    // Для boot-сообщений все символы <= 0x7FF, поэтому конвертация надёжна.
    let mut tmp = [0u8; 512];
    let mut tmp_len = 0usize;
    for &cp in buf.as_slice() {
        if tmp_len + 4 >= tmp.len() {
            break;
        }
        if let Some(ch) = char::from_u32(cp) {
            let n = ch.encode_utf8(&mut tmp[tmp_len..]).len();
            tmp_len += n;
        }
    }
    if let Ok(s) = core::str::from_utf8(&tmp[..tmp_len]) {
        print_boot_status(s);
    }
}

/// остаток опять жёлтым. Все вхождения "ОК"/"OK" красятся зелёным.
/// Только LTR (для Arabic fallback — вся строка жёлтая).
pub fn print_boot_status(msg: &str) {
    let row = boot_text_row();
    let left_col = boot_text_left_col();
    let last_col = boot_text_last_col();
    let max_cols = boot_text_cols();

    unsafe {
        let spec = locale_spec(get_locale());

        if matches!(spec.direction, TextDirection::Rtl) {
            let mut buf = LocalizedBuffer::new();
            let _ = buf.write_str(msg);
            write_codepoints_rtl_direct(buf.as_slice(), row, last_col, 0x0E, spec.numerals);
        } else {
            // Разбиваем строку на сегменты: чередуем yellow и green для каждого ОК/OK
            let mut col: usize = 0;
            let mut remaining = msg;
            loop {
                // Ищем ближайшее "ОК" или "OK"
                let found_ok = remaining.find("ОК").map(|i| (i, 4usize)); // ОК = 4 байта UTF-8
                let found_lat = remaining.find("OK").map(|i| (i, 2usize));
                let found = match (found_ok, found_lat) {
                    (Some(a), Some(b)) => Some(if a.0 <= b.0 { a } else { b }),
                    (Some(a), None) => Some(a),
                    (None, Some(b)) => Some(b),
                    (None, None) => None,
                };
                match found {
                    Some((idx, ok_len)) => {
                        // Жёлтый префикс
                        if idx > 0 {
                            let mut pre = LocalizedBuffer::new();
                            let _ = pre.write_str(&remaining[..idx]);
                            write_codepoints_ltr_direct(
                                pre.as_slice(),
                                row,
                                left_col + col.min(max_cols.saturating_sub(1)),
                                0x0E,
                                spec.numerals,
                            );
                            col += pre.as_slice().len();
                        }
                        // Зелёный ОК
                        let mut ok = LocalizedBuffer::new();
                        let _ = ok.write_str(&remaining[idx..idx + ok_len]);
                        write_codepoints_ltr_direct(
                            ok.as_slice(),
                            row,
                            left_col + col.min(max_cols.saturating_sub(1)),
                            0x0A,
                            spec.numerals,
                        );
                        col += ok.as_slice().len();
                        remaining = &remaining[idx + ok_len..];
                    }
                    None => {
                        // Оставшийся суффикс жёлтый
                        if !remaining.is_empty() {
                            let mut suf = LocalizedBuffer::new();
                            let _ = suf.write_str(remaining);
                            write_codepoints_ltr_direct(
                                suf.as_slice(),
                                row,
                                left_col + col.min(max_cols.saturating_sub(1)),
                                0x0E,
                                spec.numerals,
                            );
                        }
                        break;
                    }
                }
            }
        }
    }

    x86_64::instructions::interrupts::without_interrupts(|| {
        crate::fb_buffer::advance_line();
    });
}

// ============================================================
// PRINT RTL LINE — вывод строки справа налево в нижнюю строку VGA.
// Использовать только при захваченном без_прерываний контексте
// или вне критических секций.
// ============================================================

/// Печатает строку состояния Фазы 6 с зелёными статусными словами.
/// RU: "загружена" и "активна" — зелёные.
/// EN: "loaded" и "active" — зелёные.
/// AR: "محمل" и "مفعلة" — жёлтые (RTL fallback).
pub fn print_phase6_ok() {
    unsafe {
        let spec = locale_spec(get_locale());
        let segments: &[(&str, u8)] = match get_locale() {
            crate::kernel_messages::Locale::RuRu => &[
                ("[Фаза 6] Шрифт: кириллица ", 0x0E),
                ("загружена", 0x0A),
                (" | Локаль: ", 0x0E),
                ("активна", 0x0A),
            ],
            crate::kernel_messages::Locale::EnUs => &[
                ("[Phase 6] Font: Cyrillic ", 0x0E),
                ("loaded", 0x0A),
                (" | Locale: ", 0x0E),
                ("active", 0x0A),
            ],
            crate::kernel_messages::Locale::ArEg => {
                &[("[المرحلة 6] الخط: Cyrillic محمل | اللغة: مفعلة", 0x0E)]
            }
        };
        let row = boot_text_row();
        let left_col = boot_text_left_col();
        let last_col = boot_text_last_col();
        let max_cols = boot_text_cols();
        if matches!(spec.direction, TextDirection::Rtl) {
            // RTL: одна строка целиком
            if let Some(&(text, color)) = segments.first() {
                let mut buf = LocalizedBuffer::new();
                let _ = buf.write_str(text);
                write_codepoints_rtl_direct(buf.as_slice(), row, last_col, color, spec.numerals);
            }
        } else {
            let mut col: usize = 0;
            for &(text, color) in segments {
                let mut buf = LocalizedBuffer::new();
                let _ = buf.write_str(text);
                write_codepoints_ltr_direct(
                    buf.as_slice(),
                    row,
                    left_col + col.min(max_cols.saturating_sub(1)),
                    color,
                    spec.numerals,
                );
                col += buf.as_slice().len();
            }
        }
    }
    x86_64::instructions::interrupts::without_interrupts(|| {
        crate::fb_buffer::advance_line();
    });
}

/// Печатает строку RTL на нижней строке VGA (row 24).
/// Предварительно вызывает новую строку, чтобы освободить row 24.
pub fn print_rtl_line(text: &str, color: u8) {
    print_localized_line(text, color);
}

// ============================================================
// PANIC SCREEN — прямой framebuffer-доступ без блокировок.
// Вызывать из обработчиков исключений и panic handler.
// НЕБЕЗОПАСНО: не использовать вне контекста краша/паники.
// ============================================================

/// Рисует экран аварии прямо в framebuffer (без mutex, без прерываний).
/// Если framebuffer ещё не поднят, визуальный вывод пропускается и остаётся serial.
pub unsafe fn render_panic_screen(ev: KernelEvent) {
    if !crate::fb_buffer::is_initialized() {
        return;
    }

    const ATTR_FRAME: u8 = 0x5F; // White on Magenta  — рамка
    const ATTR_TITLE: u8 = 0x5E; // Yellow on Magenta — заголовок в рамке
    const ATTR_TEXT: u8 = 0x0F; // Bright White on Black — основной текст
    const ATTR_LORE: u8 = 0x0D; // Bright Magenta on Black — lore / акценты
    const ATTR_HINT: u8 = 0x07; // Light Gray on Black — подсказка
    const W: usize = 80;

    // --- Заливаем весь экран чёрным ---
    for row in 0..25 {
        for col in 0..W {
            crate::fb_buffer::write_char_at(col, row, b' ', 0x00);
        }
    }

    // --- Рисуем двойную рамку (CP437: ╔═╗║╚═╝) ---
    crate::fb_buffer::write_char_at(0, 0, 0xC9, ATTR_FRAME);
    for col in 1..79 {
        crate::fb_buffer::write_char_at(col, 0, 0xCD, ATTR_FRAME);
    }
    crate::fb_buffer::write_char_at(79, 0, 0xBB, ATTR_FRAME);

    for row in 1..24 {
        crate::fb_buffer::write_char_at(0, row, 0xBA, ATTR_FRAME);
        crate::fb_buffer::write_char_at(79, row, 0xBA, ATTR_FRAME);
    }

    crate::fb_buffer::write_char_at(0, 24, 0xC8, ATTR_FRAME);
    for col in 1..79 {
        crate::fb_buffer::write_char_at(col, 24, 0xCD, ATTR_FRAME);
    }
    crate::fb_buffer::write_char_at(79, 24, 0xBC, ATTR_FRAME);

    // --- Строка 0: заголовок в рамке ---
    let hdr = b">>> NeroShizaDev-OS KERNEL EVENT <<<";
    let hdr_start = (W - hdr.len()) / 2;
    for (i, &b) in hdr.iter().enumerate() {
        crate::fb_buffer::write_char_at(hdr_start + i, 0, b, ATTR_TITLE);
    }

    let locale = get_locale();
    let mode = get_mode();
    let en_entry = get_event_text(Locale::EnUs, ev);

    // --- Строка 2: код события + локаль + режим ---
    let ev_name = crate::kernel_messages::event_short_name(ev);
    let loc_name = locale_name(locale);
    let mod_name = mode_name(mode);
    write_crash_text_at_vga(ev_name, 2, 2, ATTR_TEXT);
    write_crash_text_at_vga(" [", 2, 7, ATTR_TEXT);
    write_crash_text_at_vga(loc_name, 2, 9, ATTR_TEXT);
    write_crash_text_at_vga("/", 2, 11, ATTR_TEXT);
    write_crash_text_at_vga(mod_name, 2, 12, ATTR_TEXT);
    write_crash_text_at_vga("]", 2, 16, ATTR_TEXT);

    // --- Строки 4..5: panic path stays ASCII-only for reliability ---
    write_ascii_at_vga(en_entry.lore, 4, 2, ATTR_LORE);
    write_ascii_at_vga(en_entry.technical, 5, 2, ATTR_TEXT);

    // --- Строка 9: код аварии 0xBEDABEDA01 ---
    write_ascii_at_vga("0xBEDABEDA01 / system crashed!", 9, 2, ATTR_LORE);

    // --- Строка 11: модуль (имя события) ---
    write_ascii_at_vga("Module: ", 11, 2, ATTR_TEXT);
    write_ascii_at_vga(ev_name, 11, 19, ATTR_LORE);

    // --- Строки 13..14: быстрая подсказка по шаблону падения ---
    if crate::trace::contains_recent("timer irq0 eoi sent") {
        write_ascii_at_vga("After IRQ0 return", 13, 2, ATTR_HINT);
    }
    if ev == KernelEvent::DoubleFault {
        if crate::trace::contains_recent("general protection handler entered") {
            write_ascii_at_vga("Primary fault: GP", 14, 2, ATTR_HINT);
        } else if crate::trace::contains_recent("page fault handler entered") {
            write_ascii_at_vga("Primary fault: PF", 14, 2, ATTR_HINT);
        } else if crate::trace::contains_recent("breakpoint handler entered") {
            write_ascii_at_vga("Primary fault: BP", 14, 2, ATTR_HINT);
        } else if crate::trace::contains_recent("timer irq0 entered") {
            write_ascii_at_vga("Primary fault: IRQ0", 14, 2, ATTR_HINT);
        }
    } else if ev == KernelEvent::GeneralProtection {
        write_ascii_at_vga("GP captured before DF", 14, 2, ATTR_HINT);
    }

    // --- Строки 18..22: последние действия ядра ---
    write_ascii_at_vga("Last actions:", 18, 2, ATTR_TEXT);
    let trace_len = crate::trace::len();
    let first = trace_len.saturating_sub(4);
    let mut row = 19usize;
    let mut idx = first;
    if trace_len == 0 {
        write_ascii_at_vga("- no trace recorded", 19, 2, ATTR_HINT);
    }
    while idx < trace_len && row < 23 {
        if let Some(action) = crate::trace::get_recent(idx) {
            write_ascii_at_vga("- ", row, 2, ATTR_HINT);
            write_crash_text_at_vga(trace_action_label(action), row, 4, ATTR_HINT);
        }
        idx += 1;
        row += 1;
    }

    // --- Строка 23: подсказка ---
    write_ascii_at_vga("System halted. Reset or power cycle.", 23, 9, ATTR_HINT);
}

fn trace_action_label(action: &'static str) -> &'static str {
    match action {
        "heap initialized" => "heap initialized",
        "vga mapped + text mode" => "VGA mapped + text mode",
        "vga detect complete" => "VGA detect complete",
        "gdt idt pics fpu ready" => "CPU init ready",
        "rtc validator phase done" => "RTC validator done",
        "ps2 probe complete" => "PS/2 probe done",
        "font + locale ready" => "font + locale ready",
        "shell prompt drawn" => "shell prompt drawn",
        "interrupts enabled" => "interrupts enabled",
        "timer irq0 entered" => "timer irq0 entered",
        "timer irq0 eoi sent" => "timer irq0 eoi sent",
        "panic handler entered" => "panic handler entered",
        "page fault handler entered" => "page fault handler entered",
        "general protection handler entered" => "GP handler entered",
        "double fault handler entered" => "DF handler entered",
        "breakpoint handler entered" => "BP handler entered",
        _ => action,
    }
}

// ============================================================
// ВНУТРЕННИЕ ФУНКЦИИ ПРЯМОГО ВЫВОДА В FRAMEBUFFER
// ============================================================

/// Записывает строку LTR по координатам (row, start_col) через framebuffer.
///
/// # Safety
/// row < 25, start_col < 80.
pub unsafe fn write_str_at_vga(s: &str, row: usize, start_col: usize, attr: u8) {
    debug_assert!(row < 25, "write_str_at_vga: row={} >= 25", row);
    debug_assert!(
        start_col < 80,
        "write_str_at_vga: start_col={} >= 80",
        start_col
    );
    let numerals = locale_spec(get_locale()).numerals;
    let mut col = start_col;
    for c in s.chars() {
        if col >= 80 {
            break;
        }
        let mapped = map_display_char(c, numerals);
        crate::fb_buffer::write_codepoint_at(col, row, mapped as u32, attr);
        col += 1;
    }
}

unsafe fn write_ascii_at_vga(s: &str, row: usize, start_col: usize, attr: u8) {
    debug_assert!(row < 25, "write_ascii_at_vga: row={} >= 25", row);
    debug_assert!(
        start_col < 80,
        "write_ascii_at_vga: start_col={} >= 80",
        start_col
    );
    let mut col = start_col;
    for &byte in s.as_bytes() {
        if col >= 80 {
            break;
        }
        let cp = if byte.is_ascii() {
            byte as u32
        } else {
            b'?' as u32
        };
        crate::fb_buffer::write_codepoint_at(col, row, cp, attr);
        col += 1;
    }
}

pub unsafe fn write_ascii_str_at_vga(s: &str, row: usize, start_col: usize, attr: u8) {
    write_ascii_at_vga(s, row, start_col, attr);
}

unsafe fn write_crash_text_at_vga(s: &str, row: usize, start_col: usize, attr: u8) {
    debug_assert!(row < 25, "write_crash_text_at_vga: row={} >= 25", row);
    debug_assert!(
        start_col < 80,
        "write_crash_text_at_vga: start_col={} >= 80",
        start_col
    );
    write_ascii_at_vga(s, row, start_col, attr);
}

/// Записывает строку RTL в framebuffer, начиная с (row, end_col) и двигаясь влево.
///
/// # Safety
/// row < 25, end_col < 80.
pub unsafe fn write_str_rtl_direct(s: &str, row: usize, end_col: usize, attr: u8) {
    debug_assert!(row < 25, "write_str_rtl_direct: row={} >= 25", row);
    debug_assert!(
        end_col < 80,
        "write_str_rtl_direct: end_col={} >= 80",
        end_col
    );
    // Собираем codepoints в буфер для единого RTL-рендера
    let mut buf = [0u32; 128];
    let mut len = 0usize;
    for c in s.chars() {
        if len >= 128 {
            break;
        }
        buf[len] = c as u32;
        len += 1;
    }
    let numerals = locale_spec(get_locale()).numerals;
    write_codepoints_rtl_direct(&buf[..len], row, end_col, attr, numerals);
}

/// Рисует кодпоинты LTR начиная с (row, start_col).
///
/// # Safety
/// row/start_col должны указывать в видимую текстовую сетку framebuffer.
unsafe fn write_codepoints_ltr_direct(
    codepoints: &[u32],
    row: usize,
    start_col: usize,
    attr: u8,
    numerals: NumeralSystem,
) {
    let max_col = boot_text_left_col() + boot_text_cols();
    let mut col = start_col;
    for &cp in codepoints {
        if col >= max_col {
            break;
        }
        let mapped = map_display_char(char::from_u32(cp).unwrap_or('?'), numerals);
        crate::fb_buffer::write_codepoint_at(col, row, mapped as u32, attr);
        col += 1;
    }
}

/// Рисует кодпоинты RTL начиная с (row, end_col) и двигаясь влево.
///
/// # Safety
/// row/end_col должны указывать в видимую текстовую сетку framebuffer.
unsafe fn write_codepoints_rtl_direct(
    codepoints: &[u32],
    row: usize,
    end_col: usize,
    attr: u8,
    numerals: NumeralSystem,
) {
    // SAFETY: col уменьшается с `if col == 0 { return }` — не выходит за 0.
    // Числа (ASCII 0-9, '.', ',') пишутся в обратном порядке, чтобы отображаться
    // слева направо внутри RTL-строки (BiDi: weak LTR run в RTL-контексте).
    let min_col = boot_text_left_col();
    let mut col = end_col;
    let n = codepoints.len();
    let mut i = 0usize;
    while i < n {
        let cp = codepoints[i];
        // Начало числового прогона?
        if cp >= 0x30 && cp <= 0x39 {
            // Находим конец прогона: цифры и разделители '.' ',' между ними
            let run_start = i;
            i += 1;
            while i < n {
                let c = codepoints[i];
                if (c >= 0x30 && c <= 0x39) || c == 0x2E || c == 0x2C {
                    i += 1;
                } else {
                    break;
                }
            }
            // Пишем прогон В ОБРАТНОМ порядке (i-1 → run_start),
            // чтобы первая цифра оказалась на меньшем col → LTR на экране
            let mut k = i;
            while k > run_start {
                k -= 1;
                let mapped =
                    map_display_char(char::from_u32(codepoints[k]).unwrap_or('?'), numerals);
                crate::fb_buffer::write_codepoint_at(col, row, mapped as u32, attr);
                if col <= min_col {
                    return;
                }
                col -= 1;
            }
        } else {
            let mapped = map_display_char(char::from_u32(cp).unwrap_or('?'), numerals);
            crate::fb_buffer::write_codepoint_at(col, row, mapped as u32, attr);
            if col <= min_col {
                return;
            }
            col -= 1;
            i += 1;
        }
    }
}

/// Конвертирует символ в VGA-байт (slot):
/// ASCII → прямо, кириллица → font.rs, остальное → vga_unicode glyph cache.
unsafe fn codepoint_to_vga_byte(c: char) -> u8 {
    let cp = c as u32;
    if let Some(byte) = crate::vga_unicode::codepoint_to_vga_byte(cp) {
        return byte;
    }
    b'?'
}

// ============================================================
// ПУБЛИЧНЫЙ API ДЛЯ LIB.RS
// ============================================================

/// Отображает строку статуса локали в правом верхнем углу экрана (колонки 72-79).
/// Формат: "[RU|LORE]" — 9 символов, row 0, col 72..80.
///
/// Пишет через framebuffer в правый верхний угол текстовой сетки.
pub fn draw_locale_badge() {
    let locale = get_locale();
    let mode = get_mode();

    let loc_badge: &[u8] = match locale {
        Locale::RuRu => b"RU",
        Locale::EnUs => b"EN",
        Locale::ArEg => b"AR",
    };
    let mode_badge: &[u8] = match mode {
        MessageMode::Lore => b"LORE",
        MessageMode::Technical => b"TECH",
    };

    let attr_loc: u8 = match locale {
        Locale::RuRu => 0x1F, // White on Blue
        Locale::EnUs => 0x2F, // White on Green
        Locale::ArEg => 0x4F, // White on Red
    };
    let attr_mode: u8 = 0x8F; // White on DarkGray

    crate::fb_buffer::write_codepoint_at(72, 0, b'[' as u32, attr_loc);
    for (i, &b) in loc_badge.iter().enumerate() {
        crate::fb_buffer::write_codepoint_at(73 + i, 0, b as u32, attr_loc);
    }
    crate::fb_buffer::write_codepoint_at(75, 0, b'|' as u32, attr_mode);
    for (i, &b) in mode_badge.iter().enumerate() {
        crate::fb_buffer::write_codepoint_at(76 + i, 0, b as u32, attr_mode);
    }
    crate::fb_buffer::write_codepoint_at(80, 0, b']' as u32, attr_mode);
}

// ============================================================
// HEX DUMP — прямая запись hex-значений в framebuffer (для panic/page-fault)
// ============================================================

/// Выводит hex u64 (16 символов) в строку без аллокаций.
///
/// # Safety
/// Вызывается из обработчиков исключений — без блокировок, без аллокаций.
/// row < 25 и start_col + 15 < 80 — обязанность вызывающего.
pub unsafe fn write_hex_at_vga(val: u64, row: usize, start_col: usize, attr: u8) {
    debug_assert!(row < 25, "write_hex_at_vga: row={} >= 25", row);
    debug_assert!(
        start_col + 15 < 80,
        "write_hex_at_vga: start_col={} overflows line",
        start_col
    );
    let hex = b"0123456789ABCDEF";
    let mut v = val;
    let mut nibbles = [0u8; 16];
    for i in (0..16).rev() {
        nibbles[i] = hex[(v & 0xF) as usize];
        v >>= 4;
    }
    for (i, &ch) in nibbles.iter().enumerate() {
        crate::fb_buffer::write_codepoint_at(start_col + i, row, ch as u32, attr);
    }
}

/// Выводит hex u32 (8 символов) в строку VGA без аллокаций.
///
/// # Safety
/// Те же требования что у write_hex_at_vga.
pub unsafe fn write_hex32_at_vga(val: u32, row: usize, start_col: usize, attr: u8) {
    debug_assert!(row < 25, "write_hex32_at_vga: row={} >= 25", row);
    debug_assert!(
        start_col + 7 < 80,
        "write_hex32_at_vga: start_col={} overflows line",
        start_col
    );
    let hex = b"0123456789ABCDEF";
    let mut v = val;
    let mut nibbles = [0u8; 8];
    for i in (0..8).rev() {
        nibbles[i] = hex[(v & 0xF) as usize];
        v >>= 4;
    }
    for (i, &ch) in nibbles.iter().enumerate() {
        crate::fb_buffer::write_codepoint_at(start_col + i, row, ch as u32, attr);
    }
}

/// Выводит «Сектор памяти: 0x<addr>» на строке 12 экрана аварии.
///
/// # Safety
/// Только из обработчиков исключений.
pub unsafe fn write_crash_address(addr: u64) {
    const ATTR_TEXT: u8 = 0x0F; // Bright White on Black
    const ATTR_LORE: u8 = 0x0D; // Bright Magenta on Black
    write_ascii_at_vga("Address: 0x", 12, 2, ATTR_TEXT);
    write_hex_at_vga(addr, 12, 22, ATTR_LORE);
}

/// Выводит регистры (RIP, RSP, RFLAGS) на строках 15-17 экрана аварии.
///
/// # Safety
/// Только из обработчиков исключений.
pub unsafe fn write_registers(rip: u64, rsp: u64, flags: u64) {
    const ATTR_LABEL: u8 = 0x07; // Light Gray on Black
    const ATTR_VALUE: u8 = 0x0D; // Bright Magenta on Black
    // Row 15: RIP
    write_ascii_at_vga("RIP: 0x", 15, 2, ATTR_LABEL);
    write_hex_at_vga(rip, 15, 9, ATTR_VALUE);
    // Row 16: RSP
    write_ascii_at_vga("RSP: 0x", 16, 2, ATTR_LABEL);
    write_hex_at_vga(rsp, 16, 9, ATTR_VALUE);
    // Row 17: RFLAGS
    write_ascii_at_vga("FLG: 0x", 17, 2, ATTR_LABEL);
    write_hex_at_vga(flags, 17, 9, ATTR_VALUE);
}

/// Выводит текст паники (truncated до 72 символов) на строке 14.
///
/// # Safety
/// Только из panic handler.
pub unsafe fn write_panic_message(msg: &str) {
    const ATTR_MSG: u8 = 0x0C; // Light Red on Black
    write_ascii_at_vga(msg, 14, 2, ATTR_MSG);
}
