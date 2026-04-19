// ============================================================
// BYTE DODGE — новый порт мини-игры (стартовый каркас)
// ============================================================
// Цель: показать безопасный цикл игры через common_hw:
// - poll_input() для неблокирующего ввода
// - draw_frame_begin() для единообразного рендера
// - Back (Q/Esc) для гарантированного возврата в Games-меню
// ============================================================

use super::common_hw::*;

pub struct ByteDodgeGame {
    player_x: usize,
    rock_x: usize,
    rock_y: usize,
    score: u32,
    exit_requested: bool,
}

impl ByteDodgeGame {
    pub const fn new() -> Self {
        Self {
            player_x: 40,
            rock_x: 40,
            rock_y: 3,
            score: 0,
            exit_requested: false,
        }
    }

    pub unsafe fn run(&mut self) {
        self.reset_round();

        loop {
            self.handle_input();
            if self.exit_requested {
                return;
            }
            if self.update_world() {
                if !self.game_over_screen() {
                    return;
                }
                self.reset_round();
            }
            self.draw();
            delay_cycles(6_000_000);
        }
    }

    unsafe fn reset_round(&mut self) {
        self.player_x = (VGA_WIDTH / 2).clamp(2, VGA_WIDTH - 3);
        self.rock_x = 2 + rand_range((VGA_WIDTH - 4) as u32) as usize;
        self.rock_y = 3;
        self.score = 0;
        self.exit_requested = false;
    }

    unsafe fn handle_input(&mut self) {
        match poll_input() {
            GameInput::Left => {
                if self.player_x > 2 {
                    self.player_x -= 1;
                }
            }
            GameInput::Right => {
                if self.player_x + 1 < VGA_WIDTH - 2 {
                    self.player_x += 1;
                }
            }
            input if is_back_input(input) => {
                // Мягкий выход в Games launcher
                self.exit_requested = true;
            }
            _ => {}
        }
    }

    /// Возвращает true, когда раунд завершен (поймали игрока или soft-exit).
    unsafe fn update_world(&mut self) -> bool {
        if self.rock_y >= VGA_HEIGHT {
            return true;
        }

        self.rock_y += 1;
        let player_row = VGA_HEIGHT - 3;

        if self.rock_y == player_row {
            if self.rock_x == self.player_x {
                return true;
            }
            self.score = self.score.saturating_add(1);
            self.rock_x = 2 + rand_range((VGA_WIDTH - 4) as u32) as usize;
            self.rock_y = 3;
        }

        false
    }

    unsafe fn draw(&self) {
        draw_frame_begin(0x00, 0x0B);
        Self::print_center(2, "BYTE DODGE", 0x0E);
        Self::print_center(3, "A/D move, Q/Esc back", 0x08);

        // HUD
        Self::print_at(3, 2, "Score:", 0x0A);
        Self::print_u32_at(10, 2, self.score, 0x0A);

        // Падающий байт и игрок
        let player_row = VGA_HEIGHT - 3;
        write_vga_cell(self.player_x, player_row, b'W', 0x0F);
        if self.rock_y < VGA_HEIGHT - 1 {
            write_vga_cell(self.rock_x, self.rock_y, b'X', 0x0C);
        }

        sync_cursor();
    }

    /// true = restart, false = back to menu.
    unsafe fn game_over_screen(&self) -> bool {
        draw_frame_begin(0x00, 0x0B);
        Self::print_center(9, "ROUND END", 0x0C);
        Self::print_center(11, "Enter/Space = restart", 0x0F);
        Self::print_center(12, "Q/Esc = back", 0x08);
        Self::print_at(30, 14, "Final score:", 0x0A);
        Self::print_u32_at(43, 14, self.score, 0x0A);
        sync_cursor();

        loop {
            match poll_input() {
                GameInput::Confirm => return true,
                input if is_back_input(input) => return false,
                _ => delay_cycles(1_000_000),
            }
        }
    }

    unsafe fn print_center(y: usize, s: &str, color: u8) {
        let x = (VGA_WIDTH.saturating_sub(s.len())) / 2;
        Self::print_at(x, y, s, color);
    }

    unsafe fn print_at(x: usize, y: usize, s: &str, color: u8) {
        let mut xx = x;
        for &b in s.as_bytes() {
            if xx >= VGA_WIDTH || y >= VGA_HEIGHT {
                break;
            }
            write_vga_cell(xx, y, b, color);
            xx += 1;
        }
    }

    unsafe fn print_u32_at(x: usize, y: usize, n: u32, color: u8) {
        if n == 0 {
            write_vga_cell(x, y, b'0', color);
            return;
        }
        let mut buf = [0u8; 10];
        let mut i = 10usize;
        let mut v = n;
        while v > 0 {
            i -= 1;
            buf[i] = b'0' + (v % 10) as u8;
            v /= 10;
        }
        let mut xx = x;
        for &b in &buf[i..] {
            if xx >= VGA_WIDTH || y >= VGA_HEIGHT { break; }
            write_vga_cell(xx, y, b, color);
            xx += 1;
        }
    }
}


