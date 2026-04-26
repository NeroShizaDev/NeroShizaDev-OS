// ============================================================
// GAMES LAUNCHER — 5-in-1 menu, интегрирован в ядро
// ============================================================
// Возвращает управление в apps::launcher после выхода.
// ============================================================

use super::absurd_loading_screen::AbsurdLoadingScreen;
use super::byte_dodge::ByteDodgeGame;
use super::clicker_game::ClickerGame;
use super::common_hw::*;
use super::debil_card_game::DebilGame;
use super::match_grab_and_leave::CrystalMatchGame;
use crate::kernel_messages::Locale;

pub struct Launcher {
    selected: usize,
}

impl Launcher {
    pub const fn new() -> Self {
        Self { selected: 0 }
    }

    const ITEMS_COUNT: usize = 7;

    fn locale() -> Locale {
        crate::locale::get_locale()
    }

    fn use_russian() -> bool {
        matches!(Self::locale(), Locale::RuRu)
    }

    fn use_arabic() -> bool {
        matches!(Self::locale(), Locale::ArEg)
    }

    unsafe fn draw_border(&self) {
        let color = 0x0b;

        let mut x = 0;
        while x < VGA_WIDTH {
            write_vga_cell(x, 0, b'=', color);
            write_vga_cell(x, VGA_HEIGHT - 1, b'=', color);
            x += 1;
        }

        let mut y = 0;
        while y < VGA_HEIGHT {
            write_vga_cell(0, y, b'|', color);
            write_vga_cell(VGA_WIDTH - 1, y, b'|', color);
            y += 1;
        }

        write_vga_cell(0, 0, b'+', color);
        write_vga_cell(VGA_WIDTH - 1, 0, b'+', color);
        write_vga_cell(0, VGA_HEIGHT - 1, b'+', color);
        write_vga_cell(VGA_WIDTH - 1, VGA_HEIGHT - 1, b'+', color);
    }

    unsafe fn print_at(x: usize, y: usize, s: &str, color: u8) {
        let mut xx = x;
        for b in s.as_bytes() {
            if xx >= VGA_WIDTH || y >= VGA_HEIGHT {
                break;
            }
            write_vga_cell(xx, y, *b, color);
            xx += 1;
        }
    }

    unsafe fn print_at_utf8(x: usize, y: usize, s: &str, color: u8) {
        let mut xx = x;
        for ch in s.chars() {
            if xx >= VGA_WIDTH || y >= VGA_HEIGHT {
                break;
            }
            xx += crate::vga_unicode::print_char(ch as u32, xx, y, color).max(1);
        }
    }

    unsafe fn center_text(y: usize, s: &str, color: u8) {
        let len = s.as_bytes().len();
        let x = if len >= VGA_WIDTH {
            0
        } else {
            (VGA_WIDTH - len) / 2
        };
        Self::print_at(x, y, s, color);
    }

    unsafe fn center_text_utf8(y: usize, s: &str, color: u8) {
        let len = s.chars().count();
        let x = if len >= VGA_WIDTH {
            0
        } else {
            (VGA_WIDTH - len) / 2
        };
        Self::print_at_utf8(x, y, s, color);
    }

    fn subtitle_text() -> &'static str {
        match Self::locale() {
            Locale::RuRu => "ВЫБЕРИ ИГРУ",
            Locale::EnUs => "SELECT GAME",
            Locale::ArEg => "اختر لعبة",
        }
    }

    fn legal_text() -> &'static str {
        match Self::locale() {
            Locale::RuRu => "Doom, Племя и ещё пять странных игр",
            Locale::EnUs => "Doom, Tribe and five other strange games",
            Locale::ArEg => "Doom و Tribe وخمس ألعاب غريبة أخرى",
        }
    }

    fn status_line_text() -> &'static str {
        match Self::locale() {
            Locale::RuRu => "^v/W/S - выбор   ENTER - старт   Q/Esc - назад в APPS",
            Locale::EnUs => "^v/W/S - move   ENTER - start   Q/Esc - back to APPS",
            Locale::ArEg => "^v/W/S - تنقل   ENTER - بدء   Q/Esc - رجوع إلى APPS",
        }
    }

    fn status_line_2_text() -> &'static str {
        match Self::locale() {
            Locale::RuRu => "PC speaker включён / Doom и Tribe теперь внутри",
            Locale::EnUs => "PC speaker enabled / Doom and Tribe are now inside",
            Locale::ArEg => "PC speaker يعمل / Doom و Tribe داخل القائمة الآن",
        }
    }

    fn halt_title_text() -> &'static str {
        match Self::locale() {
            Locale::RuRu => "СИСТЕМА ОСТАНОВЛЕНА",
            Locale::EnUs => "SYSTEM HALTED",
            Locale::ArEg => "توقفت المنظومة",
        }
    }

    fn halt_subtitle_text() -> &'static str {
        match Self::locale() {
            Locale::RuRu => "энергия старого картриджа сохранена навсегда",
            Locale::EnUs => "old cartridge energy preserved forever",
            Locale::ArEg => "طاقة الخرطوشة القديمة محفوظة إلى الأبد",
        }
    }

    fn loading_text() -> &'static str {
        match Self::locale() {
            Locale::RuRu => "ЗАГРУЗКА...",
            Locale::EnUs => "LOADING...",
            Locale::ArEg => "جار التحميل...",
        }
    }

    fn footer_text() -> &'static str {
        match Self::locale() {
            Locale::RuRu => "NERO SHIZA DEV ИГРЫ",
            Locale::EnUs => "NERO SHIZA DEV",
            Locale::ArEg => "NERO SHIZA DEV ألعاب",
        }
    }

    fn footer_sub_text() -> &'static str {
        match Self::locale() {
            Locale::RuRu => "СБОРНИК 7 В 1",
            Locale::EnUs => "7 IN 1",
            Locale::ArEg => "7 في 1",
        }
    }

    unsafe fn draw_logo(&self) {
        Self::center_text(2, "########################################", 0x0e);
        Self::center_text(3, "#      NERO PIRATE 7 IN 1 TURBO       #", 0x0c);
        Self::center_text(4, "#      FAMILY GAME MEGA CARTRIDGE     #", 0x0a);
        Self::center_text(5, "########################################", 0x0e);

        Self::center_text_utf8(7, Self::subtitle_text(), 0x0f);
        Self::center_text_utf8(8, Self::legal_text(), 0x08);
    }

    unsafe fn draw_status_bar(&self) {
        let y = VGA_HEIGHT - 3;
        Self::print_at_utf8(3, y, Self::status_line_text(), 0x07);
        Self::print_at_utf8(3, y + 1, Self::status_line_2_text(), 0x08);
    }

    unsafe fn item_name(index: usize) -> &'static str {
        match Self::locale() {
            Locale::RuRu => match index {
                0 => "АБСУРДНАЯ ЗАГРУЗКА",
                1 => "КЛИКО-СКАМ",
                2 => "СХВАТИ ПАРУ",
                3 => "ДЕБИЛ-КАРТЫ",
                4 => "БАЙТ-ДОДЖ",
                5 => "DOOM FIRE MOD",
                _ => "ПЛЕМЯ",
            },
            Locale::EnUs => match index {
                0 => "ABSURD LOADING",
                1 => "KLIKO-SCAM",
                2 => "MATCH GRAB",
                3 => "DEBIL CARD",
                4 => "BYTE DODGE",
                5 => "DOOM FIRE MOD",
                _ => "TRIBE",
            },
            Locale::ArEg => match index {
                0 => "تحميل عبثي",
                1 => "كليكو-سكام",
                2 => "امسك الزوج",
                3 => "ورق غبي",
                4 => "بايت دودج",
                5 => "DOOM FIRE MOD",
                _ => "القبيلة",
            },
        }
    }

    unsafe fn item_desc(index: usize) -> &'static str {
        match Self::locale() {
            Locale::RuRu => match index {
                0 => "проклятая загрузка наоборот",
                1 => "закликай свою жизнь",
                2 => "схватил, поменял, повторил",
                3 => "хаос дворовых карт",
                4 => "новый порт с безопасным выходом",
                5 => "классический огонь, wad и mode 13h",
                _ => "каменный век: выживание и знания",
            },
            Locale::EnUs => match index {
                0 => "reverse progress cursed loader",
                1 => "click your life away",
                2 => "grab swap cascade repeat",
                3 => "yard card chaos simulator",
                4 => "new port with safe back handling",
                5 => "classic fire, wad and mode 13h",
                _ => "stone-age survival and knowledge",
            },
            Locale::ArEg => match index {
                0 => "شريط تحميل ملعون بالعكس",
                1 => "انقر حتى تضيع حياتك",
                2 => "امسك وبدل وكرر",
                3 => "فوضى ورق الشارع",
                4 => "منفذ جديد مع خروج آمن",
                5 => "نار كلاسيكية و wad و mode 13h",
                _ => "العصر الحجري: بقاء ومعرفة",
            },
        }
    }

    unsafe fn draw_menu(&self) {
        let base_y = 9;
        let mut i = 0;
        while i < Self::ITEMS_COUNT {
            let y = base_y + i;
            let selected = i == self.selected;
            let name_color = if selected { 0x0f } else { 0x07 };
            let desc_color = if selected { 0x0a } else { 0x08 };

            Self::print_at(
                3,
                y,
                "                                                                        ",
                0x01,
            );
            // Стрелка курсора: символ 0x10 (►) из VGA ROM
            if selected {
                unsafe {
                    write_vga_cell(4, y, 0x10, 0x0e); // ►
                    write_vga_cell(5, y, b' ', 0x0e);
                }
            } else {
                unsafe {
                    write_vga_cell(4, y, b' ', 0x08);
                    write_vga_cell(5, y, b' ', 0x08);
                }
            }
            Self::print_at_utf8(8, y, Self::item_name(i), name_color);
            Self::print_at(27, y, "|", if selected { 0x0e } else { 0x08 });
            Self::print_at_utf8(30, y, Self::item_desc(i), desc_color);

            i += 1;
        }
    }

    unsafe fn draw_frame(&mut self) {
        clear_screen(0x01);
        self.draw_border();
        self.draw_logo();
        self.draw_menu();
        self.draw_status_bar();

        Self::center_text_utf8(VGA_HEIGHT - 5, Self::footer_text(), 0x0d);
        Self::center_text_utf8(VGA_HEIGHT - 4, Self::footer_sub_text(), 0x0e);

        sync_cursor();
    }

    unsafe fn halt_screen(&self) -> ! {
        clear_screen(0x00);
        self.draw_border();
        Self::center_text_utf8(10, Self::halt_title_text(), 0x0c);
        Self::center_text_utf8(12, Self::halt_subtitle_text(), 0x07);
        speaker_off();
        loop {}
    }

    unsafe fn transition_screen(title: &str) {
        clear_screen(0x00);

        let mut i = 0;
        while i < VGA_HEIGHT {
            let mut x = 0;
            while x < VGA_WIDTH {
                write_vga_cell(x, i, b' ', ((i as u8) & 0x07) | 0x10);
                x += 1;
            }
            delay_cycles(2_000_000);
            i += 1;
        }

        clear_screen(0x00);
        Self::center_text_utf8(10, Self::loading_text(), 0x0e);
        Self::center_text_utf8(12, title, 0x0f);
        menu_launch_beep();
        delay_cycles(70_000_000);
    }

    unsafe fn run_selected(&mut self) {
        crate::serial_println!(
            "[GAMES][RUN] sel={} item={}",
            self.selected,
            Self::item_name(self.selected)
        );
        match self.selected {
            0 => {
                Self::transition_screen(Self::item_name(0));
                let mut m = AbsurdLoadingScreen::new();
                m.run();
            }
            1 => {
                Self::transition_screen(Self::item_name(1));
                let mut m = ClickerGame::new();
                m.run();
            }
            2 => {
                Self::transition_screen(Self::item_name(2));
                let mut m = CrystalMatchGame::new();
                m.run();
            }
            3 => {
                Self::transition_screen(Self::item_name(3));
                let mut m = DebilGame::new();
                m.run();
            }
            4 => {
                Self::transition_screen(Self::item_name(4));
                let mut m = ByteDodgeGame::new();
                m.run();
            }
            5 => {
                Self::transition_screen(Self::item_name(5));
                crate::apps::games::doom::init(&[]);
                crate::apps::games::doom::run();
                crate::apps::games::doom::on_destroy();
            }
            6 => {
                Self::transition_screen(Self::item_name(6));
                crate::apps::games::tribe::on_start();
                crate::apps::games::tribe::run();
                crate::apps::games::tribe::on_destroy();
            }
            _ => {}
        }
    }

    /// Возвращает `true` если нужно вернуться в APPS-меню, `false` = halt.
    pub unsafe fn run(&mut self) -> bool {
        self.draw_frame();
        crate::serial_println!(
            "[GAMES][SEL] init sel={} item={}",
            self.selected,
            Self::item_name(self.selected)
        );
        loop {
            let key = read_key_blocking_ext();
            let prev = self.selected;
            match key {
                b'w' | super::common_hw::KEY_UP => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    } else {
                        self.selected = Self::ITEMS_COUNT - 1;
                    }
                    menu_move_beep();
                    crate::serial_println!(
                        "[GAMES][SEL] up key=0x{:02X} sel={} item={}",
                        key,
                        self.selected,
                        Self::item_name(self.selected)
                    );
                }
                b's' | super::common_hw::KEY_DOWN => {
                    self.selected += 1;
                    if self.selected >= Self::ITEMS_COUNT {
                        self.selected = 0;
                    }
                    menu_move_beep();
                    crate::serial_println!(
                        "[GAMES][SEL] down key=0x{:02X} sel={} item={}",
                        key,
                        self.selected,
                        Self::item_name(self.selected)
                    );
                }
                b'\n' | b' ' => {
                    crate::serial_println!(
                        "[GAMES][CONFIRM] key=0x{:02X} sel={} item={}",
                        key,
                        self.selected,
                        Self::item_name(self.selected)
                    );
                    self.run_selected();
                    // После возврата из игры — перерисовываем меню
                    self.draw_frame();
                    continue;
                }
                // Esc (0x1B) или Q = выход обратно в APPS-меню
                0x1B | b'q' => {
                    crate::serial_println!("[GAMES][BACK] key=0x{:02X}", key);
                    return true;
                }
                _ => {}
            }
            // Перерисовываем только если выбор изменился
            if self.selected != prev {
                self.draw_frame();
            }
        }
    }
}
