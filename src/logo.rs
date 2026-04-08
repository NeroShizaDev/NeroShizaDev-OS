// Логотип загрузки — ASCII-арт "no_mangle" в стиле FNAF

use crate::vga_buffer::{WRITER, Color, ColorCode};

pub fn show_boot_logo() {
    // Рисуем красным цветом запрещающий знак
    {
        let mut writer = WRITER.lock();
        writer.color_code = ColorCode::new(Color::LightRed, Color::Black);
    }

    crate::println!("        ████████████");
    crate::println!("      ██            ██");
    crate::println!("    ██   ██████████   ██");
    crate::println!("   ██  ██          ██  ██");
    crate::println!("  ██  ██   /\\  /\\   ██  ██");
    crate::println!("  ██ ██   ( o  o )   ██ ██");
    crate::println!("  ██ ██    \\ ^^ /    ██ ██");
    crate::println!("  ██  ██  ████████  ██  ██");
    crate::println!("   ██  ██████████████  ██");
    crate::println!("    ██   ██████████   ██");
    crate::println!("      ██            ██");
    crate::println!("        ████████████");

    // Текст белым
    {
        let mut writer = WRITER.lock();
        writer.color_code = ColorCode::new(Color::White, Color::Black);
    }
    crate::println!("");
    crate::println!("        #[no_mangle]");

    // Возвращаем жёлтый
    {
        let mut writer = WRITER.lock();
        writer.color_code = ColorCode::new(Color::Yellow, Color::Black);
    }
    crate::println!("");
}
