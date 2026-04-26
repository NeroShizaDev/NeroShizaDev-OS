use super::common_hw::*;

pub struct ClickerGame {
    pub total_clicks_x10: u64,
    pub manual_click_power: u32,
    pub autoclicker_power_x10: u32,
    pub is_ng_plus_unlocked: bool,

    pub manual_upgrade_level: u32,
    pub manual_upgrade_base_cost: u32,

    pub autoclicker_upgrade_level: u32,
    pub autoclicker_upgrade_base_cost: u32,

    pub interface_upgrade_unlocked: bool,
    pub interface_upgraded: bool,
    pub navigation_upgrade_unlocked: bool,
    pub navigation_upgraded: bool,
    pub selected_option: u8,

    pub manual_upgrade_cost: u32,
    pub autoclicker_upgrade_cost: u32,

    pub last_tsc: u64,
}

impl ClickerGame {
    pub const fn new() -> Self {
        Self {
            total_clicks_x10: 0,
            manual_click_power: 1,
            autoclicker_power_x10: 0,
            is_ng_plus_unlocked: false,
            manual_upgrade_level: 1,
            manual_upgrade_base_cost: 10,
            autoclicker_upgrade_level: 0,
            autoclicker_upgrade_base_cost: 25,
            interface_upgrade_unlocked: false,
            interface_upgraded: false,
            navigation_upgrade_unlocked: false,
            navigation_upgraded: false,
            selected_option: 0,
            manual_upgrade_cost: 0,
            autoclicker_upgrade_cost: 0,
            last_tsc: 0,
        }
    }

    unsafe fn pow_ceil_int(base: u32, level: u32, ng_plus: bool, is_auto: bool) -> u32 {
        let mut val = base as u64;
        let mut i = 0;
        while i < level {
            if ng_plus {
                if is_auto {
                    val = (val * 18 + 9) / 10;
                } else {
                    val = (val * 15 + 9) / 10;
                }
            } else {
                if is_auto {
                    val = (val * 12 + 9) / 10;
                } else {
                    val = (val * 115 + 99) / 100;
                }
            }
            i += 1;
        }
        if val > 0xFFFF_FFFF {
            0xFFFF_FFFF
        } else {
            val as u32
        }
    }

    pub unsafe fn init(&mut self) {
        self.update_costs();
        self.last_tsc = rdtsc();
    }

    pub unsafe fn update_costs(&mut self) {
        self.manual_upgrade_cost = Self::pow_ceil_int(
            self.manual_upgrade_base_cost,
            self.manual_upgrade_level,
            self.is_ng_plus_unlocked,
            false,
        );
        self.autoclicker_upgrade_cost = Self::pow_ceil_int(
            self.autoclicker_upgrade_base_cost,
            self.autoclicker_upgrade_level,
            self.is_ng_plus_unlocked,
            true,
        );
    }

    pub unsafe fn update(&mut self) -> bool {
        let now = rdtsc();
        let delta = now.wrapping_sub(self.last_tsc);
        self.last_tsc = now;
        let steps = delta / 50_000_000u64;
        self.total_clicks_x10 = self
            .total_clicks_x10
            .saturating_add((self.autoclicker_power_x10 as u64).saturating_mul(steps));
        let total_whole = (self.total_clicks_x10 / 10) as u32;
        if !self.interface_upgrade_unlocked && total_whole >= 100 {
            self.interface_upgrade_unlocked = true;
        }
        if !self.navigation_upgrade_unlocked && total_whole >= 200 {
            self.navigation_upgrade_unlocked = true;
        }
        if !self.is_ng_plus_unlocked && total_whole >= 1_000_000 {
            self.is_ng_plus_unlocked = true;
            self.update_costs();
            return true;
        }
        false
    }

    pub unsafe fn display_status(&self) {
        clear_screen(0x0f);
        print_line("==================================================", 0x0b);
        print_line("               KLIKO-SCAM: START", 0x0e);
        print_line("==================================================", 0x0b);

        print("CLICKS: ", 0x0f);
        print_u64(self.total_clicks_x10 / 10, 0x0a);
        put_byte(b'\n', 0x0f);

        print("CPS x10: ", 0x0f);
        print_u32(self.autoclicker_power_x10, 0x0a);
        put_byte(b'\n', 0x0f);

        print_line("", 0x0f);
        print_line("--- UPGRADES ---", 0x0e);

        print("Manual click power: ", 0x0f);
        print_u32(self.manual_click_power, 0x0a);
        print("  Level: ", 0x0f);
        print_u32(self.manual_upgrade_level, 0x0b);
        put_byte(b'\n', 0x0f);

        print("Manual upgrade cost: ", 0x0f);
        print_u32(self.manual_upgrade_cost, 0x0c);
        put_byte(b'\n', 0x0f);

        print("Autoclicker power x10: ", 0x0f);
        print_u32(self.autoclicker_power_x10, 0x0a);
        print("  Level: ", 0x0f);
        print_u32(self.autoclicker_upgrade_level, 0x0b);
        put_byte(b'\n', 0x0f);

        print("Autoclicker cost: ", 0x0f);
        print_u32(self.autoclicker_upgrade_cost, 0x0c);
        put_byte(b'\n', 0x0f);

        if self.is_ng_plus_unlocked {
            print_line("", 0x0f);
            print_line("NG+ ACTIVE", 0x0d);
        }
        if self.interface_upgrade_unlocked && !self.interface_upgraded {
            print_line("UI upgrade available: press U", 0x0e);
        }
        if self.navigation_upgrade_unlocked && !self.navigation_upgraded {
            print_line("Navigation upgrade available: press N", 0x0e);
        }

        print_line("", 0x0f);
        print_line(
            "1=Click  2=Upgrade Click  3=Upgrade Auto  U=UI  N=NAV  Q=Exit",
            0x08,
        );

        let _x87 = x87_fyl2x_demo(2.0, 8.0);
        sync_cursor();
    }

    pub unsafe fn manual_click(&mut self) {
        self.total_clicks_x10 = self
            .total_clicks_x10
            .saturating_add((self.manual_click_power as u64) * 10);
    }

    pub unsafe fn upgrade_manual(&mut self) -> bool {
        let total_whole = (self.total_clicks_x10 / 10) as u32;
        if total_whole >= self.manual_upgrade_cost {
            self.total_clicks_x10 = self
                .total_clicks_x10
                .saturating_sub((self.manual_upgrade_cost as u64) * 10);
            self.manual_upgrade_level = self.manual_upgrade_level.saturating_add(1);
            self.manual_click_power = self.manual_click_power.saturating_add(1);
            self.update_costs();
            return true;
        }
        false
    }

    pub unsafe fn upgrade_autoclicker(&mut self) -> bool {
        let total_whole = (self.total_clicks_x10 / 10) as u32;
        if total_whole >= self.autoclicker_upgrade_cost {
            self.total_clicks_x10 = self
                .total_clicks_x10
                .saturating_sub((self.autoclicker_upgrade_cost as u64) * 10);
            if self.autoclicker_upgrade_level == 0 {
                self.autoclicker_power_x10 = self.autoclicker_power_x10.saturating_add(1);
            } else {
                self.autoclicker_power_x10 = self
                    .autoclicker_power_x10
                    .saturating_add(self.autoclicker_upgrade_level);
            }
            self.autoclicker_upgrade_level = self.autoclicker_upgrade_level.saturating_add(1);
            self.update_costs();
            return true;
        }
        false
    }

    pub unsafe fn upgrade_interface(&mut self) -> bool {
        if !self.interface_upgraded && self.interface_upgrade_unlocked {
            self.interface_upgraded = true;
            return true;
        }
        false
    }

    pub unsafe fn upgrade_navigation(&mut self) -> bool {
        if !self.navigation_upgraded && self.navigation_upgrade_unlocked {
            self.navigation_upgraded = true;
            return true;
        }
        false
    }

    pub unsafe fn run(&mut self) {
        self.init();
        loop {
            let ng_unlocked = self.update();
            self.display_status();
            if ng_unlocked {
                print_line("NEW GAME PLUS UNLOCKED!", 0x0d);
                delay_cycles(250_000_000);
            }
            let ch = read_key_blocking();
            match ch {
                b'1' | b' ' | b'\n' => self.manual_click(),
                b'2' => {
                    let _ = self.upgrade_manual();
                }
                b'3' => {
                    let _ = self.upgrade_autoclicker();
                }
                b'u' => {
                    let _ = self.upgrade_interface();
                }
                b'n' => {
                    let _ = self.upgrade_navigation();
                }
                b'q' => break,
                _ => {}
            }
        }
    }
}
