// FPU (x87) — инициализация и математика через inline assembly
// Rust компилирует с soft-float, поэтому все float-операции через asm!

use core::arch::asm;

/// Инициализация x87 FPU
pub fn init() {
    unsafe {
        let mut cr0: u64;
        asm!("mov {}, cr0", out(reg) cr0);
        cr0 &= !(1 << 2); // EM = 0 (не эмулирует FPU)
        cr0 |= 1 << 1; // MP = 1 (мониторинг WAIT/FWAIT)
        cr0 |= 1 << 5; // NE = 1 (нативные обработчики FP-исключений)
        asm!("mov cr0, {}", in(reg) cr0);
        asm!("fninit"); // Инициализация x87
    }
}

/// log2(x) через x87 fyl2x.
/// fyl2x: ST(0) = ST(1) * log2(ST(0)) — мы ставим y=1, поэтому результат = log2(x).
pub fn log2(x: u64) -> i64 {
    // x передаём как целое, конвертим внутри FPU через fild
    let result: i64;
    unsafe {
        asm!(
            "fld1",                        // ST(0) = 1.0 (ставим y=1)
            "push {x}",
            "fild qword ptr [rsp]",        // ST(0) = x  ST(1) = 1.0
            "add rsp, 8",
            "fyl2x",                       // ST(0) = 1.0 * log2(x)
            "push 0",
            "fistp qword ptr [rsp]",       // конвертируем в i64 → на стек
            "pop {out}",
            x   = in(reg)  x,
            out = out(reg) result,
        );
    }
    result
}

/// sqrt(x) через x87 fsqrt.
pub fn sqrt(x: u64) -> u64 {
    let result: u64;
    unsafe {
        asm!(
            "push {x}",
            "fild qword ptr [rsp]",        // ST(0) = x
            "add rsp, 8",
            "fsqrt",                       // ST(0) = sqrt(x)
            "push 0",
            "fistp qword ptr [rsp]",       // конвертируем в u64 → на стек
            "pop {out}",
            x   = in(reg)  x,
            out = out(reg) result,
        );
    }
    result
}

/// Энтропия Шеннона × 1000 (целое, без float в println).
/// H = log2(len) - (1/len)*sum(count*log2(count))
/// Все x87 операции через указатели — push/pop не используется.
pub fn shannon_entropy(data: &[u8]) -> u64 {
    if data.is_empty() {
        return 0;
    }

    let mut freq = [0u32; 256];
    for &b in data {
        freq[b as usize] += 1;
    }

    let len = data.len() as u64;
    let mut sum_part: i64 = 0;

    for i in 0..256usize {
        let count = freq[i];
        if count == 0 {
            continue;
        }
        let c64: u64 = count as u64;
        let mut part: i64 = 0;
        unsafe {
            asm!(
                "fld1",
                "fild qword ptr [{c}]",   // ST0=count ST1=1.0
                "fyl2x",                   // ST0=log2(count)
                "fild qword ptr [{c}]",   // ST0=count ST1=log2(count)
                "fmulp",                   // ST0=count*log2(count)
                "fistp qword ptr [{o}]",
                c = in(reg) &c64    as *const u64,
                o = in(reg) &mut part as *mut i64,
                options(nostack),
            );
        }
        sum_part += part;
    }

    let k: u64 = 1000;
    let mut log2_len: i64 = 0;
    unsafe {
        asm!(
            "fld1",
            "fild qword ptr [{n}]",   // ST0=len ST1=1.0
            "fyl2x",                   // ST0=log2(len)
            "fild qword ptr [{k}]",   // ST0=1000 ST1=log2(len)
            "fmulp",                   // ST0=log2(len)*1000
            "fistp qword ptr [{o}]",
            n = in(reg) &len as *const u64,
            k = in(reg) &k   as *const u64,
            o = in(reg) &mut log2_len as *mut i64,
            options(nostack),
        );
    }

    let scaled = (sum_part * 1000) / (len as i64);
    let h = log2_len - scaled;
    if h > 0 { h as u64 } else { 0 }
}

// ── TUI helpers ──────────────────────────────────────────────────────────────

const VGA: *mut u8 = 0xb8000 as *mut u8;

unsafe fn vput(row: usize, col: usize, ch: u8, color: u8) {
    if row >= 25 || col >= 80 {
        return;
    }
    let off = (row * 80 + col) * 2;
    core::ptr::write_volatile(VGA.add(off), ch);
    core::ptr::write_volatile(VGA.add(off + 1), color);
}

unsafe fn vfill(row: usize, col: usize, len: usize, color: u8) {
    for c in 0..len {
        vput(row, col + c, b' ', color);
    }
}

unsafe fn vputs(row: usize, col: usize, s: &[u8], color: u8) {
    for (i, &b) in s.iter().enumerate() {
        vput(row, col + i, b, color);
    }
}

unsafe fn vfill_screen(color: u8) {
    for r in 0..25usize {
        vfill(r, 0, 80, color);
    }
}

// CP437 box
const TL: u8 = 0xC9;
const TR: u8 = 0xBB;
const BL: u8 = 0xC8;
const BR: u8 = 0xBC;
const HZ: u8 = 0xCD;
const VT: u8 = 0xBA;
const ML: u8 = 0xCC;
const MR: u8 = 0xB9;

// Цвета: жёлтая тема (как у FPU — математика/наука)
const BG: u8 = 0x00; // Black on Black
const BORDER: u8 = 0x0E; // Yellow on Black
const TITLE: u8 = 0x0B; // Cyan on Black
const LABEL: u8 = 0x07; // LtGray on Black
const VAL: u8 = 0x0A; // LtGreen on Black
const HINT: u8 = 0x08; // DkGray on Black
const HDR: u8 = 0x0E; // Yellow on Black

const BOX_COL: usize = 14;
const BOX_W: usize = 51;
const BOX_ROW: usize = 1;

unsafe fn draw_box(top: usize, w: usize, h: usize) {
    vput(top, BOX_COL, TL, BORDER);
    for c in 1..=w {
        vput(top, BOX_COL + c, HZ, BORDER);
    }
    vput(top, BOX_COL + w + 1, TR, BORDER);
    for r in 1..h {
        vput(top + r, BOX_COL, VT, BORDER);
        vput(top + r, BOX_COL + w + 1, VT, BORDER);
    }
    vput(top + h, BOX_COL, BL, BORDER);
    for c in 1..=w {
        vput(top + h, BOX_COL + c, HZ, BORDER);
    }
    vput(top + h, BOX_COL + w + 1, BR, BORDER);
}

unsafe fn divider(row: usize) {
    vput(row, BOX_COL, ML, BORDER);
    for c in 1..=BOX_W {
        vput(row, BOX_COL + c, HZ, BORDER);
    }
    vput(row, BOX_COL + BOX_W + 1, MR, BORDER);
}

unsafe fn row_blank(row: usize) {
    vput(row, BOX_COL, VT, BORDER);
    vfill(row, BOX_COL + 1, BOX_W, BG);
    vput(row, BOX_COL + BOX_W + 1, VT, BORDER);
}

// Форматирование u64 без alloc
fn fmt_u64(buf: &mut [u8; 20], val: u64) -> usize {
    if val == 0 {
        buf[0] = b'0';
        return 1;
    }
    let mut tmp = [0u8; 20];
    let mut n = 0;
    let mut v = val;
    while v > 0 {
        tmp[n] = b'0' + (v % 10) as u8;
        v /= 10;
        n += 1;
    }
    for i in 0..n {
        buf[i] = tmp[n - 1 - i];
    }
    n
}

fn fmt_i64(buf: &mut [u8; 20], val: i64) -> usize {
    if val < 0 {
        buf[0] = b'-';
        let mut b2 = [0u8; 20];
        let n = fmt_u64(&mut b2, (-val) as u64);
        buf[1..1 + n].copy_from_slice(&b2[..n]);
        1 + n
    } else {
        fmt_u64(buf, val as u64)
    }
}

unsafe fn row_kv_u64(row: usize, label: &[u8], val: u64) {
    vput(row, BOX_COL, VT, BORDER);
    vfill(row, BOX_COL + 1, BOX_W, BG);
    vputs(row, BOX_COL + 3, label, LABEL);
    let mut buf = [0u8; 20];
    let n = fmt_u64(&mut buf, val);
    vputs(row, BOX_COL + 3 + label.len() + 1, &buf[..n], VAL);
    vput(row, BOX_COL + BOX_W + 1, VT, BORDER);
}

unsafe fn row_entropy(row: usize, label: &[u8], milli: u64) {
    vput(row, BOX_COL, VT, BORDER);
    vfill(row, BOX_COL + 1, BOX_W, BG);
    vputs(row, BOX_COL + 3, label, LABEL);
    let int_part = milli / 1000;
    let frac = milli % 1000;
    let mut b1 = [0u8; 20];
    let n1 = fmt_u64(&mut b1, int_part);
    let mut b2 = [0u8; 20];
    let n2 = fmt_u64(&mut b2, frac);
    let col = BOX_COL + 3 + label.len() + 1;
    vputs(row, col, &b1[..n1], VAL);
    vput(row, col + n1, b'.', VAL);
    // дополняем дробную часть нулями до 3 цифр
    let frac_start = col + n1 + 1;
    if frac < 100 {
        vput(row, frac_start, b'0', VAL);
    }
    if frac < 10 {
        vput(row, frac_start + 1, b'0', VAL);
    }
    let pad = if frac < 10 {
        2
    } else if frac < 100 {
        1
    } else {
        0
    };
    vputs(row, frac_start + pad, &b2[..n2], VAL);
    vputs(row, frac_start + pad + n2, b" bits", VAL);
    vput(row, BOX_COL + BOX_W + 1, VT, BORDER);
}

// RDTSC-based delay — calibrated against the same QEMU TSC scale as doom/watchdog.rs.
// Raised for the FPU demo so its staged output gets more breathing room on slower hosts.
// FRAME_TICKS = 70_000_000 ≈ 70 ms at the ~1 GHz QEMU virtual TSC.
// Using RDTSC avoids dependence on loop-iteration throughput, which
// varies wildly between QEMU TCG / WHPX / KVM backends.
const FRAME_TICKS: u64 = 70_000_000; // ~70 ms @ 1 GHz QEMU TSC

#[inline(always)]
unsafe fn rdtsc() -> u64 {
    let lo: u32;
    let hi: u32;
    asm!("rdtsc", out("eax") lo, out("edx") hi, options(nomem, nostack));
    ((hi as u64) << 32) | lo as u64
}

unsafe fn rdtsc_delay(ticks: u64) {
    let start = rdtsc();
    loop {
        if rdtsc().wrapping_sub(start) >= ticks {
            break;
        }
        asm!("pause", options(nomem, nostack));
    }
}

unsafe fn draw_spinner_row(row: usize, frame: u8) {
    let ch = match frame & 3 {
        0 => b'|',
        1 => b'/',
        2 => b'-',
        _ => b'\\',
    };
    vput(row, BOX_COL, VT, BORDER);
    vfill(row, BOX_COL + 1, BOX_W, BG);
    vputs(row, BOX_COL + 3, b"Calculating x87 FPU results... ", HINT);
    vput(row, BOX_COL + 35, ch, 0x0E);
    vput(row, BOX_COL + BOX_W + 1, VT, BORDER);
}

unsafe fn draw_row_placeholder(row: usize) {
    vput(row, BOX_COL, VT, BORDER);
    vfill(row, BOX_COL + 1, BOX_W, BG);
    vput(row, BOX_COL + BOX_W + 1, VT, BORDER);
}

/// Демо FPU: TUI-экран с рамкой. Esc = возврат в APPS.
pub fn demo() {
    unsafe {
        vfill_screen(BG);

        // Breadcrumb
        vputs(0, 2, b"[ NeroShizaDev-OS ]", 0x0E);
        vputs(0, 21, b" \xB7 APPS \xB7 FPU", 0x0B);

        // Box (высота = 17 строк)
        draw_box(BOX_ROW, BOX_W, 17);

        // Title row
        let h = BOX_ROW + 1;
        vput(h, BOX_COL, VT, BORDER);
        vfill(h, BOX_COL + 1, BOX_W, 0x00);
        vputs(
            h,
            BOX_COL + 12,
            b"x87 FPU  \xB7  log2 / sqrt / entropy",
            HDR,
        );
        vput(h, BOX_COL + BOX_W + 1, VT, BORDER);

        divider(BOX_ROW + 2);

        // Blank placeholders for all result rows
        for r in (BOX_ROW + 3)..=(BOX_ROW + 16) {
            draw_row_placeholder(r);
        }

        // ── Phase 1: spinner in the result area ──────────────────────────────
        let spin_row = BOX_ROW + 7; // middle of result area
        for frame in 0u8..16 {
            draw_spinner_row(spin_row, frame);
            rdtsc_delay(FRAME_TICKS);
        }
        // Erase spinner row
        draw_row_placeholder(spin_row);

        // ── Phase 2: compute everything ──────────────────────────────────────
        let s1 = sqrt(144);
        let s2 = sqrt(1_000_000);
        let l1 = log2(256);
        let l2 = log2(1024);
        let low_entropy = b"AAAAAAAAAAAAAAAA";
        let high_entropy = b"abcdefghijklmnop";
        let h_low = shannon_entropy(low_entropy);
        let h_high = shannon_entropy(high_entropy);

        // ── Phase 3: reveal each row with a brief delay ───────────────────────

        // Section: SQRT
        let r = BOX_ROW + 3;
        vput(r, BOX_COL, VT, BORDER);
        vfill(r, BOX_COL + 1, BOX_W, BG);
        vputs(r, BOX_COL + 3, b"sqrt(144)   =", LABEL);
        let mut b = [0u8; 20];
        let n = fmt_u64(&mut b, s1);
        vputs(r, BOX_COL + 17, &b[..n], VAL);
        vput(r, BOX_COL + BOX_W + 1, VT, BORDER);
        rdtsc_delay(FRAME_TICKS);

        let r = BOX_ROW + 4;
        vput(r, BOX_COL, VT, BORDER);
        vfill(r, BOX_COL + 1, BOX_W, BG);
        vputs(r, BOX_COL + 3, b"sqrt(1000000)=", LABEL);
        let mut b = [0u8; 20];
        let n = fmt_u64(&mut b, s2);
        vputs(r, BOX_COL + 17, &b[..n], VAL);
        vput(r, BOX_COL + BOX_W + 1, VT, BORDER);
        rdtsc_delay(FRAME_TICKS);

        divider(BOX_ROW + 5);

        // Section: LOG2
        let r = BOX_ROW + 6;
        vput(r, BOX_COL, VT, BORDER);
        vfill(r, BOX_COL + 1, BOX_W, BG);
        vputs(r, BOX_COL + 3, b"log2(256)   =", LABEL);
        let mut b = [0u8; 20];
        let n = fmt_i64(&mut b, l1);
        vputs(r, BOX_COL + 17, &b[..n], VAL);
        vput(r, BOX_COL + BOX_W + 1, VT, BORDER);
        rdtsc_delay(FRAME_TICKS);

        let r = BOX_ROW + 7;
        vput(r, BOX_COL, VT, BORDER);
        vfill(r, BOX_COL + 1, BOX_W, BG);
        vputs(r, BOX_COL + 3, b"log2(1024)  =", LABEL);
        let mut b = [0u8; 20];
        let n = fmt_i64(&mut b, l2);
        vputs(r, BOX_COL + 17, &b[..n], VAL);
        vput(r, BOX_COL + BOX_W + 1, VT, BORDER);
        rdtsc_delay(FRAME_TICKS);

        divider(BOX_ROW + 8);

        // Section: ENTROPY
        let r = BOX_ROW + 9;
        vput(r, BOX_COL, VT, BORDER);
        vfill(r, BOX_COL + 1, BOX_W, BG);
        vputs(r, BOX_COL + 3, b"Shannon entropy:", TITLE);
        vput(r, BOX_COL + BOX_W + 1, VT, BORDER);
        rdtsc_delay(FRAME_TICKS);

        row_entropy(BOX_ROW + 10, b"\"AAAA...\" (low) =", h_low);
        rdtsc_delay(FRAME_TICKS);
        row_entropy(BOX_ROW + 11, b"\"abcd...\" (high)=", h_high);
        rdtsc_delay(FRAME_TICKS);

        divider(BOX_ROW + 12);

        // OK row
        row_blank(BOX_ROW + 13);
        let r = BOX_ROW + 14;
        vput(r, BOX_COL, VT, BORDER);
        vfill(r, BOX_COL + 1, BOX_W, BG);
        vputs(
            r,
            BOX_COL + 3,
            b"\x01 All x87 FPU operations completed OK",
            0x0A,
        );
        vput(r, BOX_COL + BOX_W + 1, VT, BORDER);

        row_blank(BOX_ROW + 15);

        // Hint row
        let r = BOX_ROW + 16;
        vput(r, BOX_COL, VT, BORDER);
        vfill(r, BOX_COL + 1, BOX_W, BG);
        vputs(r, BOX_COL + 14, b"Press any key to return to APPS...", HINT);
        vput(r, BOX_COL + BOX_W + 1, VT, BORDER);
    }

    // Drain any scancodes queued during the animation so they don't leak
    // into the Launcher (a stale Esc would pop the Launcher to shell).
    unsafe {
        while crate::ps2::has_scancode() {
            crate::ps2::read_scancode();
        }
    }
    wait_key();
}

fn wait_key() {
    use x86_64::instructions::hlt;
    unsafe {
        loop {
            if crate::ps2::has_scancode() {
                if crate::ps2::read_scancode() & 0x80 == 0 {
                    return;
                }
            }
            hlt();
        }
    }
}
