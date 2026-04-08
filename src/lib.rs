#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod beeper;
pub mod chronos;
pub mod font;
pub mod fpu;
pub mod gdt;
pub mod interrupts;
pub mod logo;
pub mod memory;
pub mod menger;
pub mod rng;
pub mod rtc;
pub mod serial;
pub mod unicode;
pub mod unicode_blocks;
pub mod unicode_categories;
pub mod unicode_scripts;
pub mod vga_buffer;

use core::panic::PanicInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

// Инициализация системы
pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    unsafe { font::load_cyrillic_font() };
    fpu::init();
    x86_64::instructions::interrupts::enable();
}

// ============================================================
// UTF-32 БУФЕР ВВОДА — каждый символ = 4 байта, всегда
// ============================================================
static mut BUFFER: [u32; 64] = [0; 64];
static mut INDEX: usize = 0;

// ============================================================
// СОСТОЯНИЕ: язык, подтверждение, хоткеи
// ============================================================
static mut LANG_RUS: bool = false;
static mut CONFIRM_PENDING: bool = false;
static mut ALT_HELD: bool = false;
static mut CTRL_HELD: bool = false;
static mut SHIFT_HELD: bool = false;

// Курсор и выделение
static mut CURSOR: usize = 0;
static mut SEL_ACTIVE: bool = false;
static mut SEL_START: usize = 0;
static mut SEL_END: usize = 0;

// Буфер обмена
static mut CLIPBOARD: [u32; 64] = [0; 64];
static mut CLIP_LEN: usize = 0;

// Хоткеи F1-F12: 12 слотов × 64 кодпоинта
static mut HOTKEYS: [[u32; 64]; 12] = [[0; 64]; 12];
static mut HOTKEY_LEN: [usize; 12] = [0; 12];
// Режим записи: None = обычный, Some(n) = записываем команду для F(n+1)
static mut RECORDING_SLOT: Option<usize> = None;

// История команд — кольцевой буфер на 32 команды
static mut HISTORY: [[u32; 64]; 32] = [[0; 64]; 32];
static mut HIST_LEN: [usize; 32] = [0; 32];
static mut HIST_COUNT: usize = 0;
static mut HIST_IDX: usize = 0;
static mut HIST_NAV: isize = -1;
static mut HIST_SAVE: [u32; 64] = [0; 64];
static mut HIST_SAVE_LEN: usize = 0;
static mut HIST_SAVE_CUR: usize = 0;

// Статистика: 0=Exit,1=Help,2=Clear,3=Status,4=Reboot,5=Menger,6=Beep,7=Time,8=Unknown
static mut CMD_STATS: [u32; 9] = [0; 9];
static mut CMD_TOTAL: u32 = 0;

// Режим прокрутки
static mut SCROLL_MODE: bool = false;
static mut SCROLL_OFFSET: usize = 0;

// ============================================================
// QWERTY → ЙЦУКЕН маппинг
// ============================================================
fn qwerty_to_russian(c: char) -> char {
    match c {
        'q' => 'й', 'w' => 'ц', 'e' => 'у', 'r' => 'к', 't' => 'е',
        'y' => 'н', 'u' => 'г', 'i' => 'ш', 'o' => 'щ', 'p' => 'з',
        '[' => 'х', ']' => 'ъ',
        'a' => 'ф', 's' => 'ы', 'd' => 'в', 'f' => 'а', 'g' => 'п',
        'h' => 'р', 'j' => 'о', 'k' => 'л', 'l' => 'д', ';' => 'ж',
        '\'' => 'э',
        'z' => 'я', 'x' => 'ч', 'c' => 'с', 'v' => 'м', 'b' => 'и',
        'n' => 'т', 'm' => 'ь', ',' => 'б', '.' => 'ю',
        'Q' => 'Й', 'W' => 'Ц', 'E' => 'У', 'R' => 'К', 'T' => 'Е',
        'Y' => 'Н', 'U' => 'Г', 'I' => 'Ш', 'O' => 'Щ', 'P' => 'З',
        '{' => 'Х', '}' => 'Ъ',
        'A' => 'Ф', 'S' => 'Ы', 'D' => 'В', 'F' => 'А', 'G' => 'П',
        'H' => 'Р', 'J' => 'О', 'K' => 'Л', 'L' => 'Д', ':' => 'Ж',
        '"' => 'Э',
        'Z' => 'Я', 'X' => 'Ч', 'C' => 'С', 'V' => 'М', 'B' => 'И',
        'N' => 'Т', 'M' => 'Ь', '<' => 'Б', '>' => 'Ю',
        '`' => 'ё', '~' => 'Ё',
        _ => c,
    }
}

// ============================================================
// VGA КУРСОР — аппаратный мигающий курсор
// ============================================================
fn set_vga_cursor(row: usize, col: usize) {
    let pos: u16 = (row * 80 + col) as u16;
    unsafe {
        let mut cmd = x86_64::instructions::port::Port::<u8>::new(0x3D4);
        let mut data = x86_64::instructions::port::Port::<u8>::new(0x3D5);
        cmd.write(0x0F); data.write((pos & 0xFF) as u8);
        cmd.write(0x0E); data.write(((pos >> 8) & 0xFF) as u8);
    }
}

// ============================================================
// ПЕРЕРИСОВКА СТРОКИ ВВОДА — прямой VGA доступ
// Рисуем BUFFER[0..INDEX] на строке 24 начиная с колонки 2
// Подсветка выделения: Black on Yellow (0xE0)
// ============================================================
fn redraw_input() {
    unsafe {
        let vga = 0xB8000 as *mut u8;
        let row = 24usize;
        let prompt_col = 2usize; // после "> "
        let normal: u8 = 0x0E; // Yellow on Black
        let sel_color: u8 = 0xE0; // Black on Yellow (инверсия)

        // Рисуем каждый символ буфера
        for i in 0..INDEX {
            let cp = BUFFER[i];
            let ch = char::from_u32(cp).unwrap_or('?');
            let vga_byte = if let Some(code) = font::cyrillic_to_vga(ch) {
                code
            } else if cp < 128 {
                cp as u8
            } else {
                b'?'
            };

            let col = prompt_col + i;
            if col >= 80 { break; }

            let color = if SEL_ACTIVE && i >= SEL_START && i < SEL_END {
                sel_color
            } else {
                normal
            };

            let offset = (row * 80 + col) * 2;
            *vga.add(offset) = vga_byte;
            *vga.add(offset + 1) = color;
        }

        // Очищаем остаток строки после буфера
        for i in INDEX..78 {
            let col = prompt_col + i;
            if col >= 80 { break; }
            let offset = (row * 80 + col) * 2;
            *vga.add(offset) = b' ';
            *vga.add(offset + 1) = normal;
        }

        // Ставим аппаратный курсор
        set_vga_cursor(row, prompt_col + CURSOR);

        // Синхронизируем Writer.column_position
        x86_64::instructions::interrupts::without_interrupts(|| {
            vga_buffer::WRITER.lock().column_position = prompt_col + INDEX;
        });
    }
}

// ============================================================
// УДАЛЕНИЕ ВЫДЕЛЕНИЯ
// ============================================================
fn delete_selection() {
    unsafe {
        if !SEL_ACTIVE { return; }
        let start = SEL_START;
        let end = SEL_END;
        let len = end - start;
        if len == 0 { SEL_ACTIVE = false; return; }

        // Сдвиг влево
        let mut i = start;
        while i + len < INDEX {
            BUFFER[i] = BUFFER[i + len];
            i += 1;
        }
        // Обнуляем хвост
        while i < INDEX {
            BUFFER[i] = 0;
            i += 1;
        }
        INDEX -= len;
        CURSOR = start;
        SEL_ACTIVE = false;
    }
}

// ============================================================
// CLIPBOARD: copy / paste / cut / select_all
// ============================================================
fn do_copy() {
    unsafe {
        if SEL_ACTIVE && SEL_END > SEL_START {
            let len = SEL_END - SEL_START;
            for i in 0..len {
                CLIPBOARD[i] = BUFFER[SEL_START + i];
            }
            CLIP_LEN = len;
        } else if INDEX > 0 {
            // Копируем весь буфер
            for i in 0..INDEX {
                CLIPBOARD[i] = BUFFER[i];
            }
            CLIP_LEN = INDEX;
        }
    }
}

fn do_paste() {
    unsafe {
        if CLIP_LEN == 0 { return; }
        // Удаляем выделение если есть
        if SEL_ACTIVE { delete_selection(); }

        // Сколько влезет
        let space = 63 - INDEX;
        let paste_len = CLIP_LEN.min(space);
        if paste_len == 0 { return; }

        // Сдвиг вправо от CURSOR
        let mut i = INDEX;
        while i > CURSOR {
            BUFFER[i + paste_len - 1] = BUFFER[i - 1];
            i -= 1;
        }

        // Вставка
        for i in 0..paste_len {
            BUFFER[CURSOR + i] = CLIPBOARD[i];
        }
        INDEX += paste_len;
        CURSOR += paste_len;
        redraw_input();
    }
}

fn do_cut() {
    unsafe {
        if SEL_ACTIVE && SEL_END > SEL_START {
            do_copy();
            delete_selection();
            redraw_input();
        }
    }
}

fn do_select_all() {
    unsafe {
        if INDEX > 0 {
            SEL_ACTIVE = true;
            SEL_START = 0;
            SEL_END = INDEX;
            redraw_input();
        }
    }
}

// ============================================================
// ИСТОРИЯ КОМАНД
// ============================================================
fn push_history() {
    unsafe {
        if INDEX == 0 { return; }
        let idx = HIST_IDX;
        for i in 0..INDEX {
            core::ptr::write_volatile(&raw mut HISTORY[idx][i], BUFFER[i]);
        }
        for i in INDEX..64 {
            core::ptr::write_volatile(&raw mut HISTORY[idx][i], 0);
        }
        HIST_LEN[idx] = INDEX;
        HIST_IDX = (idx + 1) % 32;
        if HIST_COUNT < 32 { HIST_COUNT += 1; }
        HIST_NAV = -1;
    }
}

fn history_up() {
    unsafe {
        if HIST_COUNT == 0 { return; }
        if HIST_NAV == -1 {
            // Сохраняем текущий ввод
            for i in 0..INDEX { HIST_SAVE[i] = BUFFER[i]; }
            for i in INDEX..64 { HIST_SAVE[i] = 0; }
            HIST_SAVE_LEN = INDEX;
            HIST_SAVE_CUR = CURSOR;
            HIST_NAV = 0;
        } else if (HIST_NAV as usize) < HIST_COUNT - 1 {
            HIST_NAV += 1;
        } else {
            return;
        }
        // Загружаем из истории
        let ring_idx = (HIST_IDX + 32 - 1 - HIST_NAV as usize) % 32;
        let len = HIST_LEN[ring_idx];
        for i in 0..len {
            BUFFER[i] = core::ptr::read_volatile(&raw const HISTORY[ring_idx][i]);
        }
        for i in len..64 { BUFFER[i] = 0; }
        INDEX = len;
        CURSOR = len;
        SEL_ACTIVE = false;
        redraw_input();
    }
}

fn history_down() {
    unsafe {
        if HIST_NAV < 0 { return; }
        HIST_NAV -= 1;
        if HIST_NAV < 0 {
            // Восстанавливаем сохранённый ввод
            for i in 0..HIST_SAVE_LEN { BUFFER[i] = HIST_SAVE[i]; }
            for i in HIST_SAVE_LEN..64 { BUFFER[i] = 0; }
            INDEX = HIST_SAVE_LEN;
            CURSOR = HIST_SAVE_CUR;
        } else {
            let ring_idx = (HIST_IDX + 32 - 1 - HIST_NAV as usize) % 32;
            let len = HIST_LEN[ring_idx];
            for i in 0..len {
                BUFFER[i] = core::ptr::read_volatile(&raw const HISTORY[ring_idx][i]);
            }
            for i in len..64 { BUFFER[i] = 0; }
            INDEX = len;
            CURSOR = len;
        }
        SEL_ACTIVE = false;
        redraw_input();
    }
}

// ============================================================
// СКРОЛЛБЭК — PageUp/PageDown
// ============================================================
fn enter_scroll_mode() {
    unsafe {
        if SCROLL_MODE { return; }
        let total = vga_buffer::scroll_total();
        if total == 0 { return; }
        vga_buffer::save_screen();
        SCROLL_MODE = true;
        SCROLL_OFFSET = 0;
        vga_buffer::show_scrollback(SCROLL_OFFSET);
    }
}

fn scroll_page_up() {
    unsafe {
        let total = vga_buffer::scroll_total();
        if SCROLL_OFFSET + 12 < total {
            SCROLL_OFFSET += 12;
        } else if total > 0 {
            SCROLL_OFFSET = total - 1;
        }
        vga_buffer::show_scrollback(SCROLL_OFFSET);
    }
}

fn scroll_page_down() {
    unsafe {
        if SCROLL_OFFSET >= 12 {
            SCROLL_OFFSET -= 12;
            vga_buffer::show_scrollback(SCROLL_OFFSET);
        } else {
            exit_scroll_mode();
        }
    }
}

fn exit_scroll_mode() {
    unsafe {
        if !SCROLL_MODE { return; }
        SCROLL_MODE = false;
        vga_buffer::restore_screen();
        redraw_input();
    }
}

// ============================================================
// ОБРАБОТКА RawKey (CapsLock, ScrollLock, PauseBreak, стрелки, Home/End)
// ============================================================
pub fn handle_raw_key(key: pc_keyboard::KeyCode) {
    use pc_keyboard::KeyCode;
    unsafe {
        // Скроллбэк: только PgUp/PgDn, остальное — выход
        if SCROLL_MODE {
            match key {
                KeyCode::PageUp => { scroll_page_up(); return; }
                KeyCode::PageDown => { scroll_page_down(); return; }
                _ => { exit_scroll_mode(); return; }
            }
        }

        match key {
            KeyCode::CapsLock => {
                if INDEX > 0 && !CONFIRM_PENDING {
                    CONFIRM_PENDING = true;
                    println!("");
                    println!("[Enter=Da / Esc=Otmena]");
                }
            }
            KeyCode::ScrollLock => {
                LANG_RUS = !LANG_RUS;
                let vga = 0xB8000 as *mut u8;
                if LANG_RUS {
                    let text = b"RUS";
                    for (i, &ch) in text.iter().enumerate() {
                        *vga.add((77 + i) * 2) = ch;
                        *vga.add((77 + i) * 2 + 1) = 0x4F;
                    }
                } else {
                    let text = b"ENG";
                    for (i, &ch) in text.iter().enumerate() {
                        *vga.add((77 + i) * 2) = ch;
                        *vga.add((77 + i) * 2 + 1) = 0x2F;
                    }
                }
            }
            KeyCode::PauseBreak => {
                if CONFIRM_PENDING {
                    CONFIRM_PENDING = false;
                    println!("[Otmeneno]");
                    INDEX = 0; CURSOR = 0; SEL_ACTIVE = false;
                    for i in 0..64 { BUFFER[i] = 0; }
                    print!("> ");
                }
            }
            // ← → стрелки + Shift-выделение
            KeyCode::ArrowLeft => {
                if CURSOR > 0 {
                    if SHIFT_HELD {
                        if !SEL_ACTIVE {
                            SEL_ACTIVE = true;
                            SEL_START = CURSOR - 1;
                            SEL_END = CURSOR;
                        } else if SEL_START == CURSOR {
                            SEL_START = CURSOR - 1;
                        } else if SEL_END == CURSOR {
                            SEL_END = CURSOR - 1;
                            if SEL_START == SEL_END { SEL_ACTIVE = false; }
                        }
                    } else {
                        SEL_ACTIVE = false;
                    }
                    CURSOR -= 1;
                    redraw_input();
                }
            }
            KeyCode::ArrowRight => {
                if CURSOR < INDEX {
                    if SHIFT_HELD {
                        if !SEL_ACTIVE {
                            SEL_ACTIVE = true;
                            SEL_START = CURSOR;
                            SEL_END = CURSOR + 1;
                        } else if SEL_END == CURSOR {
                            SEL_END = CURSOR + 1;
                        } else if SEL_START == CURSOR {
                            SEL_START = CURSOR + 1;
                            if SEL_START == SEL_END { SEL_ACTIVE = false; }
                        }
                    } else {
                        SEL_ACTIVE = false;
                    }
                    CURSOR += 1;
                    redraw_input();
                }
            }
            KeyCode::Home => {
                if CURSOR > 0 {
                    if SHIFT_HELD {
                        if !SEL_ACTIVE {
                            SEL_ACTIVE = true;
                            SEL_START = 0;
                            SEL_END = CURSOR;
                        } else {
                            SEL_START = 0;
                        }
                    } else {
                        SEL_ACTIVE = false;
                    }
                    CURSOR = 0;
                    redraw_input();
                }
            }
            KeyCode::End => {
                if CURSOR < INDEX {
                    if SHIFT_HELD {
                        if !SEL_ACTIVE {
                            SEL_ACTIVE = true;
                            SEL_START = CURSOR;
                            SEL_END = INDEX;
                        } else {
                            SEL_END = INDEX;
                        }
                    } else {
                        SEL_ACTIVE = false;
                    }
                    CURSOR = INDEX;
                    redraw_input();
                }
            }
            // ↑↓ история команд
            KeyCode::ArrowUp => { history_up(); }
            KeyCode::ArrowDown => { history_down(); }
            // PageUp/PageDown — прокрутка экрана
            KeyCode::PageUp => { enter_scroll_mode(); }
            KeyCode::PageDown => {} // вне scroll mode — ничего
            // Delete — удалить символ ПОД курсором
            KeyCode::Delete => {
                if SEL_ACTIVE {
                    delete_selection();
                    HIST_NAV = -1;
                    redraw_input();
                } else if CURSOR < INDEX {
                    let mut i = CURSOR;
                    while i + 1 < INDEX {
                        BUFFER[i] = BUFFER[i + 1];
                        i += 1;
                    }
                    BUFFER[INDEX - 1] = 0;
                    INDEX -= 1;
                    HIST_NAV = -1;
                    redraw_input();
                }
            }
            // F1-F12: Alt+Fn = запись, Fn = выполнение
            KeyCode::F1 => handle_fkey(0),
            KeyCode::F2 => handle_fkey(1),
            KeyCode::F3 => handle_fkey(2),
            KeyCode::F4 => handle_fkey(3),
            KeyCode::F5 => handle_fkey(4),
            KeyCode::F6 => handle_fkey(5),
            KeyCode::F7 => handle_fkey(6),
            KeyCode::F8 => handle_fkey(7),
            KeyCode::F9 => handle_fkey(8),
            KeyCode::F10 => handle_fkey(9),
            KeyCode::F11 => handle_fkey(10),
            KeyCode::F12 => handle_fkey(11),
            _ => {}
        }
    }
}

// ============================================================
// ХОТКЕИ F1-F12
// ============================================================
fn handle_fkey(slot: usize) {
    unsafe {
        if ALT_HELD {
            // Alt+Fn: начать/завершить запись
            if let Some(s) = RECORDING_SLOT {
                if s == slot {
                    // Повторное Alt+Fn — сохраняем
                    let len = INDEX.min(64);
                    for i in 0..len {
                        let val = core::ptr::read_volatile(&raw const BUFFER[i]);
                        core::ptr::write_volatile(&raw mut HOTKEYS[slot][i], val);
                    }
                    core::ptr::write_volatile(&raw mut HOTKEY_LEN[slot], len);
                    RECORDING_SLOT = None;
                    println!("");
                    println!("[F{}: Принято! ({} симв.)]", slot + 1, len);
                    reset_buffer();
                    print!("> ");
                    redraw_input();
                    return;
                }
            }
            // Начинаем запись для этого слота
            RECORDING_SLOT = Some(slot);
            reset_buffer();
            println!("");
            println!("[F{}: Введи команду, потом Alt+F{}]", slot + 1, slot + 1);
            print!("F{}> ", slot + 1);
        } else {
            // Fn без Alt: выполнить записанную команду
            let rec = core::ptr::read_volatile(&raw const RECORDING_SLOT);
            if rec.is_some() { return; }
            let len = core::ptr::read_volatile(&raw const HOTKEY_LEN[slot]);
            if len == 0 {
                println!("");
                println!("[F{}: Пусто. Alt+F{} для записи]", slot + 1, slot + 1);
                print!("> ");
                return;
            }
            // Копируем хоткей в буфер и выполняем
            for i in 0..len {
                let val = core::ptr::read_volatile(&raw const HOTKEYS[slot][i]);
                BUFFER[i] = val;
            }
            INDEX = len;
            CURSOR = len;
            print!("[F{}] ", slot + 1);
            handle_keyboard_input('\n');
        }
    }
}

// ============================================================
// СБРОС БУФЕРА + КУРСОР
// ============================================================
fn reset_buffer() {
    unsafe {
        INDEX = 0;
        CURSOR = 0;
        SEL_ACTIVE = false;
        for i in 0..64 { BUFFER[i] = 0; }
    }
}

// ============================================================
// ГЛАВНАЯ ЛОГИКА — Unicode Intent Engine
// Ctrl+C/V/X/A, cursor-aware insert/delete, redraw_input
// ============================================================
pub fn handle_keyboard_input(c: char) {
    unsafe {
        // Выход из скроллбэка на любую клавишу
        if SCROLL_MODE {
            exit_scroll_mode();
            if c == '\x1B' { return; }
        }

        // === Ctrl+key: clipboard ===
        if CTRL_HELD {
            match c {
                'c' => { do_copy(); return; }
                'v' => { do_paste(); return; }
                'x' => { do_cut(); return; }
                'a' => { do_select_all(); return; }
                _ => { return; }
            }
        }

        // === Escape: сброс буфера / отмена подтверждения ===
        if c == '\x1B' {
            if CONFIRM_PENDING {
                CONFIRM_PENDING = false;
                println!("[Otmeneno]");
            }
            reset_buffer();
            print!("> ");
            redraw_input();
            return;
        }

        if c == '\n' {
            if CONFIRM_PENDING {
                CONFIRM_PENDING = false;
            }
            SEL_ACTIVE = false;
            println!("");

            if INDEX > 0 {
                push_history();
                let intent = unicode::lookup_intent(&BUFFER[..INDEX]);

                match intent {
                    unicode::Intent::Exit => {
                        CMD_STATS[0] += 1; CMD_TOTAL += 1;
                        println!("NeroShiza: Понял. Сваливаю...");
                        let mut port = x86_64::instructions::port::Port::new(0x64);
                        port.write(0xfeu8);
                    }
                    unicode::Intent::Help => {
                        CMD_STATS[1] += 1; CMD_TOTAL += 1;
                        println!("=== NeroShiza Unicode Engine ===");
                        println!("Unicode 17.0 / UTF-32 / UCS-4");
                        println!("Блоков:     {}", unicode_blocks::block_count());
                        println!("Скриптов:   {}", unicode_scripts::script_count());
                        println!("Символов:   {}", unicode_categories::total_defined_chars());
                        println!("Диапазонов: {}", unicode_categories::category_range_count());
                        println!("Словарь:    {} команд ({} байт)",
                            unicode::dict_size(), unicode::dict_bytes());
                        println!("Команды (любой язык):");
                        println!("  выход/exit/свали       - Выход");
                        println!("  помощь/help/?          - Помощь");
                        println!("  очистить/cls/clear     - Очистка");
                        println!("  статус/status          - Статус+стата");
                        println!("  ребут/reboot           - Ребут");
                        println!("  губка/menger/fractal   - Губка Менгера");
                        println!("  звук/beep/sound        - 16-нотный бипер");
                        println!("  время/time/часы/триго  - 4 реальности");
                        println!("Навигация:");
                        println!("  ←/→       - Курсор по строке");
                        println!("  ↑/↓       - История команд");
                        println!("  Home/End  - Начало/конец строки");
                        println!("  PgUp/PgDn - Прокрутка экрана");
                        println!("  Delete    - Удалить символ");
                        println!("Выделение и буфер:");
                        println!("  Shift+←/→ - Выделение текста");
                        println!("  Ctrl+A    - Выделить всё");
                        println!("  Ctrl+C/V  - Копировать/Вставить");
                        println!("  Ctrl+X    - Вырезать");
                        println!("Системные:");
                        println!("  Esc       - Сброс ввода");
                        println!("  CapsLock  - Медленный Enter");
                        println!("  ScrollLock- RUS/ENG язык");
                        println!("  Alt+F1..12- Запись хоткея");
                        println!("  F1..F12   - Выполнить хоткей");
                    }
                    unicode::Intent::Clear => {
                        CMD_STATS[2] += 1; CMD_TOTAL += 1;
                        vga_buffer::clear_screen();
                    }
                    unicode::Intent::Status => {
                        CMD_STATS[3] += 1; CMD_TOTAL += 1;
                        println!("=== Статус ядра ===");
                        println!("Unicode Engine: UTF-32 / UCS-4 (v17.0)");
                        println!("Кодпоинт = 32 бит. Всегда. Везде.");
                        println!("Блоков:     {} (полная карта)", unicode_blocks::block_count());
                        println!("Скриптов:   {} (все языки)", unicode_scripts::script_count());
                        println!("Символов:   {}", unicode_categories::total_defined_chars());
                        println!("Словарь:    {} намерений", unicode::dict_size());
                        println!("Буфер:      [u32; 64] = {} байт", 64 * 4);
                        println!("История:    {} команд (макс 32)",
                            core::ptr::read_volatile(&raw const HIST_COUNT));
                        rtc::display_status();
                        // Статистика
                        let ct = core::ptr::read_volatile(&raw const CMD_TOTAL);
                        let s1 = core::ptr::read_volatile(&raw const CMD_STATS[1]);
                        let s2 = core::ptr::read_volatile(&raw const CMD_STATS[2]);
                        let s3 = core::ptr::read_volatile(&raw const CMD_STATS[3]);
                        let s5 = core::ptr::read_volatile(&raw const CMD_STATS[5]);
                        let s6 = core::ptr::read_volatile(&raw const CMD_STATS[6]);
                        let s7 = core::ptr::read_volatile(&raw const CMD_STATS[7]);
                        let s8 = core::ptr::read_volatile(&raw const CMD_STATS[8]);
                        println!("=== Статистика ===");
                        println!("Всего:      {}", ct);
                        println!("  помощь:    {}", s1);
                        println!("  губка:     {}", s5);
                        println!("  звук:      {}", s6);
                        println!("  время:     {}", s7);
                        println!("  очистить:  {}", s2);
                        println!("  статус:    {}", s3);
                        println!("  неизвестно:{}", s8);
                        // Хоткеи
                        let mut has_hotkeys = false;
                        for slot in 0..12usize {
                            let len = core::ptr::read_volatile(&raw const HOTKEY_LEN[slot]);
                            if len > 0 { has_hotkeys = true; break; }
                        }
                        if has_hotkeys {
                            println!("=== Хоткеи ===");
                            for slot in 0..12usize {
                                let len = core::ptr::read_volatile(&raw const HOTKEY_LEN[slot]);
                                if len > 0 {
                                    print!("  F{}: ", slot + 1);
                                    for i in 0..len {
                                        let cp = core::ptr::read_volatile(&raw const HOTKEYS[slot][i]);
                                        if let Some(ch) = char::from_u32(cp) {
                                            print!("{}", ch);
                                        }
                                    }
                                    println!("");
                                }
                            }
                        }
                    }
                    unicode::Intent::Reboot => {
                        CMD_STATS[4] += 1; CMD_TOTAL += 1;
                        println!("NeroShiza: Перезагрузка...");
                        let mut port = x86_64::instructions::port::Port::new(0x64);
                        port.write(0xfeu8);
                    }
                    unicode::Intent::Menger => {
                        CMD_STATS[5] += 1; CMD_TOTAL += 1;
                        menger::run_demo();
                    }
                    unicode::Intent::Beep => {
                        CMD_STATS[6] += 1; CMD_TOTAL += 1;
                        beeper::demo_hex_scale();
                    }
                    unicode::Intent::Time => {
                        CMD_STATS[7] += 1; CMD_TOTAL += 1;
                        chronos::display_triple_time();
                    }
                    unicode::Intent::Unknown => {
                        CMD_STATS[8] += 1; CMD_TOTAL += 1;
                        let first_cp = BUFFER[0];
                        let block = unicode::unicode_block_name(first_cp);
                        let script = unicode::unicode_script_name(first_cp);
                        let cat = unicode::unicode_category(first_cp);
                        println!("NeroShiza: Не понял [U+{:04X}]", first_cp);
                        println!("  Блок:     {}", block);
                        println!("  Скрипт:   {}", script);
                        println!("  Категория: {}", cat.name());
                    }
                }
            }

            reset_buffer();
            print!("> ");
            redraw_input();
        } else if c == '\x08' {
            // Backspace
            HIST_NAV = -1;
            if CONFIRM_PENDING {
                CONFIRM_PENDING = false;
                println!("[Otmeneno]");
                reset_buffer();
                print!("> ");
                redraw_input();
            } else if SEL_ACTIVE {
                // Удаляем выделение
                delete_selection();
                redraw_input();
            } else if CURSOR > 0 {
                // Сдвиг влево от CURSOR
                let mut i = CURSOR - 1;
                while i + 1 < INDEX {
                    BUFFER[i] = BUFFER[i + 1];
                    i += 1;
                }
                BUFFER[INDEX - 1] = 0;
                INDEX -= 1;
                CURSOR -= 1;
                redraw_input();
            }
        } else if INDEX < 63 {
            if CONFIRM_PENDING { return; }
            HIST_NAV = -1;

            // Удаляем выделение если есть
            if SEL_ACTIVE { delete_selection(); }

            // Применяем языковую раскладку
            let mapped = if LANG_RUS { qwerty_to_russian(c) } else { c };
            let cp = unicode::char_to_codepoint(mapped);

            // Сдвиг вправо от CURSOR для вставки в середину
            let mut i = INDEX;
            while i > CURSOR {
                BUFFER[i] = BUFFER[i - 1];
                i -= 1;
            }
            BUFFER[CURSOR] = cp;
            INDEX += 1;
            CURSOR += 1;
            redraw_input();
        }
    }
}

pub fn hlt_loop() -> ! {
    loop { x86_64::instructions::hlt(); }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub trait Testable { fn run(&self); }
impl<T> Testable for T where T: Fn() {
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    hlt_loop();
}
