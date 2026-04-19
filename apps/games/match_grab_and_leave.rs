use super::common_hw::*;

pub struct CrystalMatchGame {
    pub size: usize,
    pub grid: [[u8; 6]; 6],
    pub score: u32,
    pub held_crystal: u8,
    pub held_x: usize,
    pub held_y: usize,
    pub has_held: bool,
    pub moves_left: u32,
}

impl CrystalMatchGame {
    pub const fn new() -> Self {
        Self {
            size: 6,
            grid: [[0; 6]; 6],
            score: 0,
            held_crystal: 0,
            held_x: 0,
            held_y: 0,
            has_held: false,
            moves_left: 30,
        }
    }

    unsafe fn random_crystal(&self) -> u8 {
        (rand_range(5) as u8) + 1
    }

    unsafe fn crystal_char(id: u8) -> u8 {
        match id {
            1 => b'D', 2 => b'F', 3 => b'W', 4 => b'A', _ => b'S',
        }
    }

    pub unsafe fn generate_grid(&mut self) {
        let mut y = 0;
        while y < self.size {
            let mut x = 0;
            while x < self.size {
                self.grid[y][x] = self.random_crystal();
                x += 1;
            }
            y += 1;
        }
    }

    pub unsafe fn init(&mut self) {
        loop {
            self.generate_grid();
            if self.find_matches_count() == 0 { break; }
        }
    }

    pub unsafe fn display(&self) {
        clear_screen(0x0f);
        print_line("==================================================", 0x0b);
        print_line("              ABSURD MATCH-3", 0x0e);
        print_line("==================================================", 0x0b);

        print("Score: ", 0x0f);
        print_u32(self.score, 0x0a);
        print("  Moves: ", 0x0f);
        print_u32(self.moves_left, 0x0c);
        put_byte(b'\n', 0x0f);
        put_byte(b'\n', 0x0f);

        print_line("   0 1 2 3 4 5", 0x08);

        let mut y = 0;
        while y < self.size {
            print_u32(y as u32, 0x08);
            print(" ", 0x08);
            let mut x = 0;
            while x < self.size {
                if self.has_held && self.held_x == x && self.held_y == y {
                    print(". ", 0x0d);
                } else {
                    put_byte(Self::crystal_char(self.grid[y][x]), 0x0f);
                    put_byte(b' ', 0x0f);
                }
                x += 1;
            }
            put_byte(b'\n', 0x0f);
            y += 1;
        }

        put_byte(b'\n', 0x0f);
        if self.has_held {
            print_line("Held crystal active. Use XY to swap, R to return.", 0x0e);
        } else {
            print_line("Select crystal with two digits XY. Q=exit", 0x08);
        }
    }

    pub unsafe fn grab_crystal(&mut self, x: usize, y: usize) -> bool {
        if x >= self.size || y >= self.size || self.has_held { return false; }
        self.held_crystal = self.grid[y][x];
        self.held_x = x; self.held_y = y;
        self.grid[y][x] = 0;
        self.has_held = true;
        true
    }

    pub unsafe fn swap_crystal(&mut self, x: usize, y: usize) -> bool {
        if !self.has_held || x >= self.size || y >= self.size { return false; }
        if x == self.held_x && y == self.held_y { return false; }
        if self.grid[y][x] == 0 { return false; }
        let target = self.grid[y][x];
        self.grid[y][x] = self.held_crystal;
        self.grid[self.held_y][self.held_x] = target;
        self.has_held = false;
        let had = self.check_and_resolve_matches();
        self.moves_left = self.moves_left.saturating_sub(1);
        if had { self.score = self.score.saturating_add(10); }
        true
    }

    pub unsafe fn return_crystal(&mut self) {
        if self.has_held {
            self.grid[self.held_y][self.held_x] = self.held_crystal;
            self.has_held = false;
        }
    }

    pub unsafe fn find_matches_count(&self) -> u32 {
        let mut count = 0;
        let mut y = 0;
        while y < self.size {
            let mut x = 0;
            while x + 2 < self.size {
                let c = self.grid[y][x];
                if c != 0 && c == self.grid[y][x+1] && c == self.grid[y][x+2] { count += 3; }
                x += 1;
            }
            y += 1;
        }
        let mut x = 0;
        while x < self.size {
            let mut y2 = 0;
            while y2 + 2 < self.size {
                let c = self.grid[y2][x];
                if c != 0 && c == self.grid[y2+1][x] && c == self.grid[y2+2][x] { count += 3; }
                y2 += 1;
            }
            x += 1;
        }
        count
    }

    pub unsafe fn check_and_resolve_matches(&mut self) -> bool {
        let mut any = false;
        loop {
            let mut marked = [[false; 6]; 6];
            let mut found = false;
            let mut y = 0;
            while y < self.size {
                let mut x = 0;
                while x + 2 < self.size {
                    let c = self.grid[y][x];
                    if c != 0 && c == self.grid[y][x+1] && c == self.grid[y][x+2] {
                        marked[y][x] = true; marked[y][x+1] = true; marked[y][x+2] = true;
                        found = true;
                    }
                    x += 1;
                }
                y += 1;
            }
            let mut x = 0;
            while x < self.size {
                let mut y2 = 0;
                while y2 + 2 < self.size {
                    let c = self.grid[y2][x];
                    if c != 0 && c == self.grid[y2+1][x] && c == self.grid[y2+2][x] {
                        marked[y2][x] = true; marked[y2+1][x] = true; marked[y2+2][x] = true;
                        found = true;
                    }
                    y2 += 1;
                }
                x += 1;
            }
            if !found { break; }
            any = true;
            let mut yy = 0;
            while yy < self.size {
                let mut xx = 0;
                while xx < self.size {
                    if marked[yy][xx] {
                        self.grid[yy][xx] = 0;
                        self.score = self.score.saturating_add(10);
                    }
                    xx += 1;
                }
                yy += 1;
            }
            self.apply_gravity();
            self.fill_empty_cells();
        }
        any
    }

    pub unsafe fn apply_gravity(&mut self) {
        let mut x = 0;
        while x < self.size {
            let mut write_y = self.size;
            let mut y = self.size;
            while y > 0 {
                y -= 1;
                let c = self.grid[y][x];
                if c != 0 {
                    write_y -= 1;
                    self.grid[write_y][x] = c;
                    if write_y != y { self.grid[y][x] = 0; }
                }
            }
            let mut y2 = 0;
            while y2 < write_y { self.grid[y2][x] = 0; y2 += 1; }
            x += 1;
        }
    }

    pub unsafe fn fill_empty_cells(&mut self) {
        let mut y = 0;
        while y < self.size {
            let mut x = 0;
            while x < self.size {
                if self.grid[y][x] == 0 { self.grid[y][x] = self.random_crystal(); }
                x += 1;
            }
            y += 1;
        }
    }

    unsafe fn read_xy() -> Option<(usize, usize)> {
        let a = read_key_blocking();
        if a == b'q' { return None; }
        if a == b'r' { return Some((9, 9)); }
        if a < b'0' || a > b'9' { return Some((8, 8)); }
        let b = read_key_blocking();
        if b < b'0' || b > b'9' { return Some((8, 8)); }
        Some(((a - b'0') as usize, (b - b'0') as usize))
    }

    pub unsafe fn run(&mut self) {
        self.init();
        loop {
            self.display();
            if self.moves_left == 0 {
                print_line("GAME OVER", 0x0c);
                delay_cycles(300_000_000);
                break;
            }
            match Self::read_xy() {
                None => break,
                Some((9, 9)) => self.return_crystal(),
                Some((8, 8)) => {}
                Some((x, y)) => {
                    if self.has_held { let _ = self.swap_crystal(x, y); }
                    else             { let _ = self.grab_crystal(x, y); }
                }
            }
        }
    }
}

