// Логотип загрузки — ASCII-арт "no_mangle" в стиле FNAF

use crate::vga_buffer::{WRITER, Color, ColorCode};

pub fn show_boot_logo() {
    // Рисуем красным цветом запрещающий знак
    {
        let mut writer = WRITER.lock();
        writer.color_code = ColorCode::new(Color::LightRed, Color::Black);
    }

    for line in crate::user_messages::LOGO_ART {
        crate::println!("{}", line);
    }

    // Текст белым
    {
        let mut writer = WRITER.lock();
        writer.color_code = ColorCode::new(Color::White, Color::Black);
    }
    crate::println!("");
    crate::println!("{}", crate::user_messages::LOGO_TAGLINE);

    // Возвращаем жёлтый
    {
        let mut writer = WRITER.lock();
        writer.color_code = ColorCode::new(Color::Yellow, Color::Black);
    }
    crate::println!("");
}
