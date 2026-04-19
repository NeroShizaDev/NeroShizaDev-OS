use super::common_hw::*;

#[derive(Copy, Clone)]
pub struct Card {
    pub suit: u8,
    pub rank: u8,
    pub is_joker: bool,
    pub is_red: bool,
    pub active: bool,
}

pub struct DebilGame {
    pub deck: [Card; 54],
    pub deck_len: usize,
    pub trump_suit: u8,
    pub hands: [[Card; 12]; 3],
    pub hand_len: [usize; 3],
    pub current_attacker: usize,
    pub current_defender: usize,
    pub game_over: bool,
}

impl DebilGame {
    pub const fn empty_card() -> Card {
        Card { suit: 0, rank: 0, is_joker: false, is_red: false, active: false }
    }

    pub const fn new() -> Self {
        Self {
            deck: [Self::empty_card(); 54],
            deck_len: 0,
            trump_suit: 0,
            hands: [[Self::empty_card(); 12]; 3],
            hand_len: [0; 3],
            current_attacker: 0,
            current_defender: 1,
            game_over: false,
        }
    }

    pub unsafe fn create_deck(&mut self) {
        self.deck_len = 0;
        let mut suit = 0;
        while suit < 4 {
            let mut rank = 0;
            while rank < 13 {
                self.deck[self.deck_len] = Card {
                    suit, rank, is_joker: false,
                    is_red: suit < 2, active: true,
                };
                self.deck_len += 1;
                rank += 1;
            }
            suit += 1;
        }
        self.deck[self.deck_len] = Card { suit: 4, rank: 99, is_joker: true, is_red: true,  active: true };
        self.deck_len += 1;
        self.deck[self.deck_len] = Card { suit: 5, rank: 99, is_joker: true, is_red: false, active: true };
        self.deck_len += 1;
    }

    pub unsafe fn shuffle(&mut self) {
        let mut i = self.deck_len;
        while i > 1 {
            i -= 1;
            let j = rand_range((i + 1) as u32) as usize;
            let tmp = self.deck[i];
            self.deck[i] = self.deck[j];
            self.deck[j] = tmp;
        }
    }

    pub unsafe fn pop_card(&mut self) -> Card {
        if self.deck_len == 0 { return Self::empty_card(); }
        self.deck_len -= 1;
        self.deck[self.deck_len]
    }

    pub unsafe fn hand_push(&mut self, player: usize, card: Card) {
        let idx = self.hand_len[player];
        if idx < 12 { self.hands[player][idx] = card; self.hand_len[player] += 1; }
    }

    pub unsafe fn start_game(&mut self) {
        self.create_deck();
        self.shuffle();
        let mut i = 0;
        while i < 3 { self.hand_len[i] = 0; i += 1; }
        let mut p = 0;
        while p < 3 {
            let mut c = 0;
            while c < 6 { let card = self.pop_card(); self.hand_push(p, card); c += 1; }
            p += 1;
        }
        let trump = self.pop_card();
        self.trump_suit = trump.suit;
        self.current_attacker = 0;
        self.current_defender = 1;
        self.game_over = false;
    }

    unsafe fn suit_char(suit: u8) -> u8 {
        match suit { 0 => b'H', 1 => b'D', 2 => b'C', 3 => b'S', _ => b'J' }
    }

    unsafe fn rank_char(rank: u8) -> u8 {
        match rank {
            0 => b'2', 1 => b'3', 2 => b'4', 3 => b'5', 4 => b'6',
            5 => b'7', 6 => b'8', 7 => b'9', 8 => b'T', 9 => b'J',
            10 => b'Q', 11 => b'K', 12 => b'A', _ => b'*',
        }
    }

    pub unsafe fn display(&self) {
        clear_screen(0x0f);
        print_line("==================================================", 0x0b);
        print_line("                    DEBIL", 0x0e);
        print_line("==================================================", 0x0b);

        print("Trump: ", 0x0f);
        put_byte(Self::suit_char(self.trump_suit), 0x0c);
        put_byte(b'\n', 0x0f);

        print("Deck: ", 0x0f);
        print_u32(self.deck_len as u32, 0x0a);
        put_byte(b'\n', 0x0f);

        print("Attacker: ", 0x0f);
        print_u32((self.current_attacker + 1) as u32, 0x0b);
        print(" Defender: ", 0x0f);
        print_u32((self.current_defender + 1) as u32, 0x0c);
        put_byte(b'\n', 0x0f);

        put_byte(b'\n', 0x0f);
        print_line("Your hand:", 0x0e);

        let mut i = 0;
        while i < self.hand_len[0] {
            let c = self.hands[0][i];
            print_u32(i as u32, 0x08);
            print(": ", 0x08);
            if c.is_joker {
                print_line("JK", if c.is_red { 0x0c } else { 0x07 });
            } else {
                put_byte(Self::rank_char(c.rank), 0x0f);
                put_byte(Self::suit_char(c.suit), if c.is_red { 0x0c } else { 0x0f });
                put_byte(b'\n', 0x0f);
            }
            i += 1;
        }

        put_byte(b'\n', 0x0f);
        print_line("Q=exit, placeholder battle loop", 0x08);
    }

    pub unsafe fn run(&mut self) {
        self.start_game();
        loop {
            self.display();
            let ch = read_key_blocking();
            if ch == b'q' { break; }
        }
    }
}

