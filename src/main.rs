#![no_std]
#![no_main]

use core::panic::PanicInfo;
use blog_os::println;

// ТВОЙ КОМПИЛЯТОР ТРЕБУЕТ ИМЕННО ТАК:
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    blog_os::init();
    blog_os::vga_buffer::clear_screen();
    blog_os::logo::show_boot_logo();
    
    println!("=== NeroShizaDev OS v0.2 ===");
    println!("Unicode 17.0 / UTF-32 / UCS-4");
    println!("Блоков: {} | Скриптов: {} | Символов: {}",
        blog_os::unicode_blocks::block_count(),
        blog_os::unicode_scripts::script_count(),
        blog_os::unicode_categories::total_defined_chars());
    println!("Словарь: {} намерений ({} байт)",
        blog_os::unicode::dict_size(),
        blog_os::unicode::dict_bytes());
    blog_os::fpu::demo();
    blog_os::rng::demo();
    blog_os::rtc::display_status();
    println!("Система готова. Введи 'помощь' или 'help'");
    blog_os::print!("> ");

    blog_os::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::println!("{}", info);
    blog_os::hlt_loop();
}
