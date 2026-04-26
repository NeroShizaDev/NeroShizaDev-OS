// ============================================================
// DOOM WATCHDOG — RDTSC-таймер кадров + KernelBiteCode система
// ============================================================
// Если кадр не обновлялся >2 сек — издаём KernelBite и перезагружаемся.
//
// KernelBiteCode (0xBAF_...):
//   BITE_CLASSIC    0xBAFBAF   — стандартный таймаут watchdog
//   BITE_ORACLE_LOOP 0xBAF001  — Voodoo завис в вопросе
//   BITE_DEMONIC    0xBAF666   — демонический сбой без причины
//   BITE_DEAD       0xBAFDEAD  — Double Fault + Abort
//   BITE_BASE_415   0xBAFBACE415 — катастрофический сбой памяти
//   BITE_ELITE      0xBAF1337  — LEET-укус
//   BITE_INFINITY   0xBAF88    — бесконечный цикл (88 = ∞)
//   BITE_EMERGENCY  0xBAF911   — экстренный сигнал
//
// AppBiteCode (0xBAD_... / 0xBED_...):
//   BITE_BAD_BOY    0xBADBAF   — app вышел за пределы
//   BITE_POISON     0xBADF00D  — отравленные данные (BAD FOOD)
//   BITE_SLEEPY     0xBEDBAF   — app уснул
//   BITE_BEDA_BEDA  0xBEDABEDA — двойной сбой в app
// ============================================================

use core::arch::asm;

// Грубая оценка тиков TSC (при ~1 ГГц).
// Был 500 мс — слишком мало для 2-пассового fire (рендер ~50–100 мс/кадр).
const FRAME_TICKS: u64 = 35_000_000; // ~35 FPS @ 1 GHz
const TIMEOUT_TICKS: u64 = 2_000_000_000; // 2 сек — с запасом для медленных кадров
const FPU_TIMEOUT_TICKS: u64 = 5_000_000_000; // 5 сек — FPU fire заметно тяжелее обычного

static mut LAST_FRAME_TSC: u64 = 0;
static mut FRAME_COUNT: u64 = 0;
static mut BITE_ELAPSED: u64 = 0;
static mut BITE_TIMEOUT: u64 = 0;
static mut BITE_FIRE_MODE: [u8; 32] = [0u8; 32];
static mut BITE_FIRE_MODE_LEN: usize = 0;

// ============================================================
// KernelBiteCode — коды сбоев NeroShizaDev-OS
// ============================================================

pub type BiteCode = u64;

// Kernel (0xBAF_...)
pub const BITE_CLASSIC: BiteCode = 0x00BAF_BAF; // стандартный таймаут
pub const BITE_ORACLE_LOOP: BiteCode = 0x00BAF_001; // Voodoo завис
pub const BITE_DEMONIC: BiteCode = 0x00BAF_666; // демонический сбой
pub const BITE_DEAD: BiteCode = 0x00BAF_DEAD; // Double Fault + Abort
pub const BITE_BASE_415: BiteCode = 0xBAF_BACE415; // катастрофа памяти
pub const BITE_ELITE: BiteCode = 0x00BAF_1337; // LEET-статус
pub const BITE_INFINITY: BiteCode = 0x000_BAF_88; // бесконечный цикл
pub const BITE_EMERGENCY: BiteCode = 0x000_BAF_911; // экстренный сигнал

// App (0xBAD_... / 0xBED_...)
pub const BITE_BAD_BOY: BiteCode = 0x00BAD_BAF; // app за пределами
pub const BITE_POISON: BiteCode = 0x00BAD_F00D; // BAD FOOD
pub const BITE_SLEEPY: BiteCode = 0x00BED_BAF; // app уснул
pub const BITE_BEDA_BEDA: BiteCode = 0xBEDA_BEDA; // двойной сбой app

fn bite_name(code: BiteCode) -> &'static str {
    match code {
        BITE_CLASSIC => "BITE_CLASSIC",
        BITE_ORACLE_LOOP => "BITE_ORACLE_LOOP",
        BITE_DEMONIC => "BITE_DEMONIC",
        BITE_DEAD => "BITE_DEAD",
        BITE_BASE_415 => "BITE_BASE_415",
        BITE_ELITE => "BITE_ELITE",
        BITE_INFINITY => "BITE_INFINITY",
        BITE_EMERGENCY => "BITE_EMERGENCY",
        BITE_BAD_BOY => "BITE_BAD_BOY",
        BITE_POISON => "BITE_POISON",
        BITE_SLEEPY => "BITE_SLEEPY",
        BITE_BEDA_BEDA => "BITE_BEDA_BEDA",
        _ => "BITE_UNKNOWN",
    }
}

/// Выводит экран KernelBite (стиль как у DF-handler) и останавливает систему.
pub fn kernel_bite(code: BiteCode) -> ! {
    crate::serial_println!("[BITE] 0x{:X} :: {}", code, bite_name(code));
    crate::serial_println!("[BITE] KernelBite — system halted.");

    // Вспомогательные функции — прямая запись в VGA 0xB8000, без mutex/alloc.
    #[inline(always)]
    fn vga_put(row: usize, col: usize, ch: u8, attr: u8) {
        let vga = 0xB8000 as *mut u16;
        unsafe {
            core::ptr::write_volatile(vga.add(row * 80 + col), ((attr as u16) << 8) | ch as u16);
        }
    }
    fn vga_write(row: usize, col: usize, s: &[u8], attr: u8) {
        let mut c = col;
        for &b in s {
            if c >= 80 {
                break;
            }
            vga_put(row, c, b, attr);
            c += 1;
        }
    }
    fn vga_hex64(row: usize, col: usize, mut v: u64, attr: u8) {
        let mut buf = [0u8; 16];
        for i in (0..16).rev() {
            let d = (v & 0xF) as u8;
            buf[i] = if d < 10 { b'0' + d } else { b'A' + d - 10 };
            v >>= 4;
        }
        let mut c = col;
        for &b in &buf {
            if c >= 80 {
                break;
            }
            vga_put(row, c, b, attr);
            c += 1;
        }
    }

    // Восстанавливаем текстовый режим из Mode 13h (если активен).
    // Не вызываем load_static_glyphs — нам нужен только ASCII 32-127.
    unsafe {
        super::vga_graphics::restore_text_mode();
    }

    const ATTR_BG: u8 = 0x0F; // белый текст, чёрный фон
    const ATTR_FRAME: u8 = 0xDF; // белый текст, пурпурный фон (как у DF-экрана)
    const ATTR_HI: u8 = 0x0D; // ярко-пурпурный текст
    const ATTR_WARN: u8 = 0x0E; // жёлтый текст (код ошибки)

    // Очистка экрана
    for row in 0..25 {
        for col in 0..80 {
            vga_put(row, col, b' ', ATTR_BG);
        }
    }

    // Рамка (верх/низ = '=', бока = '|')
    for col in 0..80 {
        vga_put(0, col, b'=', ATTR_FRAME);
        vga_put(24, col, b'=', ATTR_FRAME);
    }
    for row in 1..24 {
        vga_put(row, 0, b'|', ATTR_FRAME);
        vga_put(row, 79, b'|', ATTR_FRAME);
    }

    // Заголовок
    vga_write(0, 22, b"NeroShizaOS KERNEL EVENT", ATTR_FRAME);

    // Тип события
    vga_write(2, 3, b"BITE  [KERNEL WATCHDOG SCREEN]", ATTR_BG);

    // Имя укуса
    let name = bite_name(code).as_bytes();
    vga_write(4, 3, name, ATTR_HI);

    // Вспомогательная функция: u64 → decimal в буфер, возвращает длину
    fn fmt_u64_dec(mut n: u64, buf: &mut [u8; 20]) -> usize {
        if n == 0 {
            buf[0] = b'0';
            return 1;
        }
        let mut tmp = [0u8; 20];
        let mut len = 0usize;
        while n > 0 {
            tmp[len] = b'0' + (n % 10) as u8;
            n /= 10;
            len += 1;
        }
        for i in 0..len {
            buf[i] = tmp[len - 1 - i];
        }
        len
    }

    // Код укуса: "CODE: 0x..."
    vga_write(6, 3, b"CODE: 0x", ATTR_BG);
    vga_hex64(6, 11, code, ATTR_WARN);

    // Режим огня: "FIRE: MODE 1 / CLASSIC"
    let fire_len = unsafe { BITE_FIRE_MODE_LEN };
    let mut fire_buf = [0u8; 32];
    unsafe {
        fire_buf.copy_from_slice(&*core::ptr::addr_of!(BITE_FIRE_MODE));
    }
    vga_write(7, 3, b"FIRE: ", ATTR_BG);
    vga_write(7, 9, &fire_buf[..fire_len], ATTR_WARN);

    // Кол-во кадров: "FRAMES: N"
    let fc = unsafe { FRAME_COUNT };
    vga_write(8, 3, b"FRAMES: ", ATTR_BG);
    {
        let mut buf = [0u8; 20];
        let l = fmt_u64_dec(fc, &mut buf);
        vga_write(8, 11, &buf[..l], ATTR_WARN);
    }

    // Таймаут: "ELAPSED: N > N"
    let (elapsed, timeout) = unsafe { (BITE_ELAPSED, BITE_TIMEOUT) };
    vga_write(9, 3, b"ELAPSED: ", ATTR_BG);
    {
        let mut buf = [0u8; 20];
        let l = fmt_u64_dec(elapsed, &mut buf);
        vga_write(9, 12, &buf[..l], ATTR_WARN);
    }
    vga_write(9, 33, b" > ", ATTR_BG);
    {
        let mut buf = [0u8; 20];
        let l = fmt_u64_dec(timeout, &mut buf);
        vga_write(9, 36, &buf[..l], ATTR_WARN);
    }

    // Trace — последние действия (как в DF-экране)
    vga_write(11, 3, b"Last actions:", ATTR_BG);
    let trace_len = crate::trace::len();
    let mut row = 12usize;
    if trace_len == 0 {
        vga_write(row, 5, b"- (none)", ATTR_HI);
    } else {
        let first = trace_len.saturating_sub(8);
        for i in first..trace_len {
            if row >= 22 {
                break;
            }
            if let Some(entry) = crate::trace::get_recent(i) {
                vga_write(row, 5, b"- ", ATTR_BG);
                vga_write(row, 7, entry.as_bytes(), ATTR_BG);
                row += 1;
            }
        }
    }

    vga_write(23, 17, b"System halted. Check serial.log", ATTR_BG);

    x86_64::instructions::interrupts::disable();
    loop {
        x86_64::instructions::hlt();
    }
}

// ============================================================
// RDTSC
// ============================================================

#[inline(always)]
pub fn rdtsc() -> u64 {
    let lo: u32;
    let hi: u32;
    unsafe {
        asm!(
        "rdtsc",
        out("eax") lo,
        out("edx") hi,
        options(nomem, nostack),
        );
    }
    ((hi as u64) << 32) | lo as u64
}

// ============================================================
// Публичный API
// ============================================================

pub fn init() {
    unsafe {
        LAST_FRAME_TSC = rdtsc();
        FRAME_COUNT = 0;
    }
    log_profile("init");
}

pub fn pet() {
    unsafe {
        LAST_FRAME_TSC = rdtsc();
        FRAME_COUNT += 1;
    }
}

pub fn check() {
    unsafe {
        let now = rdtsc();
        let elapsed = now.wrapping_sub(LAST_FRAME_TSC);
        let timeout_ticks = current_timeout_ticks();
        if elapsed > timeout_ticks {
            BITE_ELAPSED = elapsed;
            BITE_TIMEOUT = timeout_ticks;
            let frame_count = FRAME_COUNT;
            // Сохраняем имя режима огня
            let mode_name = super::vga_graphics::fire_mode_name(super::vga_graphics::fire_mode());
            let mb = mode_name.as_bytes();
            let mlen = mb.len().min((*core::ptr::addr_of!(BITE_FIRE_MODE)).len());
            BITE_FIRE_MODE[..mlen].copy_from_slice(&mb[..mlen]);
            BITE_FIRE_MODE_LEN = mlen;
            crate::serial_println!(
                "[BITE] Watchdog timeout: elapsed={} > {} frames={} mode={} fpu={}",
                elapsed,
                timeout_ticks,
                frame_count,
                mode_name,
                if super::vga_graphics::fire_fpu_enabled() {
                    1
                } else {
                    0
                }
            );
            kernel_bite(BITE_CLASSIC);
        }
    }
}

pub fn wait_frame() {
    unsafe {
        let frame_start = LAST_FRAME_TSC;
        loop {
            let now = rdtsc();
            if now.wrapping_sub(frame_start) >= FRAME_TICKS {
                break;
            }
            x86_64::instructions::hlt();
        }
    }
}

pub fn frame_count() -> u64 {
    unsafe { FRAME_COUNT }
}

pub fn effective_timeout_ticks() -> u64 {
    current_timeout_ticks()
}

pub fn log_profile(reason: &str) {
    crate::serial_println!(
        "[DOOM][WDOG] profile reason={} frame_ticks={} timeout_ticks={} frames={} mode={} fpu={}",
        reason,
        FRAME_TICKS,
        current_timeout_ticks(),
        frame_count(),
        super::vga_graphics::fire_mode_name(super::vga_graphics::fire_mode()),
        if super::vga_graphics::fire_fpu_enabled() {
            1
        } else {
            0
        }
    );
}

#[inline(always)]
fn current_timeout_ticks() -> u64 {
    if super::vga_graphics::fire_fpu_enabled() {
        FPU_TIMEOUT_TICKS
    } else {
        TIMEOUT_TICKS
    }
}
