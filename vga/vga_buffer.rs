use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0, Blue = 1, Green = 2, Cyan = 3, Red = 4, Magenta = 5, Brown = 6, LightGray = 7,
    DarkGray = 8, LightBlue = 9, LightGreen = 10, LightCyan = 11, LightRed = 12, Pink = 13, Yellow = 14, White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

// Use functions from vga_hw to get the actual VGA mode dimensions
fn get_buffer_height() -> usize {
    crate::vga_hw::get_rows()
}

fn get_buffer_width() -> usize {
    crate::vga_hw::get_columns()
}

// Standard VGA text mode: 80x25
// IMPORTANT: MAX_COLS must match physical VGA row stride (80 cells = 160 bytes).
// If Buffer uses a different stride, every row in memory is misaligned vs VGA
// hardware, causing a cascading indent (each row shifted right by the delta).
const MAX_COLS: usize = 80;
const MAX_ROWS: usize = 25;

/// VGA text buffer (0xB8000).
/// Все чтения/записи — через core::ptr::{read,write}_volatile,
/// чтобы компилятор не убрал обращения к memory-mapped IO.
#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; MAX_COLS]; MAX_ROWS],
}

pub struct Writer {
    pub column_position: usize,
    pub color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        let buffer_height = get_buffer_height();
        let buffer_width = get_buffer_width();
        match byte {
            b'\n' => self.new_line(),
            0x08 => {
                // Backspace: сдвигаем курсор назад и стираем
                if self.column_position > 0 {
                    self.column_position -= 1;
                    let row = buffer_height - 1;
                    let col = self.column_position;
                    let color_code = self.color_code;
                    unsafe {
                        core::ptr::write_volatile(
                            &mut self.buffer.chars[row][col],
                            ScreenChar { ascii_character: b' ', color_code },
                        );
                    }
                }
            }
            byte => {
                if self.column_position >= buffer_width {
                    self.new_line();
                }
                let row = buffer_height - 1;
                let col = self.column_position;
                let color_code = self.color_code;
                unsafe {
                    core::ptr::write_volatile(
                        &mut self.buffer.chars[row][col],
                        ScreenChar { ascii_character: byte, color_code },
                    );
                }
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        let buffer_height = get_buffer_height();
        let buffer_width = get_buffer_width();
        // Сохраняем строку 0 в scrollback перед уничтожением
        // SAFETY: SCROLLBACK/SCROLL_WRITE/SCROLL_TOTAL — static mut; вызывается
        // только из write_byte под WRITER.lock() (spin::Mutex), что гарантирует
        // взаимное исключение. &raw mut — сырой указатель без &mut-ссылки
        // (Rust edition-2024: static_mut_refs hard error).
        unsafe {
            let sw = SCROLL_WRITE;
            let sb = &raw mut SCROLLBACK as *mut u8;
            let base = sw * buffer_width * 2;
            for col in 0..buffer_width {
                let sc = core::ptr::read_volatile(&self.buffer.chars[0][col]);
                *sb.add(base + col * 2) = sc.ascii_character;
                *sb.add(base + col * 2 + 1) = sc.color_code.0;
            }
            SCROLL_WRITE = (sw + 1) % SCROLLBACK_LINES;
            if SCROLL_TOTAL < SCROLLBACK_LINES {
                SCROLL_TOTAL += 1;
            }
        }
        for row in 1..buffer_height {
            for col in 0..buffer_width {
                let character = unsafe {
                    core::ptr::read_volatile(&self.buffer.chars[row][col])
                };
                unsafe {
                    core::ptr::write_volatile(&mut self.buffer.chars[row - 1][col], character);
                }
            }
        }
        self.clear_row(buffer_height - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        let buffer_width = get_buffer_width();
        for col in 0..buffer_width {
            unsafe { core::ptr::write_volatile(&mut self.buffer.chars[row][col], blank); }
        }
    }

    pub fn clear_screen(&mut self) {
        let buffer_height = get_buffer_height();
        for row in 0..buffer_height {
            self.clear_row(row);
        }
        self.column_position = 0;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            let byte = crate::vga_unicode::codepoint_to_vga_byte(c as u32).unwrap_or(b'?');
            self.write_byte(byte);
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        // SAFETY: 0xB8000 — VGA text buffer, identity-mapped загрузчиком.
        // lazy_static гарантирует однократную инициализацию.
        // Мьютекс Mutex<Writer> обеспечивает эксклюзивный доступ к &mut Buffer.
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        // ВОТ ТУТ БЫЛА ОШИБКА. Теперь правильно: write_fmt
        WRITER.lock().write_fmt(args).unwrap();
    });
}

pub fn clear_screen() {
    WRITER.lock().clear_screen();
}

// ============================================================
// SCROLLBACK — кольцевой буфер 100 строк
// ============================================================
// Великий скролл: 1000 строк вместо 100 (160 000 байт вместо 16 000)
// Для 32 ГБ RAM это вообще ничто!
const SCROLLBACK_LINES: usize = 1000;
// Use maximum possible size for scrollback buffers (90x30 mode)
static mut SCROLLBACK: [u8; SCROLLBACK_LINES * MAX_COLS * 2] = [0; SCROLLBACK_LINES * MAX_COLS * 2];
static mut SCROLL_WRITE: usize = 0;
static mut SCROLL_TOTAL: usize = 0;
// SAVED_SCREEN — снимок VGA-экрана при входе в scroll mode
static mut SAVED_SCREEN: [u8; MAX_COLS * MAX_ROWS * 2] = [0; MAX_COLS * MAX_ROWS * 2];

pub fn scroll_total() -> usize {
    unsafe { SCROLL_TOTAL }
}

/// Сохраняет текущий VGA-экран в SAVED_SCREEN и дописывает видимые строки в scrollback.
pub fn save_screen_to_scrollback() {
    let buffer_height = get_buffer_height();
    let buffer_width = get_buffer_width();
    unsafe {
        let vga = 0xB8000 as *const u8;
        let saved = &raw mut SAVED_SCREEN as *mut u8;
        let sb = &raw mut SCROLLBACK as *mut u8;

        // 1) Снимок всего экрана для restore
        for row in 0..buffer_height {
            for col in 0..buffer_width {
                let off = row * buffer_width * 2 + col * 2;
                *saved.add(off) = *vga.add(off);
                *saved.add(off + 1) = *vga.add(off + 1);
            }
        }

        // 2) Все видимые строки (0..height-1) → в scrollback кольцевой буфер.
        //    Строка 0 = самая старая на экране, height-1 = самая новая.
        for row in 0..buffer_height {
            let sw = SCROLL_WRITE;
            let base = sw * buffer_width * 2;
            for col in 0..buffer_width {
                let off = row * buffer_width * 2 + col * 2;
                *sb.add(base + col * 2) = *vga.add(off);
                *sb.add(base + col * 2 + 1) = *vga.add(off + 1);
            }
            SCROLL_WRITE = (sw + 1) % SCROLLBACK_LINES;
            if SCROLL_TOTAL < SCROLLBACK_LINES {
                SCROLL_TOTAL += 1;
            }
        }
    }
}

/// Восстанавливает экран из SAVED_SCREEN (снимок при входе в scroll mode).
pub fn restore_saved_screen() {
    let buffer_height = get_buffer_height();
    let buffer_width = get_buffer_width();
    unsafe {
        let vga = 0xB8000 as *mut u8;
        let saved = &raw const SAVED_SCREEN as *const u8;
        for row in 0..buffer_height {
            for col in 0..buffer_width {
                let off = row * buffer_width * 2 + col * 2;
                *vga.add(off) = *saved.add(off);
                *vga.add(off + 1) = *saved.add(off + 1);
            }
        }
    }
}

pub fn restore_from_scrollback() {
    let buffer_height = get_buffer_height();
    let buffer_width = get_buffer_width();
    unsafe {
        let vga = 0xB8000 as *mut u8;
        let sb = &raw const SCROLLBACK as *const u8;
        let total = SCROLL_TOTAL;
        let write_pos = SCROLL_WRITE;
        
        // Восстанавливаем последние buffer_height строк из SCROLLBACK.
        // row=0 (top) → самый старый контент, row=height-1 (bottom) → самый свежий.
        for row in 0..buffer_height {
            let row_base = row * buffer_width * 2;
            let lines_back = buffer_height - 1 - row; // 0 = newest (bottom), height-1 = oldest (top)
            if lines_back < total {
                let idx = (write_pos + SCROLLBACK_LINES - 1 - lines_back) % SCROLLBACK_LINES;
                let sb_base = idx * buffer_width * 2;
                for col in 0..buffer_width {
                    *vga.add(row_base + col * 2) = *sb.add(sb_base + col * 2);
                    *vga.add(row_base + col * 2 + 1) = *sb.add(sb_base + col * 2 + 1);
                }
            } else {
                // Строк ещё нет — заполняем пробелами
                for col in 0..buffer_width {
                    *vga.add(row_base + col * 2) = b' ';
                    *vga.add(row_base + col * 2 + 1) = 0x07; // Gray on Black
                }
            }
        }
    }
}

pub fn show_scrollback(scroll_offset: usize) {
    let buffer_height = get_buffer_height();
    let buffer_width = get_buffer_width();
    unsafe {
        let vga = 0xB8000 as *mut u8;
        let total = SCROLL_TOTAL;
        let write_pos = SCROLL_WRITE;
        let sb = &raw const SCROLLBACK as *const u8;

        // Строки 0..(height-1): контент из scrollback
        for row in 0..(buffer_height - 1) {
            let lines_back = scroll_offset + (buffer_height - 2 - row);
            let row_base = row * buffer_width * 2;
            if lines_back < total {
                let idx = (write_pos + SCROLLBACK_LINES - 1 - lines_back) % SCROLLBACK_LINES;
                let sb_base = idx * buffer_width * 2;
                for col in 0..buffer_width {
                    *vga.add(row_base + col * 2) = *sb.add(sb_base + col * 2);
                    *vga.add(row_base + col * 2 + 1) = *sb.add(sb_base + col * 2 + 1);
                }
            } else {
                for col in 0..buffer_width {
                    *vga.add(row_base + col * 2) = b' ';
                    *vga.add(row_base + col * 2 + 1) = 0x07;
                }
            }
        }
        // Последняя строка: статус-бар
        let row_base = (buffer_height - 1) * buffer_width * 2;
        for col in 0..buffer_width {
            *vga.add(row_base + col * 2) = b' ';
            *vga.add(row_base + col * 2 + 1) = 0x70; // Black on LightGray
        }
        let msg = b" PgUp/PgDn  Esc=back";
        for (i, &ch) in msg.iter().enumerate() {
            *vga.add(row_base + i * 2) = ch;
        }
    }
}
