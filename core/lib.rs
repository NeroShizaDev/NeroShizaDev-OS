#![no_std]
#![cfg_attr(all(test, target_os = "none"), no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![cfg_attr(all(test, target_os = "none"), reexport_test_harness_main = "test_main")]

// ================================================================
// ГЛОБАЛЬНЫЕ АТРИБУТЫ ЛИНТОВ — только реальные ошибки, без шума
// ================================================================
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(dead_code)]
// Стиль: в no_std ядре имена, касты и паттерны нестандартны намеренно
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(unused_labels)]
#![allow(unreachable_code)]
#![allow(unreachable_patterns)]
#![allow(redundant_semicolons)]
// Clippy: подавляем косметику, оставляем correctness
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::needless_return)]
#![allow(clippy::redundant_field_names)]

extern crate alloc;

// ================================================================
// ОБЪЯВЛЕНИЕ ВНЕШНИХ ГРУПП МОДУЛЕЙ
// Файлы расположены вне core/, поэтому указываем пути явно.
// Rust resolves submodules relative to the #[path] file's directory,
// поэтому vga/mod.rs → pub mod vga_buffer; найдёт vga/vga_buffer.rs.
// ================================================================
#[path = "../vga/mod.rs"]
pub mod vga;

#[path = "../fonts/mod.rs"]
pub mod fonts;

/// crate::shell = apps/shell/shell.rs (глобальное состояние шелла, ISR-хуки)
#[path = "../apps/shell/shell.rs"]
pub mod shell;

/// crate::logo = apps/shell/logo.rs (загрузочный логотип)
#[path = "../apps/shell/logo.rs"]
pub mod logo;

/// crate::doom = doom/mod.rs (Doom-подсистема)
#[path = "../apps/doom/mod.rs"]
pub mod doom;

/// crate::apps = apps/mod.rs (лаунчер программ: Doom/Games/Jackal)
#[path = "../apps/mod.rs"]
pub mod apps;

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
pub use apps::fpu;

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
pub mod irq_guard;
pub mod port_firewall;
pub use apps::rtc;
pub use apps::chronos;
pub use apps::rng;

// ================================================================
// ФАЗА 5: Железо и Ввод
// ps2 → beeper
// ps2: клавиатура/мышь. beeper: OK-сигнал после инициализации.
// ================================================================
pub mod ps2;
pub use apps::beeper;

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
pub mod kernel_messages;
pub mod user_messages;

// ================================================================
// ФАЗА 7: Пространство пользователя и Приложения
// shell, logo, doom — объявлены выше с #[path].
// menger, voodoo_math — из moduls/.
// ================================================================
pub use apps::menger;
pub mod voodoo_engine;
pub use crate::voodoo_engine::demo_cellular_automaton;
// Compatibility alias: call sites using crate::voodoo_math still work
pub use crate::voodoo_engine as voodoo_math;

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
    loop {
        x86_64::instructions::hlt();
        shell::process_deferred_actions();
    }
}

#[cfg(all(test, target_os = "none"))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[cfg(all(test, target_os = "none"))]
mod test_harness {
    unsafe extern "Rust" {
        pub fn test_main();
    }
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

#[cfg(all(test, target_os = "none"))]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    init();
    unsafe { test_harness::test_main(); }
    hlt_loop();
}
