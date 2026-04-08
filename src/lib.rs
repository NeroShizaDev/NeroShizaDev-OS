#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod font;
pub mod fpu;
pub mod gdt;
pub mod interrupts;
pub mod logo;
pub mod memory;
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
// ГЛАВНАЯ ЛОГИКА — Unicode Intent Engine
// Никакого core::str. Только [u32]. Математика, а не филология.
// ============================================================
pub fn handle_keyboard_input(c: char) {
    unsafe {
        if c == '\n' {
            println!("");

            if INDEX > 0 {
                // Суперпоиск: буфер u32 → Intent за микросекунды
                let intent = unicode::lookup_intent(&BUFFER[..INDEX]);

                match intent {
                    unicode::Intent::Exit => {
                        println!("NeroShiza: Понял. Сваливаю...");
                        let mut port = x86_64::instructions::port::Port::new(0x64);
                        port.write(0xfeu8);
                    }
                    unicode::Intent::Help => {
                        println!("=== NeroShiza Unicode Engine ===");
                        println!("Unicode 17.0 / UTF-32 / UCS-4");
                        println!("Блоков:     {}", unicode_blocks::block_count());
                        println!("Скриптов:   {}", unicode_scripts::script_count());
                        println!("Символов:   {}", unicode_categories::total_defined_chars());
                        println!("Диапазонов: {}", unicode_categories::category_range_count());
                        println!("Словарь:    {} команд ({} байт)",
                            unicode::dict_size(), unicode::dict_bytes());
                        println!("Команды (любой язык):");
                        println!("  выход/exit/свали/cdfkb  - Сброс");
                        println!("  помощь/help/?           - Помощь");
                        println!("  очистить/cls/clear      - Очистка");
                        println!("  статус/status           - Статус");
                        println!("  ребут/reboot            - Ребут");
                    }
                    unicode::Intent::Clear => {
                        vga_buffer::clear_screen();
                    }
                    unicode::Intent::Status => {
                        println!("=== Статус ядра ===");
                        println!("Unicode Engine: UTF-32 / UCS-4 (v17.0)");
                        println!("Кодпоинт = 32 бит. Всегда. Везде.");
                        println!("Блоков:     {} (полная карта)", unicode_blocks::block_count());
                        println!("Скриптов:   {} (все языки)", unicode_scripts::script_count());
                        println!("Символов:   {}", unicode_categories::total_defined_chars());
                        println!("Словарь:    {} намерений", unicode::dict_size());
                        println!("Буфер:      [u32; 64] = {} байт", 64 * 4);
                        rtc::display_status();
                    }
                    unicode::Intent::Reboot => {
                        println!("NeroShiza: Перезагрузка...");
                        let mut port = x86_64::instructions::port::Port::new(0x64);
                        port.write(0xfeu8);
                    }
                    unicode::Intent::Unknown => {
                        // Показать: кодпоинт + блок + скрипт + категория
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

            // Сброс буфера
            INDEX = 0;
            for i in 0..64 { BUFFER[i] = 0; }
            print!("> ");
        } else if c == '\x08' {
            // Backspace: стираем последний u32 кодпоинт
            if INDEX > 0 {
                INDEX -= 1;
                BUFFER[INDEX] = 0;
                print!("\x08 \x08");
            }
        } else if INDEX < 63 {
            // Записываем кодпоинт напрямую — char as u32
            BUFFER[INDEX] = unicode::char_to_codepoint(c);
            INDEX += 1;
            print!("{}", c);
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
