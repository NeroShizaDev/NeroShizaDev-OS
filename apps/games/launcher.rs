// ============================================================
// GAMES LAUNCHER — 5-in-1 menu, интегрирован в ядро
// ============================================================
// Возвращает управление в apps::launcher после выхода.
// ============================================================

use super::common_hw::*;
use super::absurd_loading_screen::AbsurdLoadingScreen;
use super::clicker_game::ClickerGame;
use super::match_grab_and_leave::CrystalMatchGame;
use super::debil_card_game::DebilGame;
use super::module_demo::ModuleDemo;
use super::byte_dodge::ByteDodgeGame;

pub struct Launcher {
    selected: usize,
    blink_phase: bool,
}

impl Launcher {
    pub const fn new() -> Self {
        Self {
            selected: 0,
            blink_phase: false,
        }
    }

    const ITEMS_COUNT: usize = 6;

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

    unsafe fn center_text(y: usize, s: &str, color: u8) {
        let len = s.as_bytes().len();
        let x = if len >= VGA_WIDTH { 0 } else { (VGA_WIDTH - len) / 2 };
        Self::print_at(x, y, s, color);
    }

    unsafe fn draw_logo(&self) {
        Self::center_text(2,  "########################################", 0x0e);
        Self::center_text(3,  "#      NERO PIRATE 5 IN 1 TURBO       #", 0x0c);
        Self::center_text(4,  "#      FAMILY GAME MEGA CARTRIDGE     #", 0x0a);
        Self::center_text(5,  "########################################", 0x0e);

        Self::center_text(7,  "SELECT GAME", 0x0f);
        Self::center_text(8,  "totally original definitely legal", 0x08);
    }

    unsafe fn draw_status_bar(&self) {
        let y = VGA_HEIGHT - 3;
        Self::print_at(3, y, "W/S - move   ENTER - start   Q/Esc - back", 0x07);
        Self::print_at(3, y + 1, "PC speaker enabled / unsafe pirate edition", 0x08);
    }

    unsafe fn item_name(index: usize) -> &'static str {
        match index {
            0 => "ABSURD LOADING",
            1 => "KLIKO-SCAM",
            2 => "MATCH GRAB",
            3 => "DEBIL CARD",
            4 => "BYTE DODGE",
            _ => "MODULE DEMO",
        }
    }

    unsafe fn item_desc(index: usize) -> &'static str {
        match index {
            0 => "reverse progress cursed loader",
            1 => "click your life away",
            2 => "grab swap cascade repeat",
            3 => "yard card chaos simulator",
            4 => "new port with safe back handling",
            _ => "module demo from apps/games",
        }
    }

    unsafe fn draw_menu(&self) {
        let base_y = 11;
        let mut i = 0;
        while i < Self::ITEMS_COUNT {
            let y = base_y + i * 2;
            let selected = i == self.selected;

            if selected {
                let prefix = if self.blink_phase { ">>" } else { "::" };
                Self::print_at(14, y, prefix, 0x0e);
                Self::print_at(18, y, Self::item_name(i), 0x0f);
                Self::print_at(18, y + 1, Self::item_desc(i), 0x0a);
            } else {
                Self::print_at(14, y, "  ", 0x08);
                Self::print_at(18, y, Self::item_name(i), 0x07);
                Self::print_at(18, y + 1, Self::item_desc(i), 0x08);
            }

            i += 1;
        }
    }

    unsafe fn draw_frame(&mut self) {
        clear_screen(0x01);
        self.draw_border();
        self.draw_logo();
        self.draw_menu();
        self.draw_status_bar();

        Self::center_text(VGA_HEIGHT - 5, "NERO SHIZA DEV", 0x0d);
        Self::center_text(VGA_HEIGHT - 4, "5 IN 1", 0x0e);

        self.blink_phase = !self.blink_phase;
        sync_cursor();
    }

    unsafe fn halt_screen(&self) -> ! {
        clear_screen(0x00);
        self.draw_border();
        Self::center_text(10, "SYSTEM HALTED", 0x0c);
        Self::center_text(12, "old cartridge energy preserved forever", 0x07);
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
        Self::center_text(10, "LOADING...", 0x0e);
        Self::center_text(12, title, 0x0f);
        menu_launch_beep();
        delay_cycles(70_000_000);
    }

    unsafe fn run_selected(&mut self) {
        match self.selected {
            0 => {
                Self::transition_screen("ABSURD LOADING");
                let mut m = AbsurdLoadingScreen::new();
                m.run();
            }
            1 => {
                Self::transition_screen("KLIKO-SCAM");
                let mut m = ClickerGame::new();
                m.run();
            }
            2 => {
                Self::transition_screen("MATCH GRAB");
                let mut m = CrystalMatchGame::new();
                m.run();
            }
            3 => {
                Self::transition_screen("DEBIL CARD");
                let mut m = DebilGame::new();
                m.run();
            }
            4 => {
                Self::transition_screen("BYTE DODGE");
                let mut m = ByteDodgeGame::new();
                m.run();
            }
            5 => {
                Self::transition_screen("MODULE DEMO");
                let mut m = ModuleDemo::new();
                m.run();
            }
            _ => {}
        }
    }

    /// Возвращает `true` если нужно вернуться в APPS-меню, `false` = halt.
    pub unsafe fn run(&mut self) -> bool {
        loop {
            self.draw_frame();

            let key = read_key_blocking();
            match key {
                b'w' => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    } else {
                        self.selected = Self::ITEMS_COUNT - 1;
                    }
                    menu_move_beep();
                }
                b's' => {
                    self.selected += 1;
                    if self.selected >= Self::ITEMS_COUNT {
                        self.selected = 0;
                    }
                    menu_move_beep();
                }
                b'\n' | b' ' => {
                    self.run_selected();
                    // After game returns — redraw menu
                }
                // Esc (0x1B) или Q = выход обратно в APPS-меню
                0x1B | b'q' => {
                    return true;
                }
                _ => {}
            }
        }
    }
}
