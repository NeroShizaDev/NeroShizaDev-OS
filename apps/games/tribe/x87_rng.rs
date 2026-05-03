// x87_rng.rs — рандомайзер на математическом сопроцессоре x87
// no_std: никаких зависимостей от стандартной библиотеки

use core::arch::asm;

static mut X87_READY: bool = false;

#[derive(Clone, Copy)]
pub struct X87Rng {
    state: u64, // seed в битах f64
}

impl X87Rng {
    pub fn new(seed: u64) -> Self {
        ensure_x87_ready();
        Self {
            state: seed ^ 0xDEAD_BEEF_CAFE_1337,
        }
    }

    /// Основной генератор — хаос через синус и Пи на x87
    #[inline]
    fn next_raw(&mut self) -> u32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;

        let state_bits = (self.state & 0x7fff_ffff_ffff_ffff).saturating_add(1);
        let scale: f64 = 4_294_967_295.0;
        let mut mixed: i64 = 0;

        // Максимальная глубина x87-стека здесь = 2: ST(0), ST(1).
        unsafe {
            asm!(
                "fild qword ptr [{state}]",
                "fldpi",
                "fmulp",
                "fsin",
                "fabs",
                "fmul qword ptr [{scale}]",
                "fistp qword ptr [{out}]",
                state = in(reg) &state_bits as *const u64,
                scale = in(reg) &scale as *const f64,
                out = in(reg) &mut mixed as *mut i64,
                options(nostack),
            );
        }

        self.state = self.state.rotate_left(17) ^ (mixed as u64) ^ 0xA076_1D64_78BD_642F;
        ((mixed as u64) ^ self.state) as u32
    }

    pub fn roll_index(&mut self, max: usize) -> usize {
        if max <= 1 {
            0
        } else {
            (self.next_raw() as usize) % max
        }
    }

    pub fn roll_1d3(&mut self) -> u8 {
        (self.next_raw() % 3 + 1) as u8
    }

    pub fn roll_1d6(&mut self) -> u8 {
        (self.next_raw() % 6 + 1) as u8
    }

    pub fn roll_2d6(&mut self) -> u8 {
        let a = (self.next_raw() % 6 + 1) as u8;
        let b = (self.next_raw() % 6 + 1) as u8;
        a + b
    }

    pub fn roll_3d6(&mut self) -> u8 {
        let a = (self.next_raw() % 6 + 1) as u8;
        let b = (self.next_raw() % 6 + 1) as u8;
        let c = (self.next_raw() % 6 + 1) as u8;
        a + b + c
    }

    pub fn roll_1d6_minus1(&mut self) -> usize {
        (self.next_raw() % 6) as usize
    }

    pub fn roll_1d3_minus1(&mut self) -> usize {
        (self.next_raw() % 3) as usize
    }
}

fn ensure_x87_ready() {
    unsafe {
        if !X87_READY {
            crate::apps::fpu::init();
            X87_READY = true;
        }
    }
}
