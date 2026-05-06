// ============================================================
// VGA GRAPHICS — Mode 13h (320x200, 256 цветов) + Doom Fire
// ============================================================
// Физика: классический целочисленный Doom Fire (FIRE_BUF u8, 64KB)
// для режимов Classic/Dual/Inferno, f32 in-place симуляция (HEAT_BUF,
// 256KB) для FpuNoise. Нет двойного буфера HEAT_NEXT — устранён
// источник торможения (-256KB статики, нет double fault).
// ============================================================

use crate::vga_hw;
use core::arch::asm;
use x86_64::instructions::port::Port;

// ─────────────────────────────────────────────────────────────
// Константы экрана
// ─────────────────────────────────────────────────────────────

pub const SCREEN_W: usize = 320;
pub const SCREEN_H: usize = 200;
pub const FIRE_SIDE_WIDTH: usize = 56;
pub const FIRE_LEVELS: usize = 37;
pub const FIRE_MAX_HEIGHT: usize = 80;

/// Физический адрес VGA graphics framebuffer (Mode 13h, identity-mapped)
const VGA_FB: *mut u8 = 0xA0000 as *mut u8;

// ─────────────────────────────────────────────────────────────
// FireMode — публичный API (имена совместимы с mod.rs)
// ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FireMode {
    Classic = 1,   // нижняя строка -> вверх, PSX-подобный разброс
    Dual = 2,      // нижняя + верхняя строки
    Inferno = 3,   // все четыре края (OmniBorder)
    FpuNoise = 4,  // f32 in-place симуляция с шумом
    WindLeft = 5,  // ветер влево
    WindRight = 6, // ветер вправо
}

static mut FIRE_MODE: FireMode = FireMode::Classic;
static mut FIRE_STEP_ACC: u8 = 0;
static mut PHYSICS_SKIP: u8 = 2;
static mut FPU_ENABLED: bool = false; // \\ — включает FPU-физику поверх текущего режима

// ─────────────────────────────────────────────────────────────
// Палитра огня (37 уровней, значения VGA DAC 0..63)
// Спектр намеренно многоцветный: синий -> зелёный -> оранжевый -> красный -> белый.
// ─────────────────────────────────────────────────────────────

static FIRE_PAL: [[u8; 3]; FIRE_LEVELS] = [
    [0x00, 0x00, 0x00], // 0
    [0x00, 0x00, 0x06], // 1
    [0x00, 0x00, 0x0A], // 2
    [0x00, 0x02, 0x10], // 3
    [0x00, 0x04, 0x16], // 4
    [0x00, 0x08, 0x1C], // 5
    [0x00, 0x0C, 0x22], // 6
    [0x00, 0x10, 0x28], // 7
    [0x00, 0x14, 0x2E], // 8
    [0x00, 0x18, 0x22], // 9
    [0x00, 0x1C, 0x1C], // 10
    [0x00, 0x20, 0x18], // 11
    [0x04, 0x24, 0x14], // 12
    [0x08, 0x28, 0x10], // 13
    [0x0C, 0x2C, 0x0C], // 14
    [0x10, 0x30, 0x08], // 15
    [0x16, 0x34, 0x04], // 16
    [0x1C, 0x30, 0x00], // 17
    [0x22, 0x2C, 0x00], // 18
    [0x28, 0x28, 0x00], // 19
    [0x30, 0x24, 0x00], // 20
    [0x38, 0x20, 0x00], // 21
    [0x3F, 0x1C, 0x00], // 22
    [0x3F, 0x20, 0x00], // 23
    [0x3F, 0x24, 0x00], // 24
    [0x3F, 0x28, 0x00], // 25
    [0x3F, 0x2C, 0x00], // 26
    [0x3F, 0x20, 0x04], // 27
    [0x3F, 0x18, 0x08], // 28
    [0x3F, 0x12, 0x0C], // 29
    [0x3F, 0x0C, 0x10], // 30
    [0x3F, 0x08, 0x14], // 31
    [0x3F, 0x10, 0x18], // 32
    [0x3F, 0x18, 0x1C], // 33
    [0x3F, 0x24, 0x24], // 34
    [0x3F, 0x30, 0x30], // 35
    [0x3F, 0x3F, 0x3F], // 36
];

#[inline(always)]
fn fire_rgb(level: u8) -> (u8, u8, u8) {
    let entry = FIRE_PAL[level.min((FIRE_LEVELS - 1) as u8) as usize];
    (
        ((entry[0] as u16 * 255) / 63) as u8,
        ((entry[1] as u16 * 255) / 63) as u8,
        ((entry[2] as u16 * 255) / 63) as u8,
    )
}

// ─────────────────────────────────────────────────────────────
// Буферы огня
// ─────────────────────────────────────────────────────────────

// Целочисленный буфер (Classic/Dual/Inferno) — 64 KB
static mut FIRE_BUF: [u8; SCREEN_W * SCREEN_H] = [0; SCREEN_W * SCREEN_H];

// f32 буфер (только FpuNoise, in-place) — 256 KB
// Нет HEAT_NEXT — устранён источник торможения и двойного исключения
static mut HEAT_BUF: [f32; SCREEN_W * SCREEN_H] = [0.0; SCREEN_W * SCREEN_H];

// ─────────────────────────────────────────────────────────────
// ГСЧ (LCG + простой линейный)
// ─────────────────────────────────────────────────────────────

static mut LCG_STATE: u64 = 0xDEAD_BEEF_1234_5678;
static mut RAND_STATE: u32 = 1;

#[inline(always)]
unsafe fn lcg_reseed() {
    let mut lo: u32 = 0;
    let mut hi: u32 = 0;
    asm!("rdtsc", out("eax") lo, out("edx") hi, options(nomem, nostack));
    LCG_STATE ^= ((hi as u64) << 32) | (lo as u64);
}

#[inline(always)]
unsafe fn rand_f32() -> f32 {
    LCG_STATE = LCG_STATE
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    ((LCG_STATE >> 33) as f32) * (1.0 / 2_147_483_648.0)
}

#[inline(always)]
unsafe fn fast_rand() -> u32 {
    RAND_STATE = RAND_STATE.wrapping_mul(1_103_515_245).wrapping_add(12345);
    (RAND_STATE >> 16) & 0x7FFF
}

// ─────────────────────────────────────────────────────────────
// Бэкап plane 2 (шрифт VGA) — 256 символов x 32 байта = 8 KB
// Mode 13h (chain-4) перезаписывает plane 2 огнём — без бэкапа
// box-drawing символы портятся после выхода.
// ─────────────────────────────────────────────────────────────

static mut FONT_BACKUP: [u8; 256 * 32] = [0; 256 * 32];
static mut FONT_SAVED: bool = false;

/// Сохраняет все 256 символов plane 2 перед входом в Mode 13h.
pub unsafe fn save_font_plane() {
    vga_hw::enter_font_mode();
    let base = 0xA0000 as *const u8;
    for i in 0..(256 * 32) {
        FONT_BACKUP[i] = core::ptr::read_volatile(base.add(i));
    }
    vga_hw::exit_font_mode();
    FONT_SAVED = true;
}

/// Восстанавливает plane 2 из бэкапа после restore_text_mode().
pub unsafe fn restore_font_plane() {
    if !FONT_SAVED {
        return;
    }
    vga_hw::enter_font_mode();
    let base = 0xA0000 as *mut u8;
    for i in 0..(256 * 32) {
        core::ptr::write_volatile(base.add(i), FONT_BACKUP[i]);
    }
    vga_hw::exit_font_mode();
}

// ─────────────────────────────────────────────────────────────
// VGA Mode 13h — переключение без BIOS (long mode compatible)
// ─────────────────────────────────────────────────────────────

/// # Safety
/// Перепрограммирует VGA-регистры для Mode 13h (320x200, 256 цветов).
/// После вызова 0xA0000..0xAF9FF — линейный фреймбуфер 8bpp.
pub unsafe fn set_mode_13h() {
    Port::<u8>::new(0x3C2).write(0x63);

    let seq: &[(u8, u8)] = &[
        (0x00, 0x03),
        (0x01, 0x01),
        (0x02, 0x0F),
        (0x03, 0x00),
        (0x04, 0x0E),
    ];
    for &(i, d) in seq {
        vga_hw::write_seq(i, d);
    }

    vga_hw::write_reg(0x3D4, 0x3D5, 0x11, 0x0E); // снимаем защиту CRTC 0-7

    let crtc: &[(u8, u8)] = &[
        (0x00, 0x5F),
        (0x01, 0x4F),
        (0x02, 0x50),
        (0x03, 0x82),
        (0x04, 0x54),
        (0x05, 0x80),
        (0x06, 0xBF),
        (0x07, 0x1F),
        (0x08, 0x00),
        (0x09, 0x41),
        (0x0A, 0x00),
        (0x0B, 0x00),
        (0x0C, 0x00),
        (0x0D, 0x00),
        (0x0E, 0x00),
        (0x0F, 0x00),
        (0x10, 0x9C),
        (0x11, 0x8E),
        (0x12, 0x8F),
        (0x13, 0x28),
        (0x14, 0x40),
        (0x15, 0x96),
        (0x16, 0xB9),
        (0x17, 0xA3),
        (0x18, 0xFF),
    ];
    for &(i, d) in crtc {
        vga_hw::write_reg(0x3D4, 0x3D5, i, d);
    }

    let gc: &[(u8, u8)] = &[
        (0x00, 0x00),
        (0x01, 0x00),
        (0x02, 0x00),
        (0x03, 0x00),
        (0x04, 0x00),
        (0x05, 0x40),
        (0x06, 0x05),
        (0x07, 0x0F),
        (0x08, 0xFF),
    ];
    for &(i, d) in gc {
        vga_hw::write_gc(i, d);
    }

    let _ = Port::<u8>::new(0x3DA).read(); // сброс AC flip-flop

    let attr: &[(u8, u8)] = &[
        (0x00, 0x00),
        (0x01, 0x01),
        (0x02, 0x02),
        (0x03, 0x03),
        (0x04, 0x04),
        (0x05, 0x05),
        (0x06, 0x06),
        (0x07, 0x07),
        (0x08, 0x08),
        (0x09, 0x09),
        (0x0A, 0x0A),
        (0x0B, 0x0B),
        (0x0C, 0x0C),
        (0x0D, 0x0D),
        (0x0E, 0x0E),
        (0x0F, 0x0F),
        (0x10, 0x41),
        (0x11, 0x00),
        (0x12, 0x0F),
        (0x13, 0x00),
        (0x14, 0x00),
    ];
    for &(i, d) in attr {
        vga_hw::write_ac(i, d);
    }

    Port::<u8>::new(0x3C0).write(0x20); // PAS = 1, включаем вывод
}

/// # Safety
/// Восстанавливает Mode 3h (80x25 текст) через vga_hw::restore_text_mode().
pub unsafe fn restore_text_mode() {
    vga_hw::restore_text_mode();
}

// ─────────────────────────────────────────────────────────────
// Палитра DAC
// ─────────────────────────────────────────────────────────────

/// # Safety
/// Записывает FIRE_PAL (37 цветов) в VGA DAC. Вызывать только в Mode 13h.
pub unsafe fn set_fire_palette() {
    let mut pal_idx: Port<u8> = Port::new(0x3C8);
    let mut pal_dat: Port<u8> = Port::new(0x3C9);
    pal_idx.write(0);
    for entry in &FIRE_PAL {
        pal_dat.write(entry[0]);
        pal_dat.write(entry[1]);
        pal_dat.write(entry[2]);
    }
}

pub unsafe fn set_palette_color(index: u8, r: u8, g: u8, b: u8) {
    let mut pal_idx: Port<u8> = Port::new(0x3C8);
    let mut pal_dat: Port<u8> = Port::new(0x3C9);
    pal_idx.write(index);
    pal_dat.write(r & 0x3F);
    pal_dat.write(g & 0x3F);
    pal_dat.write(b & 0x3F);
}

// ─────────────────────────────────────────────────────────────
// Пиксели / очистка
// ─────────────────────────────────────────────────────────────

#[inline(always)]
pub unsafe fn put_pixel(x: usize, y: usize, color: u8) {
    debug_assert!(x < SCREEN_W && y < SCREEN_H);
    core::ptr::write_volatile(VGA_FB.add(y * SCREEN_W + x), color);
}

pub unsafe fn clear_fb(color: u8) {
    for i in 0..(SCREEN_W * SCREEN_H) {
        core::ptr::write_volatile(VGA_FB.add(i), color);
    }
}

pub unsafe fn blit(fb: &[u8; SCREEN_W * SCREEN_H]) {
    for (i, &byte) in fb.iter().enumerate() {
        core::ptr::write_volatile(VGA_FB.add(i), byte);
    }
}

// ─────────────────────────────────────────────────────────────
// Классический целочисленный Doom Fire (Classic/Dual/Inferno)
// Алгоритм: случайное рассеивание пикселей снизу вверх.
// Нет FPU — работает быстро на bare-metal x86.
// ─────────────────────────────────────────────────────────────

unsafe fn seed_classic(mode: FireMode) {
    let bottom = (SCREEN_H - 1) * SCREEN_W;
    let top = SCREEN_H - FIRE_MAX_HEIGHT;
    match mode {
        FireMode::Classic | FireMode::FpuNoise | FireMode::WindLeft | FireMode::WindRight => {
            for x in 0..SCREEN_W {
                FIRE_BUF[bottom + x] = (FIRE_LEVELS - 1) as u8;
            }
        }
        FireMode::Dual => {
            for x in 0..SCREEN_W {
                FIRE_BUF[bottom + x] = (FIRE_LEVELS - 1) as u8;
                FIRE_BUF[x] = (FIRE_LEVELS - 1) as u8;
            }
        }
        FireMode::Inferno => {
            for x in 0..SCREEN_W {
                FIRE_BUF[bottom + x] = (FIRE_LEVELS - 1) as u8;
                FIRE_BUF[x] = (FIRE_LEVELS - 1) as u8;
            }
            for y in top..SCREEN_H {
                FIRE_BUF[y * SCREEN_W] = (FIRE_LEVELS - 1) as u8;
                FIRE_BUF[y * SCREEN_W + SCREEN_W - 1] = (FIRE_LEVELS - 1) as u8;
            }
        }
    }
}

#[inline(always)]
unsafe fn wrap_x(x: usize, delta: isize) -> usize {
    ((x as isize + delta).rem_euclid(SCREEN_W as isize)) as usize
}

#[inline(always)]
unsafe fn classic_drift(mode: FireMode) -> (isize, u8) {
    let rand = (fast_rand() & 3) as u8;
    let decay = rand & 1;
    let dx = match mode {
        FireMode::Classic | FireMode::Dual | FireMode::Inferno | FireMode::FpuNoise => {
            1 - rand as isize
        }
        FireMode::WindLeft => -1 - (rand as isize),
        FireMode::WindRight => rand as isize,
    };
    (dx, decay)
}

unsafe fn fire_update_classic(mode: FireMode) {
    let top = SCREEN_H - FIRE_MAX_HEIGHT;
    for y in (top + 1..SCREEN_H).rev() {
        for x in 0..SCREEN_W {
            let src = y * SCREEN_W + x;
            let pixel = FIRE_BUF[src];
            if pixel == 0 {
                FIRE_BUF[src - SCREEN_W] = 0;
            } else {
                let (dx, decay) = classic_drift(mode);
                let dst_x = wrap_x(x, dx);
                FIRE_BUF[(y - 1) * SCREEN_W + dst_x] = pixel.saturating_sub(decay);
            }
        }
    }
    seed_classic(mode);
}

// ─────────────────────────────────────────────────────────────
// FPU-режим: f32 in-place симуляция с диффузией и конвекцией.
// Используется temp_row [f32; 320] = 1.28 KB на стеке (нет HEAT_NEXT).
// ─────────────────────────────────────────────────────────────

const COOLING: f32 = 0.055;
const DIFFUSION: f32 = 0.2;
const CONVECTION: f32 = 0.5;
const MAX_HEAT: f32 = 1.0;
const SEED_HEAT: f32 = 1.0;
const SEED_HEAT_SIDE: f32 = 0.65;

#[inline(always)]
fn fclamp(v: f32) -> f32 {
    if v < 0.0 {
        0.0
    } else if v > MAX_HEAT {
        MAX_HEAT
    } else {
        v
    }
}

unsafe fn seed_fpu(mode: FireMode) {
    let bottom = (SCREEN_H - 1) * SCREEN_W;
    let top = SCREEN_H - FIRE_MAX_HEIGHT;
    match mode {
        FireMode::FpuNoise | FireMode::Classic | FireMode::WindLeft | FireMode::WindRight => {
            for x in 0..SCREEN_W {
                HEAT_BUF[bottom + x] = SEED_HEAT;
            }
        }
        FireMode::Dual => {
            for x in 0..SCREEN_W {
                HEAT_BUF[bottom + x] = SEED_HEAT;
                HEAT_BUF[x] = SEED_HEAT;
            }
        }
        FireMode::Inferno => {
            for x in 0..SCREEN_W {
                HEAT_BUF[bottom + x] = SEED_HEAT;
                HEAT_BUF[x] = SEED_HEAT;
            }
            for y in top..SCREEN_H {
                HEAT_BUF[y * SCREEN_W] = SEED_HEAT_SIDE;
                HEAT_BUF[y * SCREEN_W + SCREEN_W - 1] = SEED_HEAT_SIDE;
            }
        }
    }
}

unsafe fn fire_update_fpu(mode: FireMode) {
    let top = SCREEN_H - FIRE_MAX_HEIGHT;
    let mut temp_row = [0.0f32; SCREEN_W]; // 1.28 KB на стеке, не статика
    for y in (top..SCREEN_H).rev() {
        let idx = y * SCREEN_W;
        temp_row.copy_from_slice(&HEAT_BUF[idx..idx + SCREEN_W]);
        for x in 0..SCREEN_W {
            let is_seed = match mode {
                FireMode::FpuNoise
                | FireMode::Classic
                | FireMode::WindLeft
                | FireMode::WindRight => y == SCREEN_H - 1,
                FireMode::Dual => y == 0 || y == SCREEN_H - 1,
                FireMode::Inferno => y == 0 || y == SCREEN_H - 1 || x == 0 || x == SCREEN_W - 1,
            };
            if is_seed {
                continue;
            }
            let c = temp_row[x];
            let l = if x > 0 { temp_row[x - 1] } else { c };
            let r = if x < SCREEN_W - 1 { temp_row[x + 1] } else { c };
            let b = if y < SCREEN_H - 1 {
                HEAT_BUF[(y + 1) * SCREEN_W + x]
            } else {
                c
            };
            let avg = (c + l + r + b) * 0.25;
            let diffused = c * (1.0 - DIFFUSION) + avg * DIFFUSION;
            let below_y = (y + 1).min(SCREEN_H - 1);
            let (drift_x, noise) = match mode {
                FireMode::WindLeft => {
                    let drift = x.saturating_sub(1 + (fast_rand() as usize & 1));
                    (drift, (rand_f32() - 0.5) * 0.035)
                }
                FireMode::WindRight => {
                    let drift = (x + 1 + (fast_rand() as usize & 1)).min(SCREEN_W - 1);
                    (drift, (rand_f32() - 0.5) * 0.035)
                }
                _ => {
                    let rf = rand_f32();
                    let drift = if rf < 0.33 {
                        x.saturating_sub(1)
                    } else if rf > 0.66 {
                        (x + 1).min(SCREEN_W - 1)
                    } else {
                        x
                    };
                    (drift, (rand_f32() - 0.5) * 0.05)
                }
            };
            let below_val = HEAT_BUF[below_y * SCREEN_W + drift_x];
            let convected = diffused * (1.0 - CONVECTION) + below_val * CONVECTION;
            HEAT_BUF[idx + x] = fclamp(convected * (1.0 - COOLING) + noise);
        }
    }
    seed_fpu(mode);
}

// ─────────────────────────────────────────────────────────────
// Публичный API огня
// ─────────────────────────────────────────────────────────────

pub fn fire_set_mode(mode: FireMode) {
    unsafe {
        FIRE_MODE = mode;
        fire_init();
    }
}

/// Переключает FPU-физику для текущего режима (клавиша \).
/// FPU ON = f32 диффузия + конвекция + шум поверх Classic/Dual/Inferno.
/// FPU OFF = чистый целочисленный алгоритм.
pub fn fire_toggle_fpu() {
    unsafe {
        FPU_ENABLED = !FPU_ENABLED;
    }
}

pub fn fire_fpu_enabled() -> bool {
    unsafe { FPU_ENABLED }
}

pub fn fire_mode() -> FireMode {
    unsafe { FIRE_MODE }
}

pub fn fire_mode_name(mode: FireMode) -> &'static str {
    match mode {
        FireMode::Classic => "MODE 1 / CLASSIC",
        FireMode::Dual => "MODE 2 / TOP+BOTTOM",
        FireMode::Inferno => "MODE 3 / ALL SIDES",
        FireMode::FpuNoise => "MODE 4 / FPU NOISE",
        FireMode::WindLeft => "MODE 5 / WIND LEFT",
        FireMode::WindRight => "MODE 6 / WIND RIGHT",
    }
}

/// Инициализирует буферы огня и засевает нижнюю строку.
/// # Safety: вызывать после set_mode_13h().
pub unsafe fn fire_init() {
    for i in 0..(SCREEN_W * SCREEN_H) {
        FIRE_BUF[i] = 0;
        HEAT_BUF[i] = 0.0;
    }
    let bottom = (SCREEN_H - 1) * SCREEN_W;
    for x in 0..SCREEN_W {
        FIRE_BUF[bottom + x] = (FIRE_LEVELS - 1) as u8;
        HEAT_BUF[bottom + x] = 1.0;
    }
    FIRE_STEP_ACC = 0;
    lcg_reseed();
}

/// Засевает пиксель нижней строки — используется chaos::ignite.
/// Пишет в оба буфера чтобы работал и Classic и FpuNoise режим.
pub fn fire_ignite_bottom(x: usize, val: u8) {
    if x >= SCREEN_W {
        return;
    }
    unsafe {
        let bottom = (SCREEN_H - 1) * SCREEN_W;
        FIRE_BUF[bottom + x] = val;
        HEAT_BUF[bottom + x] = val as f32 / (FIRE_LEVELS as f32 - 1.0);
    }
}

/// Шаг физики огня (без рендера). Вызывается mod.rs каждый кадр.
/// PHYSICS_SKIP=2: физика каждые 2 кадра, рендер каждый кадр.
pub fn fire_update() {
    unsafe {
        FIRE_STEP_ACC = FIRE_STEP_ACC.wrapping_add(1);
        if FIRE_STEP_ACC >= PHYSICS_SKIP {
            FIRE_STEP_ACC = 0;
            lcg_reseed();
            let mode = FIRE_MODE;
            if FPU_ENABLED {
                fire_update_fpu(mode);
            } else {
                fire_update_classic(mode);
            }
        }
    }
}

/// Рендер огня во фреймбуфер VGA. Вызывается mod.rs после fire_update().
pub fn fire_render() {
    if crate::fb_buffer::is_initialized() {
        fire_render_framebuffer();
        return;
    }

    unsafe {
        let top = SCREEN_H - FIRE_MAX_HEIGHT;

        // Верхняя зона — чёрная
        for y in 0..top {
            for x in 0..SCREEN_W {
                core::ptr::write_volatile(VGA_FB.add(y * SCREEN_W + x), 0);
            }
        }

        // Зона огня
        if FPU_ENABLED {
            for y in top..SCREEN_H {
                for x in 0..SCREEN_W {
                    let heat = HEAT_BUF[y * SCREEN_W + x];
                    let level =
                        ((heat * (FIRE_LEVELS as f32 - 1.0)) as usize).min(FIRE_LEVELS - 1) as u8;
                    core::ptr::write_volatile(VGA_FB.add(y * SCREEN_W + x), level);
                }
            }
        } else {
            for y in top..SCREEN_H {
                for x in 0..SCREEN_W {
                    core::ptr::write_volatile(
                        VGA_FB.add(y * SCREEN_W + x),
                        FIRE_BUF[y * SCREEN_W + x],
                    );
                }
            }
        }
    }
}

fn fire_render_framebuffer() {
    let fb_w = crate::fb_buffer::pixel_width();
    let fb_h = crate::fb_buffer::pixel_height();
    if fb_w == 0 || fb_h == 0 {
        return;
    }

    let scale_x = (fb_w / SCREEN_W).max(1);
    let scale_y = (fb_h / SCREEN_H).max(1);
    let draw_w = SCREEN_W * scale_x;
    let draw_h = SCREEN_H * scale_y;
    let offset_x = fb_w.saturating_sub(draw_w) / 2;
    let offset_y = fb_h.saturating_sub(draw_h) / 2;

    unsafe {
        for y in 0..SCREEN_H {
            for x in 0..SCREEN_W {
                let level = if y < SCREEN_H - FIRE_MAX_HEIGHT {
                    0
                } else if FPU_ENABLED {
                    ((HEAT_BUF[y * SCREEN_W + x] * (FIRE_LEVELS as f32 - 1.0)) as usize)
                        .min(FIRE_LEVELS - 1) as u8
                } else {
                    FIRE_BUF[y * SCREEN_W + x]
                };

                let (r, g, b) = fire_rgb(level);
                crate::fb_buffer::fill_rect_rgb(
                    offset_x + x * scale_x,
                    offset_y + y * scale_y,
                    scale_x,
                    scale_y,
                    r,
                    g,
                    b,
                );
            }
        }
    }
}
