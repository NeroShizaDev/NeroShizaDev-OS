use super::common_hw::*;

pub struct AbsurdLoadingScreen {
    pub loading_progress: u32,
    pub required_clicks: u32,
    pub current_clicks: u32,
}

impl AbsurdLoadingScreen {
    pub const fn new() -> Self {
        Self {
            loading_progress: 100,
            required_clicks: 100,
            current_clicks: 0,
        }
    }

    unsafe fn fake_bar_line(&self, fake_progress: u32) -> &'static str {
        if fake_progress > 75 {
            "##########.. 90% - Downloading clicks..."
        } else if fake_progress > 50 {
            "########.... 70% - Loading absurdity..."
        } else if fake_progress > 25 {
            "####........ 40% - Quantum superposition..."
        } else {
            "#........... 10% - Initializing madness..."
        }
    }

    pub unsafe fn display(&self) {
        clear_screen(0x0f);

        print_line("==================================================", 0x0c);
        print_line("         KLIKO-SCAM: ABSURD LOADING", 0x0e);
        print_line("==================================================", 0x0c);
        print_line("WARNING: LOADING WORKS IN REVERSE!", 0x0f);
        print_line("The more you click - the LESS progress!", 0x0f);
        print_line("--------------------------------------------------", 0x08);

        let fake_progress = 100 - ((self.current_clicks * 100) / self.required_clicks);
        let bar = self.fake_bar_line(fake_progress);

        print_line("", 0x0f);
        print("STATUS: ", 0x0b);
        print_line(bar, 0x0f);
        print_line("", 0x0f);

        print("[", 0x07);
        let bar_length = 40u32;
        let filled = (self.current_clicks * bar_length) / self.required_clicks;

        let mut i = 0;
        while i < filled {
            put_byte(b'#', 0x0a);
            i += 1;
        }
        while i < bar_length {
            put_byte(b'.', 0x08);
            i += 1;
        }
        print_line("]", 0x07);

        print_line("", 0x0f);
        print("CLICKS: ", 0x0f);
        print_u32(self.current_clicks, 0x0a);
        print("/", 0x0f);
        print_u32(self.required_clicks, 0x0a);
        put_byte(b'\n', 0x0f);

        print("FAKE PROGRESS: ", 0x0f);
        print_u32(fake_progress, 0x0c);
        print_line("%", 0x0c);

        print_line("", 0x0f);
        print_line("SPACE/ENTER = click, Q = exit", 0x08);
        sync_cursor();
    }

    pub unsafe fn process_click(&mut self) -> bool {
        self.current_clicks = self.current_clicks.saturating_add(1);
        self.loading_progress = 100 - ((self.current_clicks * 100) / self.required_clicks);

        if self.current_clicks % 25 == 0 && self.current_clicks < 100 {
            self.random_event();
        }

        self.current_clicks >= self.required_clicks
    }

    unsafe fn random_event(&mut self) {
        clear_screen(0x0f);
        let event_id = rand_range(5);

        match event_id {
            0 => {
                print_line("REALITY HAS FLIPPED!", 0x0d);
                print_line("Loading now goes forward. (Lie)", 0x0f);
            }
            1 => {
                print_line("GHOST OF NEROSHIZA!", 0x0d);
                print_line("It stole 3 clicks!", 0x0f);
                self.current_clicks = self.current_clicks.saturating_sub(3);
            }
            2 => {
                let add = rand_range(6) + 1;
                print_line("DICE THROW!", 0x0e);
                print("Bonus clicks: +", 0x0f);
                print_u32(add, 0x0a);
                put_byte(b'\n', 0x0f);
                self.current_clicks = self.current_clicks.saturating_add(add);
            }
            3 => {
                print_line("QUANTUM JUMP!", 0x0e);
                print_line("Clicks doubled!", 0x0f);
                self.current_clicks = self.current_clicks.saturating_mul(2);
                if self.current_clicks > self.required_clicks {
                    self.current_clicks = self.required_clicks;
                }
            }
            _ => {
                print_line("TIME LOOP!", 0x0e);
                print_line("Returned 15 clicks back!", 0x0f);
                self.current_clicks = self.current_clicks.saturating_sub(15);
            }
        }

        delay_cycles(150_000_000);
    }

    pub unsafe fn run(&mut self) {
        loop {
            self.display();
            let ch = read_key_blocking();

            if ch == b'q' {
                break;
            }

            if ch == b' ' || ch == b'\n' {
                if self.process_click() {
                    clear_screen(0x0f);
                    print_line("ABSURD LOADING COMPLETE.", 0x0a);
                    print_line("Actually progress reached zero, so success.", 0x0f);
                    delay_cycles(250_000_000);
                    break;
                }
            }
        }
    }
}

