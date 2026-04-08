use volatile::Volatile;
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

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    pub column_position: usize,
    pub color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            0x08 => {
                // Backspace: сдвигаем курсор назад и стираем
                if self.column_position > 0 {
                    self.column_position -= 1;
                    let row = BUFFER_HEIGHT - 1;
                    let col = self.column_position;
                    let color_code = self.color_code;
                    self.buffer.chars[row][col].write(ScreenChar {
                        ascii_character: b' ',
                        color_code,
                    });
                }
            }
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        // Сохраняем строку 0 в scrollback перед уничтожением
        unsafe {
            let sw = SCROLL_WRITE;
            let sb = &raw mut SCROLLBACK as *mut u8;
            let base = sw * BUFFER_WIDTH * 2;
            for col in 0..BUFFER_WIDTH {
                let sc = self.buffer.chars[0][col].read();
                *sb.add(base + col * 2) = sc.ascii_character;
                *sb.add(base + col * 2 + 1) = sc.color_code.0;
            }
            SCROLL_WRITE = (sw + 1) % SCROLLBACK_LINES;
            if SCROLL_TOTAL < SCROLLBACK_LINES {
                SCROLL_TOTAL += 1;
            }
        }
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.column_position = 0;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            if let Some(vga_code) = crate::font::cyrillic_to_vga(c) {
                self.write_byte(vga_code);
            } else if (c as u32) < 128 {
                self.write_byte(c as u8);
            } else {
                self.write_byte(b'?');
            }
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
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
const SCROLLBACK_LINES: usize = 100;
static mut SCROLLBACK: [u8; SCROLLBACK_LINES * BUFFER_WIDTH * 2] = [0; SCROLLBACK_LINES * BUFFER_WIDTH * 2];
static mut SCROLL_WRITE: usize = 0;
static mut SCROLL_TOTAL: usize = 0;
static mut SAVED_SCREEN: [u8; BUFFER_HEIGHT * BUFFER_WIDTH * 2] = [0; BUFFER_HEIGHT * BUFFER_WIDTH * 2];

pub fn scroll_total() -> usize {
    unsafe { SCROLL_TOTAL }
}

pub fn save_screen() {
    unsafe {
        let vga = 0xB8000 as *const u8;
        let ptr = &raw mut SAVED_SCREEN as *mut u8;
        for i in 0..(BUFFER_HEIGHT * BUFFER_WIDTH * 2) {
            *ptr.add(i) = *vga.add(i);
        }
    }
}

pub fn restore_screen() {
    unsafe {
        let vga = 0xB8000 as *mut u8;
        let ptr = &raw const SAVED_SCREEN as *const u8;
        for i in 0..(BUFFER_HEIGHT * BUFFER_WIDTH * 2) {
            *vga.add(i) = *ptr.add(i);
        }
    }
}

pub fn show_scrollback(scroll_offset: usize) {
    unsafe {
        let vga = 0xB8000 as *mut u8;
        let total = SCROLL_TOTAL;
        let write_pos = SCROLL_WRITE;
        let sb = &raw const SCROLLBACK as *const u8;

        // Строки 0-23: контент из scrollback
        for row in 0..(BUFFER_HEIGHT - 1) {
            let lines_back = scroll_offset + (BUFFER_HEIGHT - 2 - row);
            let row_base = row * BUFFER_WIDTH * 2;
            if lines_back < total {
                let idx = (write_pos + SCROLLBACK_LINES - 1 - lines_back) % SCROLLBACK_LINES;
                let sb_base = idx * BUFFER_WIDTH * 2;
                for col in 0..BUFFER_WIDTH {
                    *vga.add(row_base + col * 2) = *sb.add(sb_base + col * 2);
                    *vga.add(row_base + col * 2 + 1) = *sb.add(sb_base + col * 2 + 1);
                }
            } else {
                for col in 0..BUFFER_WIDTH {
                    *vga.add(row_base + col * 2) = b' ';
                    *vga.add(row_base + col * 2 + 1) = 0x07;
                }
            }
        }
        // Строка 24: статус-бар
        let row_base = (BUFFER_HEIGHT - 1) * BUFFER_WIDTH * 2;
        for col in 0..BUFFER_WIDTH {
            *vga.add(row_base + col * 2) = b' ';
            *vga.add(row_base + col * 2 + 1) = 0x70; // Black on LightGray
        }
        let msg = b" PgUp/PgDn  Esc=back";
        for (i, &ch) in msg.iter().enumerate() {
            *vga.add(row_base + i * 2) = ch;
        }
    }
}
