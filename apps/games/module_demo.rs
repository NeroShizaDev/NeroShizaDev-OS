// ============================================================
// MODULE DEMO — модуль-демо внутри apps/games
// ============================================================
// Мини-экран с сообщением и возвратом в Games-меню по любой клавише.
// Это безопасный инкремент миграции legacy-модуля в apps/games.
// ============================================================

use super::common_hw::*;

pub struct ModuleDemo;

impl Default for ModuleDemo {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleDemo {
    pub const fn new() -> Self {
        Self
    }

    pub unsafe fn run(&mut self) {
        clear_screen(0x01);

        let title = "MODULE DEMO";
        let hint = "Press any key to return";
        let msg = "Hello from apps/games module demo";

        let tx = (VGA_WIDTH.saturating_sub(title.len())) / 2;
        let mx = (VGA_WIDTH.saturating_sub(msg.len())) / 2;
        let hx = (VGA_WIDTH.saturating_sub(hint.len())) / 2;

        LauncherBridge::draw_box();
        LauncherBridge::print_at(tx, 7, title, 0x0E);
        LauncherBridge::print_at(mx, 11, msg, 0x0F);
        LauncherBridge::print_at(hx, 16, hint, 0x08);
        sync_cursor();

        let _ = read_key_blocking();
    }
}

// Небольшой мост, чтобы не дублировать рамку/печать по всему коду.
struct LauncherBridge;

impl LauncherBridge {
    unsafe fn draw_box() {
        let c = 0x0B;
        for x in 0..VGA_WIDTH {
            write_vga_cell(x, 0, b'=', c);
            write_vga_cell(x, VGA_HEIGHT - 1, b'=', c);
        }
        for y in 0..VGA_HEIGHT {
            write_vga_cell(0, y, b'|', c);
            write_vga_cell(VGA_WIDTH - 1, y, b'|', c);
        }
        write_vga_cell(0, 0, b'+', c);
        write_vga_cell(VGA_WIDTH - 1, 0, b'+', c);
        write_vga_cell(0, VGA_HEIGHT - 1, b'+', c);
        write_vga_cell(VGA_WIDTH - 1, VGA_HEIGHT - 1, b'+', c);
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
}
