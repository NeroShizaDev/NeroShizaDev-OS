// Логотип загрузки — ASCII-арт "no_mangle" в стиле FNAF

use crate::fb_buffer::{Color, ColorCode};

pub fn show_boot_logo() {
    // Рисуем красным цветом запрещающий знак
    crate::fb_buffer::set_color(ColorCode::new(Color::LightRed, Color::Black));

    for line in crate::kernel_messages::LOGO_ART {
        crate::println!("{}", line);
    }

    // Текст белым
    crate::fb_buffer::set_color(ColorCode::new(Color::White, Color::Black));
    crate::println!("");
    crate::println!("{}", crate::kernel_messages::LOGO_TAGLINE);

    // Возвращаем жёлтый
    crate::fb_buffer::set_color(ColorCode::new(Color::Yellow, Color::Black));
    crate::println!("");
}
