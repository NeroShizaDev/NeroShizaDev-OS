// ============================================================
// COMMON HW — мост между играми и ядром NeroShizaDev-OS
// ============================================================
// Предоставляет все примитивы, необходимые играм:
//   VGA (write_vga_cell, clear_screen, print*, sync_cursor)
//   Speaker (inb/outb/beep)
//   Input  (read_key_blocking)
//   Math   (rdtsc, rand_range, delay_cycles)
// ============================================================

use x86_64::instructions::hlt;

// ============================================================
// VGA размеры
// ============================================================
pub const VGA_WIDTH: usize = 80;
pub const VGA_HEIGHT: usize = 25;

// Текущая позиция «курсора» для print-функций
static mut VGA_COL: usize = 0;
static mut VGA_ROW: usize = 0;

/// Записывает одну ячейку в VGA text buffer (прямой доступ к 0xB8000).
///
/// # Safety
/// VGA text buffer identity-mapped загрузчиком. x, y должны быть в [0,VGA_WIDTH) × [0,VGA_HEIGHT).
pub unsafe fn write_vga_cell(x: usize, y: usize, byte: u8, color: u8) {
    if x >= VGA_WIDTH || y >= VGA_HEIGHT {
        return;
    }
    let vga = 0xB8000 as *mut u8;
    let offset = (y * VGA_WIDTH + x) * 2;
    *vga.add(offset) = byte;
    *vga.add(offset + 1) = color;
}

/// Заполняет весь экран пробелами, сбрасывает позицию курсора.
///
/// # Safety
/// Прямая запись в VGA buffer. Атрибут «цвет» применяется ко всем ячейкам.
pub unsafe fn clear_screen(color: u8) {
    let vga = 0xB8000 as *mut u8;
    for i in 0..(VGA_WIDTH * VGA_HEIGHT) {
        *vga.add(i * 2) = b' ';
        *vga.add(i * 2 + 1) = color;
    }
    VGA_COL = 0;
    VGA_ROW = 0;
}

/// Прокрутка экрана вверх на одну строку.
unsafe fn scroll_up() {
    let vga = 0xB8000 as *mut u8;
    for row in 0..(VGA_HEIGHT - 1) {
        for col in 0..VGA_WIDTH {
            let src = ((row + 1) * VGA_WIDTH + col) * 2;
            let dst = (row * VGA_WIDTH + col) * 2;
            *vga.add(dst) = *vga.add(src);
            *vga.add(dst + 1) = *vga.add(src + 1);
        }
    }
    for col in 0..VGA_WIDTH {
        let offset = ((VGA_HEIGHT - 1) * VGA_WIDTH + col) * 2;
        *vga.add(offset) = b' ';
        *vga.add(offset + 1) = 0x07;
    }
    if VGA_ROW > 0 {
        VGA_ROW -= 1;
    }
}

/// Выводит один байт (символ) с цветом в текущую позицию.
pub unsafe fn put_byte(b: u8, color: u8) {
    if b == b'\n' {
        VGA_COL = 0;
        VGA_ROW += 1;
        if VGA_ROW >= VGA_HEIGHT {
            scroll_up();
        }
        return;
    }
    write_vga_cell(VGA_COL, VGA_ROW, b, color);
    VGA_COL += 1;
    if VGA_COL >= VGA_WIDTH {
        VGA_COL = 0;
        VGA_ROW += 1;
        if VGA_ROW >= VGA_HEIGHT {
            scroll_up();
        }
    }
}

/// Выводит строку байтов.
pub unsafe fn print(s: &str, color: u8) {
    for b in s.as_bytes() {
        put_byte(*b, color);
    }
}

/// Выводит строку с переводом строки.
pub unsafe fn print_line(s: &str, color: u8) {
    print(s, color);
    put_byte(b'\n', color);
}

/// Выводит u32 в десятичном виде.
pub unsafe fn print_u32(n: u32, color: u8) {
    if n == 0 {
        put_byte(b'0', color);
        return;
    }
    let mut buf = [0u8; 10];
    let mut i = 10usize;
    let mut v = n;
    while v > 0 {
        i -= 1;
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
    }
    for j in i..10 {
        put_byte(buf[j], color);
    }
}

/// Выводит u64 в десятичном виде.
pub unsafe fn print_u64(n: u64, color: u8) {
    if n == 0 {
        put_byte(b'0', color);
        return;
    }
    let mut buf = [0u8; 20];
    let mut i = 20usize;
    let mut v = n;
    while v > 0 {
        i -= 1;
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
    }
    for j in i..20 {
        put_byte(buf[j], color);
    }
}

/// Обновляет аппаратный курсор VGA.
pub unsafe fn sync_cursor() {
    let pos = (VGA_ROW * VGA_WIDTH + VGA_COL) as u16;
    crate::vga_hw::write_reg(0x3D4, 0x3D5, 0x0F, (pos & 0xFF) as u8);
    crate::vga_hw::write_reg(0x3D4, 0x3D5, 0x0E, ((pos >> 8) & 0xFF) as u8);
}

// ============================================================
// Псевдо-случайные числа (LCG)
// ============================================================
static mut RNG_STATE: u64 = 0xDEAD_BEEF_1337_CAFE;

/// Возвращает псевдослучайное число в диапазоне [0, n).
pub unsafe fn rand_range(n: u32) -> u32 {
    RNG_STATE = RNG_STATE
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    (RNG_STATE >> 33) as u32 % n
}

// ============================================================
// TSC / delay
// ============================================================
/// Читает Time Stamp Counter (RDTSC).
#[inline(always)]
pub unsafe fn rdtsc() -> u64 {
    let lo: u32;
    let hi: u32;
    core::arch::asm!(
        "rdtsc",
        out("eax") lo,
        out("edx") hi,
        options(nostack, nomem)
    );
    ((hi as u64) << 32) | (lo as u64)
}

/// Ожидает N TSC-тиков (busy-wait).
pub unsafe fn delay_cycles(n: u64) {
    let start = rdtsc();
    loop {
        if rdtsc().wrapping_sub(start) >= n {
            break;
        }
    }
}

// ============================================================
// x87 FPU демо (используется ClickerGame для proof-of-concept)
// ============================================================
pub unsafe fn x87_fyl2x_demo(x: f32, y: f32) -> f32 {
    // Используем x87 fyl2x: ST = y · log2(x)
    let mut result: f32 = 0.0;
    core::arch::asm!(
        "fld   dword ptr [{y}]",   // ST = y
        "fld   dword ptr [{x}]",   // ST = x, ST(1) = y
        "fyl2x",                    // ST = y·log2(x)
        "fstp  dword ptr [{r}]",
        x  = in(reg) &x,
        y  = in(reg) &y,
        r  = in(reg) &mut result,
        options(nostack)
    );
    result
}

// ============================================================
// I/O порты
// ============================================================
#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    core::arch::asm!("in al, dx", out("al") val, in("dx") port, options(nostack, nomem));
    val
}

#[inline(always)]
pub unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val, options(nostack, nomem));
}

// ============================================================
// PC Speaker
// ============================================================
pub unsafe fn speaker_on() {
    let val = inb(0x61);
    if (val & 0x03) != 0x03 {
        outb(0x61, val | 0x03);
    }
}

pub unsafe fn speaker_off() {
    let val = inb(0x61);
    outb(0x61, val & !0x03);
}

pub unsafe fn speaker_set_freq(freq: u32) {
    if freq == 0 {
        speaker_off();
        return;
    }
    let div = 1_193_180 / freq;
    outb(0x43, 0xB6);
    outb(0x42, (div & 0xFF) as u8);
    outb(0x42, ((div >> 8) & 0xFF) as u8);
}

pub unsafe fn beep(freq: u32, cycles: u64) {
    speaker_set_freq(freq);
    speaker_on();
    delay_cycles(cycles);
    speaker_off();
}

pub unsafe fn menu_move_beep() {
    beep(880, 4_000_000);
}

pub unsafe fn menu_launch_beep() {
    beep(660, 6_000_000);
    delay_cycles(2_000_000);
    beep(990, 8_000_000);
}

// ============================================================
// PS/2 клавиатура
// ============================================================
/// Специальные коды для стрелок (расширенные scancodes 0xe0+).
/// Используются в read_key_blocking_ext().
pub const KEY_UP: u8 = 0xE8; // ↑
pub const KEY_DOWN: u8 = 0xE9; // ↓
pub const KEY_LEFT: u8 = 0xEA; // ←
pub const KEY_RIGHT: u8 = 0xEB; // →

/// Блокирующее чтение одного ASCII-символа с клавиатуры.
/// Игнорирует события отпускания клавиш (sc & 0x80).
pub unsafe fn read_key_blocking() -> u8 {
    loop {
        match read_key_blocking_ext() {
            KEY_UP => return b'w',
            KEY_DOWN => return b's',
            KEY_LEFT => return b'a',
            KEY_RIGHT => return b'd',
            0 => {}
            ascii => return ascii,
        }
    }
}

/// Блокирующее чтение клавиши с поддержкой стрелок.
/// Стрелки возвращают KEY_UP / KEY_DOWN / KEY_LEFT / KEY_RIGHT.
/// Остальные клавиши — ASCII как в read_key_blocking().
pub unsafe fn read_key_blocking_ext() -> u8 {
    loop {
        if crate::ps2::has_scancode() {
            let sc = crate::ps2::read_scancode();
            if sc & 0x80 != 0 {
                continue;
            } // key release
            if sc == 0xe0 {
                // ждём следующий байт extended scancode
                loop {
                    if crate::ps2::has_scancode() {
                        let sc2 = crate::ps2::read_scancode();
                        if sc2 & 0x80 != 0 {
                            continue;
                        }
                        return match sc2 {
                            0x48 => KEY_UP,
                            0x50 => KEY_DOWN,
                            0x4B => KEY_LEFT,
                            0x4D => KEY_RIGHT,
                            _ => 0,
                        };
                    }
                    hlt();
                }
            }
            let ascii = scancode_to_ascii(sc);
            if ascii != 0 {
                return ascii;
            }
        }
        hlt();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameInput {
    None,
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Back,
    Char(u8),
}

/// Неблокирующее чтение ввода для игровых тиков.
pub unsafe fn poll_input() -> GameInput {
    if !crate::ps2::has_scancode() {
        return GameInput::None;
    }

    let sc = crate::ps2::read_scancode();
    if sc & 0x80 != 0 {
        return GameInput::None;
    }

    // Extended scancode prefix — читаем второй байт сразу
    if sc == 0xe0 {
        // poll второй байт если уже есть
        if crate::ps2::has_scancode() {
            let sc2 = crate::ps2::read_scancode();
            if sc2 & 0x80 != 0 {
                return GameInput::None;
            }
            return match sc2 {
                0x48 => GameInput::Up,
                0x50 => GameInput::Down,
                0x4B => GameInput::Left,
                0x4D => GameInput::Right,
                _ => GameInput::None,
            };
        }
        return GameInput::None;
    }

    match sc {
        0x01 | 0x10 => GameInput::Back,    // Esc or Q
        0x11 | 0x48 => GameInput::Up,      // W or ↑
        0x1F | 0x50 => GameInput::Down,    // S or ↓
        0x1E | 0x4B => GameInput::Left,    // A or ←
        0x20 | 0x4D => GameInput::Right,   // D or →
        0x1C | 0x39 => GameInput::Confirm, // Enter or Space
        _ => {
            let ascii = scancode_to_ascii(sc);
            if ascii == 0 {
                GameInput::None
            } else {
                GameInput::Char(ascii)
            }
        }
    }
}

pub fn is_back_input(input: GameInput) -> bool {
    matches!(input, GameInput::Back | GameInput::Char(b'q'))
}

/// Унифицированный старт кадра: очистка и рамка вокруг игрового поля.
pub unsafe fn draw_frame_begin(bg_color: u8, border_color: u8) {
    clear_screen(bg_color);
    draw_border(border_color);
}

/// Рамка в стиле launcher, чтобы игры выглядели единообразно.
pub unsafe fn draw_border(color: u8) {
    for x in 0..VGA_WIDTH {
        write_vga_cell(x, 0, b'=', color);
        write_vga_cell(x, VGA_HEIGHT - 1, b'=', color);
    }
    for y in 0..VGA_HEIGHT {
        write_vga_cell(0, y, b'|', color);
        write_vga_cell(VGA_WIDTH - 1, y, b'|', color);
    }
    write_vga_cell(0, 0, b'+', color);
    write_vga_cell(VGA_WIDTH - 1, 0, b'+', color);
    write_vga_cell(0, VGA_HEIGHT - 1, b'+', color);
    write_vga_cell(VGA_WIDTH - 1, VGA_HEIGHT - 1, b'+', color);
}

fn scancode_to_ascii(sc: u8) -> u8 {
    match sc {
        0x01 => b'\x1B', // Esc
        // Цифры
        0x02 => b'1',
        0x03 => b'2',
        0x04 => b'3',
        0x05 => b'4',
        0x06 => b'5',
        0x07 => b'6',
        0x08 => b'7',
        0x09 => b'8',
        0x0A => b'9',
        0x0B => b'0',
        // Enter / Space
        0x1C => b'\n',
        0x39 => b' ',
        // Верхний ряд
        0x10 => b'q',
        0x11 => b'w',
        0x12 => b'e',
        0x13 => b'r',
        0x14 => b't',
        0x15 => b'y',
        0x16 => b'u',
        0x17 => b'i',
        0x18 => b'o',
        0x19 => b'p',
        // Средний ряд
        0x1E => b'a',
        0x1F => b's',
        0x20 => b'd',
        0x21 => b'f',
        0x22 => b'g',
        0x23 => b'h',
        0x24 => b'j',
        0x25 => b'k',
        0x26 => b'l',
        // Нижний ряд
        0x2C => b'z',
        0x2D => b'x',
        0x2E => b'c',
        0x2F => b'v',
        0x30 => b'b',
        0x31 => b'n',
        0x32 => b'm',
        _ => 0,
    }
}
