// ============================================================
// FRAMEBUFFER BUFFER — программный рендерер текста
// ============================================================
// Аналог vga_buffer.rs, но пишет в linear framebuffer переданный
// загрузчиком (bootloader-api FrameBuffer).
//
// Формат пикселя: BGRx 32-bit (байты: B, G, R, 0 — как в Boot Info).
// Шрифт: Unicode-глифы через vga_unicode::glyph_bitmap (ASCII, кириллица, арабский и др.).
//
// Инициализация: вызвать fb_buffer::init() ДО любого print! из ядра.
// После инициализации vga_buffer::_print/clear_screen автоматически
// маршрутизируются сюда.
// ============================================================

use core::fmt;
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;
use spin::Mutex;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(pub u8);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode(((background as u8) << 4) | (foreground as u8))
    }
}

static CP437_LIGHT_SHADE: [u8; 16] = [
    0x55, 0x00, 0x55, 0x00, 0x55, 0x00, 0x55, 0x00, 0x55, 0x00, 0x55, 0x00, 0x55, 0x00, 0x55, 0x00,
];
static CP437_MEDIUM_SHADE: [u8; 16] = [
    0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA,
];
static CP437_DARK_SHADE: [u8; 16] = [
    0xFF, 0xAA, 0xFF, 0x55, 0xFF, 0xAA, 0xFF, 0x55, 0xFF, 0xAA, 0xFF, 0x55, 0xFF, 0xAA, 0xFF, 0x55,
];
static CP437_BOX_FULL: [u8; 16] = [0xFF; 16];
static CP437_BOX_LOWER_HALF: [u8; 16] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
];
static CP437_BOX_UPPER_HALF: [u8; 16] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static CP437_DBL_VERT: [u8; 16] = [0x24; 16];
static CP437_DBL_HORZ: [u8; 16] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00,
];
static CP437_DBL_TL: [u8; 16] = [
    0x00, 0x00, 0x00, 0x00, 0x24, 0xE4, 0xE4, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24,
];
static CP437_DBL_TR: [u8; 16] = [
    0x00, 0x00, 0x00, 0x00, 0x24, 0x27, 0x27, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24,
];
static CP437_DBL_BL: [u8; 16] = [
    0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0xE4, 0xE4, 0x24, 0x00, 0x00, 0x00, 0x00,
];
static CP437_DBL_BR: [u8; 16] = [
    0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x24, 0x27, 0x27, 0x24, 0x00, 0x00, 0x00, 0x00,
];
static CP437_DBL_TEE_LEFT: [u8; 16] = [
    0x24, 0x24, 0x24, 0x24, 0x24, 0xE4, 0xE4, 0x24, 0x24, 0xE4, 0xE4, 0x24, 0x24, 0x24, 0x24, 0x24,
];
static CP437_DBL_TEE_RIGHT: [u8; 16] = [
    0x24, 0x24, 0x24, 0x24, 0x24, 0x27, 0x27, 0x24, 0x24, 0x27, 0x27, 0x24, 0x24, 0x24, 0x24, 0x24,
];

fn extended_cp437_glyph(vga_byte: u8) -> Option<&'static [u8; 16]> {
    match vga_byte {
        0xB0 => Some(&CP437_LIGHT_SHADE),
        0xB1 => Some(&CP437_MEDIUM_SHADE),
        0xB2 => Some(&CP437_DARK_SHADE),
        0xB9 => Some(&CP437_DBL_TEE_RIGHT),
        0xBA => Some(&CP437_DBL_VERT),
        0xBC => Some(&CP437_DBL_BR),
        0xC8 => Some(&CP437_DBL_BL),
        0xC9 => Some(&CP437_DBL_TL),
        0xCC => Some(&CP437_DBL_TEE_LEFT),
        0xCD => Some(&CP437_DBL_HORZ),
        0xDB => Some(&CP437_BOX_FULL),
        0xDC => Some(&CP437_BOX_LOWER_HALF),
        0xDF => Some(&CP437_BOX_UPPER_HALF),
        _ => None,
    }
}

// ============================================================
// Параметры шрифта
// ============================================================
const FONT_W: usize = 8;
const FONT_H: usize = 16;

// ============================================================
// Shadow buffer: статический максимальный размер экрана.
// Реальные cols/rows известны после init() — берём запас под современные
// framebuffer-режимы, иначе UI рисуется только в левом верхнем VGA-квадрате.
// ============================================================
const MAX_COLS: usize = 240;
const MAX_ROWS: usize = 135;
const MAX_SCROLLBACK_LINES: usize = 512;

static mut SHADOW_CHARS: [[u8; MAX_COLS]; MAX_ROWS] = [[b' '; MAX_COLS]; MAX_ROWS];
static mut SHADOW_COLORS: [[u8; MAX_COLS]; MAX_ROWS] = [[14u8; MAX_COLS]; MAX_ROWS];
static mut SHADOW_UNICODE: [[u32; MAX_COLS]; MAX_ROWS] = [[b' ' as u32; MAX_COLS]; MAX_ROWS];
static mut SHADOW_IS_UNICODE: [[bool; MAX_COLS]; MAX_ROWS] = [[false; MAX_COLS]; MAX_ROWS];
static mut SAVED_CHARS: [[u8; MAX_COLS]; MAX_ROWS] = [[b' '; MAX_COLS]; MAX_ROWS];
static mut SAVED_COLORS: [[u8; MAX_COLS]; MAX_ROWS] = [[14u8; MAX_COLS]; MAX_ROWS];
static mut SAVED_UNICODE: [[u32; MAX_COLS]; MAX_ROWS] = [[b' ' as u32; MAX_COLS]; MAX_ROWS];
static mut SAVED_IS_UNICODE: [[bool; MAX_COLS]; MAX_ROWS] = [[false; MAX_COLS]; MAX_ROWS];
static mut SAVED_CURSOR_COL: usize = 0;
static mut SAVED_CURSOR_ROW: usize = DEFAULT_TEXT_TOP_ROW;
static mut SAVED_COLOR_CODE: u8 = 0x0E;
static mut HAS_SAVED_SCREEN: bool = false;
static mut SCROLLBACK_CHARS: [[u8; MAX_COLS]; MAX_SCROLLBACK_LINES] =
    [[b' '; MAX_COLS]; MAX_SCROLLBACK_LINES];
static mut SCROLLBACK_COLORS: [[u8; MAX_COLS]; MAX_SCROLLBACK_LINES] =
    [[14u8; MAX_COLS]; MAX_SCROLLBACK_LINES];
static mut SCROLLBACK_UNICODE: [[u32; MAX_COLS]; MAX_SCROLLBACK_LINES] =
    [[b' ' as u32; MAX_COLS]; MAX_SCROLLBACK_LINES];
static mut SCROLLBACK_IS_UNICODE: [[bool; MAX_COLS]; MAX_SCROLLBACK_LINES] =
    [[false; MAX_COLS]; MAX_SCROLLBACK_LINES];
static mut SCROLLBACK_HEAD: usize = 0;
static mut SCROLLBACK_LEN: usize = 0;

// ============================================================
// Параметры framebuffer — заполняются однократно из boot_info
// ============================================================
static FB_ADDR: AtomicUsize = AtomicUsize::new(0);
static FB_STRIDE: AtomicUsize = AtomicUsize::new(0); // в пикселях
static FB_BYTE_LEN: AtomicUsize = AtomicUsize::new(0);
static FB_COLS: AtomicUsize = AtomicUsize::new(0);
static FB_ROWS: AtomicUsize = AtomicUsize::new(0);
static FB_BPP: AtomicUsize = AtomicUsize::new(4); // байт на пиксель (3=BGR, 4=BGRx)
const DEFAULT_TEXT_COLS: usize = 80;
const DEFAULT_TEXT_TOP_ROW: usize = 1;

/// Инициализирует framebuffer-рендерер.
///
/// - `fb_addr`  — виртуальный адрес начала буфера (из `boot_info.framebuffer`)
/// - `width`    — ширина в пикселях
/// - `height`   — высота в пикселях
/// - `stride`   — шаг строки в **пикселях** (может быть > width)
///
/// # Safety
/// `fb_addr` должен быть корректным виртуальным адресом framebuffer,
/// остающимся валидным на всё время работы ядра.
pub unsafe fn init(
    fb_addr: usize,
    width: usize,
    height: usize,
    stride: usize,
    bpp: usize,
    byte_len: usize,
) {
    if fb_addr == 0 || width == 0 || height == 0 || stride == 0 || bpp == 0 {
        return;
    }
    let cols = (width / FONT_W).min(MAX_COLS);
    let rows = (height / FONT_H).min(MAX_ROWS);
    FB_ADDR.store(fb_addr, Ordering::Release);
    FB_STRIDE.store(stride, Ordering::Release);
    FB_BYTE_LEN.store(byte_len, Ordering::Release);
    FB_COLS.store(cols, Ordering::Release);
    FB_ROWS.store(rows, Ordering::Release);
    FB_BPP.store(bpp, Ordering::Release);
}

/// Возвращает true, если framebuffer успешно инициализирован.
pub fn is_initialized() -> bool {
    FB_ADDR.load(Ordering::Acquire) != 0
}

#[inline]
fn fb_addr() -> usize {
    FB_ADDR.load(Ordering::Relaxed)
}
#[inline]
fn fb_stride() -> usize {
    FB_STRIDE.load(Ordering::Relaxed)
}
#[inline]
fn fb_byte_len() -> usize {
    FB_BYTE_LEN.load(Ordering::Relaxed)
}
#[inline]
fn fb_bpp() -> usize {
    FB_BPP.load(Ordering::Relaxed)
}
#[inline]
pub fn get_cols() -> usize {
    FB_COLS.load(Ordering::Relaxed)
}
#[inline]
pub fn get_rows() -> usize {
    FB_ROWS.load(Ordering::Relaxed)
}

#[inline]
pub fn default_text_cols() -> usize {
    get_cols().clamp(1, DEFAULT_TEXT_COLS)
}

#[inline]
pub fn default_text_left_col() -> usize {
    get_cols().saturating_sub(default_text_cols()) / 2
}

#[inline]
pub fn default_text_last_col() -> usize {
    default_text_left_col() + default_text_cols().saturating_sub(1)
}

// ============================================================
// Палитра CGA (16 цветов) → BGR (B=buf[0], G=buf[1], R=buf[2])
// Стандартные CGA-цвета, 6-битная DAC VGA.
// ============================================================
fn cga_to_bgr(index: u8) -> (u8, u8, u8) {
    match index & 0x0F {
        0 => (0x00, 0x00, 0x00),  // Black
        1 => (0xAA, 0x00, 0x00),  // Blue
        2 => (0x00, 0xAA, 0x00),  // Green
        3 => (0xAA, 0xAA, 0x00),  // Cyan
        4 => (0x00, 0x00, 0xAA),  // Red
        5 => (0xAA, 0x00, 0xAA),  // Magenta
        6 => (0x00, 0x55, 0xAA),  // Brown
        7 => (0xAA, 0xAA, 0xAA),  // Light Gray
        8 => (0x55, 0x55, 0x55),  // Dark Gray
        9 => (0xFF, 0x55, 0x55),  // Light Blue
        10 => (0x55, 0xFF, 0x55), // Light Green
        11 => (0xFF, 0xFF, 0x55), // Light Cyan
        12 => (0x55, 0x55, 0xFF), // Light Red
        13 => (0xFF, 0x55, 0xFF), // Pink
        14 => (0x55, 0xFF, 0xFF), // Yellow
        _ => (0xFF, 0xFF, 0xFF),  // White (15)
    }
}

// ============================================================
// Низкоуровневые pixel-операции
// ============================================================

/// Записывает один пиксель в BGR-framebuffer.
/// offset = (y * stride + x) * 4.
#[inline]
unsafe fn write_pixel(
    addr: usize,
    stride: usize,
    bpp: usize,
    x: usize,
    y: usize,
    b: u8,
    g: u8,
    r: u8,
) {
    let offset = (y * stride + x) * bpp;
    let ptr = (addr + offset) as *mut u8;
    ptr.write_volatile(b);
    ptr.add(1).write_volatile(g);
    ptr.add(2).write_volatile(r);
    if bpp >= 4 {
        ptr.add(3).write_volatile(0x00); // alpha/padding
    }
}

/// Возвращает bitmap-глиф для VGA-байта: ASCII/кириллица через Unicode-диспетчер,
/// расширенные CP437 (рамки, тени) — через локальные статические массивы.
fn get_glyph(vga_byte: u8) -> &'static [u8; 16] {
    if let Some(bmp) = crate::vga_unicode::glyph_bitmap(vga_byte as u32) {
        return bmp;
    }
    if let Some(bmp) = extended_cp437_glyph(vga_byte) {
        return bmp;
    }
    // '?' (0x3F) всегда есть в ASCII-таблице
    crate::vga_unicode::glyph_bitmap(b'?' as u32).unwrap()
}

/// Рисует один 8×16 символ в позиции (col, row) экранной сетки.
/// `color_code` — VGA attr-байт: high nibble = bg, low nibble = fg.
unsafe fn draw_char_at_raw(
    addr: usize,
    stride: usize,
    col: usize,
    row: usize,
    ch: u8,
    color_code: u8,
) {
    let bpp = fb_bpp();
    let fg = color_code & 0x0F;
    let bg = (color_code >> 4) & 0x0F;
    let (fg_b, fg_g, fg_r) = cga_to_bgr(fg);
    let (bg_b, bg_g, bg_r) = cga_to_bgr(bg);
    let glyph = get_glyph(ch);
    let px = col * FONT_W;
    let py = row * FONT_H;
    for gy in 0..FONT_H {
        let bits = glyph[gy];
        for gx in 0..FONT_W {
            let set = (bits >> (7 - gx)) & 1 != 0;
            let (b, g, r) = if set {
                (fg_b, fg_g, fg_r)
            } else {
                (bg_b, bg_g, bg_r)
            };
            write_pixel(addr, stride, bpp, px + gx, py + gy, b, g, r);
        }
    }
}

unsafe fn draw_codepoint_at_raw(
    addr: usize,
    stride: usize,
    col: usize,
    row: usize,
    codepoint: u32,
    color_code: u8,
) {
    let glyph = crate::vga_unicode::runtime_glyph_bitmap(codepoint)
        .or_else(|| crate::vga_unicode::runtime_glyph_bitmap(b'?' as u32))
        .unwrap_or([0u8; FONT_H]);
    let bpp = fb_bpp();
    let fg = color_code & 0x0F;
    let bg = (color_code >> 4) & 0x0F;
    let (fg_b, fg_g, fg_r) = cga_to_bgr(fg);
    let (bg_b, bg_g, bg_r) = cga_to_bgr(bg);
    let px = col * FONT_W;
    let py = row * FONT_H;
    for gy in 0..FONT_H {
        let bits = glyph[gy];
        for gx in 0..FONT_W {
            let set = (bits >> (7 - gx)) & 1 != 0;
            let (b, g, r) = if set {
                (fg_b, fg_g, fg_r)
            } else {
                (bg_b, bg_g, bg_r)
            };
            write_pixel(addr, stride, bpp, px + gx, py + gy, b, g, r);
        }
    }
}

// ============================================================
// Публичный API прямой записи символа (для locale.rs)
// ============================================================

/// Рисует символ (`vga_byte` = индекс глифа, `attr` = VGA color attr) в ячейке (col, row).
/// Если FB не инициализирован или координаты вне диапазона — no-op.
/// Также обновляет теневой буфер — символ сохранится при скролле.
pub fn write_char_at(col: usize, row: usize, vga_byte: u8, attr: u8) {
    let addr = fb_addr();
    if addr == 0 {
        return;
    }
    let cols = get_cols();
    let rows = get_rows();
    if col >= cols || row >= rows {
        return;
    }
    unsafe {
        shadow_put(row, col, vga_byte, attr);
        draw_char_at_raw(addr, fb_stride(), col, row, vga_byte, attr);
    }
}

/// Рисует Unicode codepoint в ячейке framebuffer, не проходя через VGA slot cache.
pub fn write_codepoint_at(col: usize, row: usize, codepoint: u32, attr: u8) {
    let addr = fb_addr();
    if addr == 0 {
        return;
    }
    let cols = get_cols();
    let rows = get_rows();
    if col >= cols || row >= rows {
        return;
    }
    unsafe {
        shadow_put_codepoint(row, col, codepoint, attr);
        draw_codepoint_at_raw(addr, fb_stride(), col, row, codepoint, attr);
    }
}

pub fn draw_shell_input_line(
    buffer: &[u32],
    cursor: usize,
    sel_active: bool,
    sel_start: usize,
    sel_end: usize,
    russian_layout: bool,
) {
    let fb_row = current_row();
    let text_left = if is_initialized() {
        default_text_left_col()
    } else {
        0
    };
    let text_cols = if is_initialized() {
        default_text_cols()
    } else {
        80
    };
    let max_cols = text_left + text_cols;
    let prompt_col = text_left + 2;
    let normal = 0x0E;
    let sel_color = 0xE0;

    for (i, &cp) in buffer.iter().enumerate() {
        let col = prompt_col + i;
        if col >= max_cols {
            break;
        }

        let color = if sel_active && i >= sel_start && i < sel_end {
            sel_color
        } else {
            normal
        };
        write_codepoint_at(col, fb_row, cp, color);
    }

    for i in buffer.len()..text_cols.saturating_sub(2) {
        let col = prompt_col + i;
        if col >= max_cols {
            break;
        }
        write_char_at(col, fb_row, b' ', normal);
    }

    draw_shell_input_badge(fb_row, max_cols, russian_layout);
    WRITER.lock().column_position = 2 + cursor;
}

fn draw_shell_input_badge(row: usize, max_cols: usize, russian_layout: bool) {
    if max_cols < 3 {
        return;
    }

    let start_col = max_cols.saturating_sub(3);
    let (text, color) = if russian_layout {
        (b"RUS", 0x4F)
    } else {
        (b"ENG", 0x2F)
    };

    for (offset, &byte) in text.iter().enumerate() {
        write_char_at(start_col + offset, row, byte, color);
    }
}

// ============================================================
// FbWriter — аналог Writer в vga_buffer.rs
// ============================================================

pub struct FbWriter {
    /// Текущая позиция курсора в центрированной текстовой области.
    pub column_position: usize,
    /// Текущая строка вывода.
    pub row_position: usize,
    /// VGA attr-байт по умолчанию: (bg << 4) | fg.
    pub color_code: ColorCode,
}

#[inline]
unsafe fn shadow_put(row: usize, col: usize, ch: u8, color: u8) {
    (*(&raw mut SHADOW_CHARS))[row][col] = ch;
    (*(&raw mut SHADOW_COLORS))[row][col] = color;
    (*(&raw mut SHADOW_UNICODE))[row][col] = ch as u32;
    (*(&raw mut SHADOW_IS_UNICODE))[row][col] = false;
}

#[inline]
unsafe fn shadow_put_codepoint(row: usize, col: usize, codepoint: u32, color: u8) {
    (*(&raw mut SHADOW_CHARS))[row][col] = b'?';
    (*(&raw mut SHADOW_COLORS))[row][col] = color;
    (*(&raw mut SHADOW_UNICODE))[row][col] = codepoint;
    (*(&raw mut SHADOW_IS_UNICODE))[row][col] = true;
}

#[inline]
unsafe fn shadow_char(row: usize, col: usize) -> u8 {
    (*(&raw const SHADOW_CHARS))[row][col]
}

#[inline]
unsafe fn shadow_color(row: usize, col: usize) -> u8 {
    (*(&raw const SHADOW_COLORS))[row][col]
}

#[inline]
unsafe fn shadow_codepoint(row: usize, col: usize) -> u32 {
    (*(&raw const SHADOW_UNICODE))[row][col]
}

#[inline]
unsafe fn shadow_is_unicode(row: usize, col: usize) -> bool {
    (*(&raw const SHADOW_IS_UNICODE))[row][col]
}

#[inline]
unsafe fn saved_put(row: usize, col: usize, ch: u8, color: u8) {
    (*(&raw mut SAVED_CHARS))[row][col] = ch;
    (*(&raw mut SAVED_COLORS))[row][col] = color;
    (*(&raw mut SAVED_UNICODE))[row][col] = ch as u32;
    (*(&raw mut SAVED_IS_UNICODE))[row][col] = false;
}

#[inline]
unsafe fn saved_put_codepoint(row: usize, col: usize, codepoint: u32, color: u8) {
    (*(&raw mut SAVED_CHARS))[row][col] = b'?';
    (*(&raw mut SAVED_COLORS))[row][col] = color;
    (*(&raw mut SAVED_UNICODE))[row][col] = codepoint;
    (*(&raw mut SAVED_IS_UNICODE))[row][col] = true;
}

#[inline]
unsafe fn saved_char(row: usize, col: usize) -> u8 {
    (*(&raw const SAVED_CHARS))[row][col]
}

#[inline]
unsafe fn saved_color(row: usize, col: usize) -> u8 {
    (*(&raw const SAVED_COLORS))[row][col]
}

#[inline]
unsafe fn saved_codepoint(row: usize, col: usize) -> u32 {
    (*(&raw const SAVED_UNICODE))[row][col]
}

#[inline]
unsafe fn saved_is_unicode(row: usize, col: usize) -> bool {
    (*(&raw const SAVED_IS_UNICODE))[row][col]
}

#[inline]
unsafe fn scrollback_slot(line_back: usize) -> usize {
    (SCROLLBACK_HEAD + MAX_SCROLLBACK_LINES - SCROLLBACK_LEN + line_back) % MAX_SCROLLBACK_LINES
}

unsafe fn scrollback_push_row(row: usize, cols: usize) {
    let slot = SCROLLBACK_HEAD;
    for col in 0..cols {
        (*(&raw mut SCROLLBACK_CHARS))[slot][col] = shadow_char(row, col);
        (*(&raw mut SCROLLBACK_COLORS))[slot][col] = shadow_color(row, col);
        (*(&raw mut SCROLLBACK_UNICODE))[slot][col] = shadow_codepoint(row, col);
        (*(&raw mut SCROLLBACK_IS_UNICODE))[slot][col] = shadow_is_unicode(row, col);
    }
    for col in cols..MAX_COLS {
        (*(&raw mut SCROLLBACK_CHARS))[slot][col] = b' ';
        (*(&raw mut SCROLLBACK_COLORS))[slot][col] = 0x0E;
        (*(&raw mut SCROLLBACK_UNICODE))[slot][col] = b' ' as u32;
        (*(&raw mut SCROLLBACK_IS_UNICODE))[slot][col] = false;
    }
    SCROLLBACK_HEAD = (SCROLLBACK_HEAD + 1) % MAX_SCROLLBACK_LINES;
    if SCROLLBACK_LEN < MAX_SCROLLBACK_LINES {
        SCROLLBACK_LEN += 1;
    }
}

impl FbWriter {
    #[inline]
    fn text_top_row(rows: usize) -> usize {
        DEFAULT_TEXT_TOP_ROW.min(rows.saturating_sub(1))
    }

    pub fn write_byte(&mut self, byte: u8) {
        let cols = default_text_cols();
        let rows = get_rows();
        if cols == 0 || rows == 0 {
            return;
        }
        match byte {
            b'\n' => self.new_line(),
            0x08 => {
                // Backspace
                if self.column_position > 0 {
                    self.column_position -= 1;
                    let row = self.row_position.min(rows.saturating_sub(1));
                    let col = default_text_left_col() + self.column_position;
                    let addr = fb_addr();
                    if addr != 0 {
                        unsafe {
                            shadow_put(row, col, b' ', self.color_code.0);
                            draw_char_at_raw(addr, fb_stride(), col, row, b' ', self.color_code.0);
                        }
                    } else {
                        unsafe {
                            shadow_put(row, col, b' ', self.color_code.0);
                        }
                    }
                }
            }
            byte => {
                if self.column_position >= cols {
                    self.new_line();
                }
                let row = self.row_position.min(rows.saturating_sub(1));
                let col = default_text_left_col() + self.column_position;
                unsafe {
                    shadow_put(row, col, byte, self.color_code.0);
                }
                let addr = fb_addr();
                if addr != 0 {
                    unsafe {
                        draw_char_at_raw(addr, fb_stride(), col, row, byte, self.color_code.0);
                    }
                }
                self.column_position += 1;
            }
        }
    }

    pub fn write_codepoint(&mut self, codepoint: u32) {
        let cols = default_text_cols();
        let rows = get_rows();
        if cols == 0 || rows == 0 {
            return;
        }
        if codepoint == b'\n' as u32 {
            self.new_line();
            return;
        }
        if self.column_position >= cols {
            self.new_line();
        }
        let row = self.row_position.min(rows.saturating_sub(1));
        let col = default_text_left_col() + self.column_position;
        unsafe {
            shadow_put_codepoint(row, col, codepoint, self.color_code.0);
        }
        let addr = fb_addr();
        if addr != 0 {
            unsafe {
                draw_codepoint_at_raw(addr, fb_stride(), col, row, codepoint, self.color_code.0);
            }
        }
        self.column_position += 1;
    }

    fn new_line(&mut self) {
        let cols = get_cols();
        let rows = get_rows();
        let text_top_row = Self::text_top_row(rows);
        if rows == 0 {
            self.column_position = 0;
            return;
        }
        if self.row_position + 1 < rows {
            self.row_position += 1;
            self.column_position = 0;
            return;
        }
        // Сдвигаем только текстовую область: верхний служебный отступ не трогаем.
        unsafe {
            scrollback_push_row(text_top_row, cols);
            for row in (text_top_row + 1)..rows {
                for col in 0..cols {
                    let ch = shadow_char(row, col);
                    let color = shadow_color(row, col);
                    if shadow_is_unicode(row, col) {
                        let codepoint = shadow_codepoint(row, col);
                        shadow_put_codepoint(row - 1, col, codepoint, color);
                    } else {
                        shadow_put(row - 1, col, ch, color);
                    }
                }
            }
            // Очищаем последнюю строку
            for col in 0..cols {
                shadow_put(rows - 1, col, b' ', self.color_code.0);
            }
        }
        self.column_position = 0;
        self.row_position = rows.saturating_sub(1);
        // Быстрый пиксельный сдвиг вместо redraw_all() — одна операция copy
        self.fast_scroll_pixels(text_top_row, rows);
    }

    /// Сдвигает пиксели экрана на одну текстовую строку вверх через ptr::copy.
    /// Намного быстрее redraw_all(): одна операция memcpy вместо N*M символов.
    fn fast_scroll_pixels(&self, text_top_row: usize, rows: usize) {
        let addr = fb_addr();
        let stride = fb_stride();
        let bpp = fb_bpp();
        if addr == 0 || stride == 0 || bpp == 0 || rows == 0 {
            return;
        }
        let row_pixel_bytes = stride * bpp; // байт на пиксельную строку
        let font_row_bytes = FONT_H * row_pixel_bytes; // байт на одну текстовую строку
        let visible_text_rows = rows.saturating_sub(text_top_row);
        if visible_text_rows <= 1 {
            let clear_offset = text_top_row * font_row_bytes;
            unsafe {
                let row_ptr = (addr + clear_offset) as *mut u8;
                core::ptr::write_bytes(row_ptr, 0, font_row_bytes);
            }
            return;
        }
        let scroll_start = text_top_row * font_row_bytes;
        let scroll_len = (visible_text_rows - 1) * font_row_bytes;
        unsafe {
            // Сдвигаем только активную текстовую область вверх на одну строку.
            let src = (addr + scroll_start + font_row_bytes) as *const u8;
            let dst = (addr + scroll_start) as *mut u8;
            core::ptr::copy(src, dst, scroll_len);
            // Очищаем последнюю текстовую строку пикселями (чёрный = 0x00)
            let last_row_ptr = (addr + scroll_start + scroll_len) as *mut u8;
            core::ptr::write_bytes(last_row_ptr, 0, font_row_bytes);
        }
    }

    /// Полная перерисовка экрана из теневого буфера.
    /// Вызывается при скролле и clear_screen.
    fn redraw_all(&self) {
        let cols = get_cols();
        let rows = get_rows();
        let addr = fb_addr();
        let stride = fb_stride();
        if addr == 0 {
            return;
        }
        for row in 0..rows {
            for col in 0..cols {
                unsafe {
                    if shadow_is_unicode(row, col) {
                        draw_codepoint_at_raw(
                            addr,
                            stride,
                            col,
                            row,
                            shadow_codepoint(row, col),
                            shadow_color(row, col),
                        );
                    } else {
                        draw_char_at_raw(
                            addr,
                            stride,
                            col,
                            row,
                            shadow_char(row, col),
                            shadow_color(row, col),
                        );
                    }
                }
            }
        }
    }

    pub fn clear_screen(&mut self) {
        let cols = get_cols();
        let rows = get_rows();
        let addr = fb_addr();
        let byte_len = fb_byte_len();
        unsafe {
            for row in 0..rows {
                for col in 0..cols {
                    shadow_put(row, col, b' ', self.color_code.0);
                }
            }
        }
        self.column_position = 0;
        self.row_position = Self::text_top_row(rows);
        if addr != 0 && byte_len != 0 {
            unsafe {
                core::ptr::write_bytes(addr as *mut u8, 0, byte_len);
            }
        }
    }
}

impl fmt::Write for FbWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_codepoint(c as u32);
        }
        Ok(())
    }
}

// ============================================================
// Глобальный WRITER
// ============================================================

lazy_static! {
    pub static ref WRITER: Mutex<FbWriter> = Mutex::new(FbWriter {
        column_position: 0,
        row_position: DEFAULT_TEXT_TOP_ROW,
        // Yellow on Black — совпадает с vga_buffer дефолтом
        color_code: ColorCode::new(Color::Yellow, Color::Black),
    });
}

// ============================================================
// Публичные функции — зеркало vga_buffer публичного API
// ============================================================

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

pub fn clear_screen() {
    WRITER.lock().clear_screen();
}

/// Сброс позиции курсора (вызывается из locale.rs после прямых записей).
pub fn reset_column() {
    WRITER.lock().column_position = 0;
}

pub fn current_row() -> usize {
    WRITER.lock().row_position
}

pub fn advance_line() {
    WRITER.lock().new_line();
}

/// Устанавливает текущий цвет текста для WRITER.
pub fn set_color(color_code: ColorCode) {
    WRITER.lock().color_code = color_code;
}

pub fn scroll_total() -> usize {
    unsafe { SCROLLBACK_LEN }
}

pub fn save_screen_snapshot() {
    let cols = get_cols();
    let rows = get_rows();
    let writer = WRITER.lock();
    unsafe {
        for row in 0..rows {
            for col in 0..cols {
                if shadow_is_unicode(row, col) {
                    saved_put_codepoint(
                        row,
                        col,
                        shadow_codepoint(row, col),
                        shadow_color(row, col),
                    );
                } else {
                    saved_put(row, col, shadow_char(row, col), shadow_color(row, col));
                }
            }
        }
        SAVED_CURSOR_COL = writer.column_position;
        SAVED_CURSOR_ROW = writer.row_position;
        SAVED_COLOR_CODE = writer.color_code.0;
        HAS_SAVED_SCREEN = true;
    }
}

pub fn save_screen_to_scrollback() {
    let cols = get_cols();
    let rows = get_rows();
    let text_top_row = FbWriter::text_top_row(rows);
    unsafe {
        for row in text_top_row..rows {
            scrollback_push_row(row, cols);
        }
    }
}

pub fn show_scrollback(scroll_offset: usize) {
    let cols = get_cols();
    let rows = get_rows();
    let addr = fb_addr();
    let stride = fb_stride();
    if addr == 0 || rows == 0 {
        return;
    }

    let text_top_row = FbWriter::text_top_row(rows);
    let visible_rows = rows.saturating_sub(text_top_row);
    unsafe {
        let max_offset = SCROLLBACK_LEN;
        let offset = scroll_offset.min(max_offset);
        let base_line = SCROLLBACK_LEN.saturating_sub(offset);

        for vis in 0..visible_rows {
            let dst_row = text_top_row + vis;
            let source_line = base_line + vis;
            if source_line < SCROLLBACK_LEN {
                let slot = scrollback_slot(source_line);
                for col in 0..cols {
                    if (*(&raw const SCROLLBACK_IS_UNICODE))[slot][col] {
                        draw_codepoint_at_raw(
                            addr,
                            stride,
                            col,
                            dst_row,
                            (*(&raw const SCROLLBACK_UNICODE))[slot][col],
                            (*(&raw const SCROLLBACK_COLORS))[slot][col],
                        );
                    } else {
                        draw_char_at_raw(
                            addr,
                            stride,
                            col,
                            dst_row,
                            (*(&raw const SCROLLBACK_CHARS))[slot][col],
                            (*(&raw const SCROLLBACK_COLORS))[slot][col],
                        );
                    }
                }
            } else {
                let live_row = text_top_row + (source_line - SCROLLBACK_LEN);
                if live_row < rows {
                    for col in 0..cols {
                        if shadow_is_unicode(live_row, col) {
                            draw_codepoint_at_raw(
                                addr,
                                stride,
                                col,
                                dst_row,
                                shadow_codepoint(live_row, col),
                                shadow_color(live_row, col),
                            );
                        } else {
                            draw_char_at_raw(
                                addr,
                                stride,
                                col,
                                dst_row,
                                shadow_char(live_row, col),
                                shadow_color(live_row, col),
                            );
                        }
                    }
                } else {
                    for col in 0..cols {
                        draw_char_at_raw(addr, stride, col, dst_row, b' ', 0x0E);
                    }
                }
            }
        }
    }
}

pub fn restore_saved_screen() {
    let cols = get_cols();
    let rows = get_rows();
    let mut writer = WRITER.lock();
    unsafe {
        if !HAS_SAVED_SCREEN {
            writer.clear_screen();
            return;
        }
        for row in 0..rows {
            for col in 0..cols {
                if saved_is_unicode(row, col) {
                    shadow_put_codepoint(
                        row,
                        col,
                        saved_codepoint(row, col),
                        saved_color(row, col),
                    );
                } else {
                    shadow_put(row, col, saved_char(row, col), saved_color(row, col));
                }
            }
        }
        writer.column_position = SAVED_CURSOR_COL;
        writer.row_position = SAVED_CURSOR_ROW.min(rows.saturating_sub(1));
        writer.color_code = ColorCode(SAVED_COLOR_CODE);
    }
    writer.redraw_all();
}
pub fn restore_from_scrollback() {
    restore_saved_screen();
}
