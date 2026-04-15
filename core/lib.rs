#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(dead_code)]

extern crate alloc;

// ================================================================
// ОБЪЯВЛЕНИЕ ВНЕШНИХ ГРУПП МОДУЛЕЙ
// Файлы расположены вне core/, поэтому указываем пути явно.
// Rust resolves submodules relative to the #[path] file's directory,
// поэтому vga/mod.rs → pub mod vga_buffer; найдёт vga/vga_buffer.rs.
// ================================================================
#[path = "../vga/mod.rs"]
pub mod vga;

#[path = "../demo/mod.rs"]
pub mod demo;

#[path = "../fonts/mod.rs"]
pub mod fonts;

#[path = "../moduls/mod.rs"]
pub mod moduls;

/// crate::shell = shell/shell.rs (глобальное состояние шелла, ISR-хуки)
#[path = "../shell/shell.rs"]
pub mod shell;

/// crate::logo = shell/logo.rs (загрузочный логотип)
#[path = "../shell/logo.rs"]
pub mod logo;

/// crate::doom = doom/mod.rs (Doom-подсистема)
#[path = "../doom/mod.rs"]
pub mod doom;

// ================================================================
// ФАЗА 1: Ранняя инициализация & Линия жизни
// serial → vga_hw → vga_buffer → logo
// Цель: видеть вывод и логи до любой другой инициализации.
// ================================================================
pub mod serial;
pub use vga::vga_hw;
pub use vga::vga_buffer;

// ================================================================
// ФАЗА 2: Архитектура CPU
// gdt → interrupts → fpu
// Без GDT/IDT любое исключение = тройной сброс.
// ================================================================
pub mod gdt;
pub mod interrupts;
pub use demo::fpu;

// ================================================================
// ФАЗА 3: Память
// memory — страничная адресация, аллокатор кучи.
// После этой фазы доступны alloc::vec::Vec, Box и т.д.
// ================================================================
pub mod memory;

// ================================================================
// ФАЗА 4: Время и Энтропия
// validator ПЕРВЫМ — probe перед любым чтением порта.
// rtc → chronos → rng
// ================================================================
pub mod validator;
pub mod trace;
pub use moduls::rtc;
pub use moduls::chronos;
pub use moduls::rng;

// ================================================================
// ФАЗА 5: Железо и Ввод
// ps2 → beeper
// ps2: клавиатура/мышь. beeper: OK-сигнал после инициализации.
// ================================================================
pub mod ps2;
pub use demo::beeper;

// ================================================================
// ФАЗА 6: Мультиязычность и Рендеринг текста
// unicode_* → vga_unicode → locale → kernel_messages
// vga_unicode содержит все шрифты (кириллица, арабский, кеш).
// ================================================================
pub use fonts::unicode;
pub use fonts::unicode_blocks;
pub use fonts::unicode_categories;
pub use fonts::unicode_scripts;
pub use vga::vga_unicode;
pub use fonts::locale;
#[path = "../src/kernel_messages.rs"]
pub mod kernel_messages;
pub mod user_messages;

// ================================================================
// ФАЗА 7: Пространство пользователя и Приложения
// shell, logo, doom — объявлены выше с #[path].
// menger, voodoo_math — из moduls/.
// ================================================================
pub use moduls::menger;
pub use moduls::voodoo_math;
pub use crate::voodoo_math::demo_cellular_automaton;

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

// Фаза 2: инициализация CPU — GDT, IDT, PICS, FPU.
// Шрифты и локаль загружаются позже в Фазе 6 (main.rs).
pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    fpu::init();
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
        let name = core::any::type_name::<T>();
        serial_print!("{}...\t", name);
        // Имя теста на VGA (жёлтый, стандартный цвет)
        for &b in name.as_bytes() { vga_buffer::WRITER.lock().write_byte(b); }
        for &b in b"...\t" { vga_buffer::WRITER.lock().write_byte(b); }
        self();
        serial_println!("[ok]");
        // Зелёный [ok] на VGA
        {
            let mut w = vga_buffer::WRITER.lock();
            let saved = w.color_code;
            w.color_code = vga_buffer::ColorCode::new(
                vga_buffer::Color::LightGreen,
                vga_buffer::Color::Black,
            );
            for &b in b"[ok]\n" { w.write_byte(b); }
            w.color_code = saved;
        }
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
