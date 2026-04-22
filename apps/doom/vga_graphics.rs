/// Максимальная высота распространения огня (от низа экрана)
pub const FIRE_MAX_HEIGHT: usize = 80; // например, 80 строк от низа (можно менять)

// ============================================================
// VGA GRAPHICS — Mode 13h (320×200, 256 цветов)
// ============================================================
// Переключение VGA из текстового Mode 3 в графический Mode 13h
// напрямую через I/O-порты без BIOS (long mode compatible).
//
// Фреймбуфер: физический адрес 0xA0000 (identity-mapped bootloader).
// Порты:
//   0x3C2 — Miscellaneous Output (запись)
//   0x3C4/0x3C5 — Sequencer (index/data)
//   0x3D4/0x3D5 — CRTC (index/data)
//   0x3CE/0x3CF — Graphics Controller (index/data)
//   0x3C0       — Attribute Controller (чередование index/data)
//   0x3DA       — Input Status 1 (чтение сбрасывает AC flip-flop)
//   0x3C8/0x3C9 — DAC Palette (index/data)
// ============================================================

use core::arch::asm;
use x86_64::instructions::port::Port;

use crate::vga_hw;

pub const SCREEN_W: usize = 320;
pub const SCREEN_H: usize = 200;
pub const FIRE_SIDE_WIDTH: usize = 56;

/// Физический адрес VGA graphics framebuffer (Mode 13h)
const VGA_FB: *mut u8 = 0xA0000 as *mut u8;

// ============================================================
// Режим DOOM Fire — 37 уровней яркости (чёрный → красный → белый)
// Палитра из оригинального Doom (делённые на 4 для VGA DAC 0-63)
// ============================================================
pub const FIRE_LEVELS: usize = 37;

/// RGB-компоненты палитры огня (0..63 для VGA DAC)
static FIRE_PAL: [[u8; 3]; FIRE_LEVELS] = [
    [0x00, 0x00, 0x00], // 0  — чёрный фон
    [0x07, 0x01, 0x01], // 1
    [0x0B, 0x03, 0x01], // 2
    [0x11, 0x03, 0x01], // 3
    [0x15, 0x05, 0x01], // 4
    [0x19, 0x07, 0x01], // 5
    [0x1D, 0x07, 0x01], // 6
    [0x23, 0x09, 0x01], // 7
    [0x27, 0x0B, 0x01], // 8
    [0x2B, 0x0F, 0x01], // 9
    [0x2F, 0x11, 0x01], // 10
    [0x31, 0x11, 0x01], // 11
    [0x37, 0x13, 0x01], // 12
    [0x37, 0x15, 0x01], // 13
    [0x37, 0x15, 0x01], // 14
    [0x35, 0x17, 0x01], // 15
    [0x35, 0x17, 0x01], // 16
    [0x35, 0x19, 0x03], // 17
    [0x33, 0x1B, 0x03], // 18
    [0x33, 0x1D, 0x03], // 19
    [0x33, 0x1F, 0x03], // 20
    [0x33, 0x21, 0x05], // 21
    [0x31, 0x21, 0x05], // 22
    [0x31, 0x23, 0x05], // 23
    [0x31, 0x25, 0x07], // 24
    [0x2F, 0x27, 0x07], // 25
    [0x2F, 0x27, 0x07], // 26
    [0x2F, 0x29, 0x09], // 27
    [0x2F, 0x29, 0x09], // 28
    [0x2F, 0x2B, 0x0B], // 29
    [0x2D, 0x2B, 0x0B], // 30
    [0x2D, 0x2D, 0x0B], // 31
    [0x2D, 0x2D, 0x0D], // 32
    [0x33, 0x33, 0x1B], // 33
    [0x37, 0x37, 0x27], // 34
    [0x3B, 0x3B, 0x31], // 35
    [0x3F, 0x3F, 0x3F], // 36 — белый
];

// ============================================================
// Буфер тепла — 320×200 × f32 (симуляция с непрерывными значениями)
// ============================================================
static mut HEAT_BUF:  [f32; SCREEN_W * SCREEN_H] = [0.0; SCREEN_W * SCREEN_H];
static mut HEAT_NEXT: [f32; SCREEN_W * SCREEN_H] = [0.0; SCREEN_W * SCREEN_H];

// Параметры симуляции
const COOLING:   f32 = 0.02; // остывание за кадр
const DIFFUSION: f32 = 0.2;  // сила размытия
const CONVECTION: f32 = 0.5; // подъём тепла вверх
const MAX_HEAT:  f32 = 1.0;  // максимальное тепло
const SEED_HEAT: f32 = 1.0;  // тепло на краях

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FireMode {
    Classic = 1,
    Dual = 2,
    Inferno = 3,
    FpuNoise = 4,
}

static mut FIRE_MODE: FireMode = FireMode::Dual;
static mut FIRE_STEP_ACC: u8 = 0;
static mut LCG_STATE: u64 = 0x_DEAD_BEEF_1234_5678;



// ============================================================
// Бэкап plane 2 (шрифт VGA) — 256 символов × 32 байта = 8 KB
// Mode 13h (chain-4) перезаписывает plane 2 огнём → после выхода
// необходимо полностью восстановить все 256 слотов шрифта.
// ============================================================
static mut FONT_BACKUP: [u8; 256 * 32] = [0; 256 * 32];
static mut FONT_SAVED:  bool = false;

/// Сохраняет все 256 символов plane 2 перед входом в Mode 13h.
pub unsafe fn save_font_plane() {
    crate::vga_hw::enter_font_mode();
    let base = 0xA0000 as *const u8;
    for i in 0..(256 * 32) {
        FONT_BACKUP[i] = core::ptr::read_volatile(base.add(i));
    }
    crate::vga_hw::exit_font_mode();
    FONT_SAVED = true;
}

/// Восстанавливает все 256 символов plane 2 из сохранённого бэкапа.
/// Вызывать ПОСЛЕ restore_text_mode() — тогда регистры уже в Mode 3.
pub unsafe fn restore_font_plane() {
    if !FONT_SAVED { return; }
    crate::vga_hw::enter_font_mode();
    let base = 0xA0000 as *mut u8;
    for i in 0..(256 * 32) {
        core::ptr::write_volatile(base.add(i), FONT_BACKUP[i]);
    }
    crate::vga_hw::exit_font_mode();
}

// ============================================================
// ПЕРЕКЛЮЧЕНИЕ В MODE 13h
// Полный набор регистров для 320×200 256-цветного режима.
// Порядок: MISC → SEQ → CRTC → GC → AC
// ============================================================

/// # Safety
/// Перепрограммирует все VGA-регистры для Mode 13h без BIOS (long mode).
/// Требует: VGA-карта присутствует и identity-mapped загрузчиком.
/// Сразу после вызова 0xA0000..0xAF9FF — линейный фреймбуфер 320×200×8bpp.
/// Нельзя вызывать из обработчика прерывания одновременно с другой записью портов.
pub unsafe fn set_mode_13h() {
    // --- Miscellaneous Output ---
    Port::<u8>::new(0x3C2).write(0x63);

    // --- Sequencer ---
    let seq: &[(u8, u8)] = &[
        (0x00, 0x03), // Reset
        (0x01, 0x01), // Clocking Mode
        (0x02, 0x0F), // Map Mask
        (0x03, 0x00), // Character Map
        (0x04, 0x0E), // Memory Mode
    ];
    for &(i, d) in seq { vga_hw::write_seq(i, d); }

    // --- CRTC: снимаем защиту регистров 0-7 (CRTC[11] bit7=0) ---
    vga_hw::write_reg(0x3D4, 0x3D5, 0x11, 0x0E); // unlock first

    let crtc: &[(u8, u8)] = &[
        (0x00, 0x5F), // Horizontal Total
        (0x01, 0x4F), // H Display End
        (0x02, 0x50), // H Blank Start
        (0x03, 0x82), // H Blank End
        (0x04, 0x54), // H Retrace Start
        (0x05, 0x80), // H Retrace End
        (0x06, 0xBF), // Vertical Total
        (0x07, 0x1F), // Overflow
        (0x08, 0x00), // Preset Row Scan
        (0x09, 0x41), // Max Scan Line
        (0x0A, 0x00), // Cursor Start
        (0x0B, 0x00), // Cursor End
        (0x0C, 0x00), // Start Address High
        (0x0D, 0x00), // Start Address Low
        (0x0E, 0x00), // Cursor Location High
        (0x0F, 0x00), // Cursor Location Low
        (0x10, 0x9C), // V Retrace Start
        (0x11, 0x8E), // V Retrace End (bit7=1 — protect back on)
        (0x12, 0x8F), // V Display End
        (0x13, 0x28), // Offset
        (0x14, 0x40), // Underline Location
        (0x15, 0x96), // V Blank Start
        (0x16, 0xB9), // V Blank End
        (0x17, 0xA3), // Mode Control
        (0x18, 0xFF), // Line Compare
    ];
    for &(i, d) in crtc { vga_hw::write_reg(0x3D4, 0x3D5, i, d); }

    // --- Graphics Controller ---
    let gc: &[(u8, u8)] = &[
        (0x00, 0x00), // Set/Reset
        (0x01, 0x00), // Enable Set/Reset
        (0x02, 0x00), // Color Compare
        (0x03, 0x00), // Data Rotate
        (0x04, 0x00), // Read Map Select
        (0x05, 0x40), // Mode
        (0x06, 0x05), // Miscellaneous
        (0x07, 0x0F), // Color Don't Care
        (0x08, 0xFF), // Bit Mask
    ];
    for &(i, d) in gc { vga_hw::write_gc(i, d); }

    // --- Attribute Controller ---
    // Сбрасываем flip-flop чтением Input Status 1 (0x3DA)
    let _ = Port::<u8>::new(0x3DA).read();

    let attr: &[(u8, u8)] = &[
        (0x00, 0x00), (0x01, 0x01), (0x02, 0x02), (0x03, 0x03),
        (0x04, 0x04), (0x05, 0x05), (0x06, 0x06), (0x07, 0x07),
        (0x08, 0x08), (0x09, 0x09), (0x0A, 0x0A), (0x0B, 0x0B),
        (0x0C, 0x0C), (0x0D, 0x0D), (0x0E, 0x0E), (0x0F, 0x0F),
        (0x10, 0x41), // Mode Control
        (0x11, 0x00), // Overscan Color
        (0x12, 0x0F), // Color Plane Enable
        (0x13, 0x00), // Horiz. Pixel Panning
        (0x14, 0x00), // Color Select
    ];
    for &(i, d) in attr { vga_hw::write_ac(i, d); }

    // Включаем вывод (PAS bit = 1)
    Port::<u8>::new(0x3C0).write(0x20);
}

// ============================================================
// ВОССТАНОВЛЕНИЕ ТЕКСТОВОГО РЕЖИМА (Mode 3h, 80×25)
// После выхода из Doom — обязательно перед crate::vga_unicode::load_static_glyphs()
// ============================================================

/// # Safety
/// Восстанавливает Mode 3h (80×25 текст): регистры, DAC-палитру, шрифт Plane 2,
/// буфер 0xB8000. Делегирует в `vga_hw::restore_text_mode()` (4 шага Ритуала).
/// После возврата 0xB8000 снова является текстовым буфером.
pub unsafe fn restore_text_mode() {
    crate::vga_hw::restore_text_mode();
}

// ============================================================
// ПАЛИТРА огня — загружаем FIRE_PAL в слоты 0..36 VGA DAC
// ============================================================

/// # Safety
/// VGA DAC: запись тройки R/G/B (0-63) после выбора индекса через 0x3C8.
/// Порты 0x3C8/0x3C9 — I/O с аппаратными побочными эффектами.
/// Вызывать только в Mode 13h, после set_mode_13h().
pub unsafe fn set_fire_palette() {
    let mut pal_idx: Port<u8> = Port::new(0x3C8);
    let mut pal_dat: Port<u8> = Port::new(0x3C9);

    pal_idx.write(0); // начинаем с индекса 0
    for entry in &FIRE_PAL {
        pal_dat.write(entry[0]); // R (0-63)
        pal_dat.write(entry[1]); // G
        pal_dat.write(entry[2]); // B
    }
}

/// Устанавливает один слот VGA DAC-палитры.
///
/// # Safety
/// r/g/b маскируются до 0-63 (VGA DAC принимает 6-битные значения).
/// Порты 0x3C8/0x3C9 — I/O с аппаратными побочными эффектами; вызывать
/// только в графическом режиме.
pub unsafe fn set_palette_color(index: u8, r: u8, g: u8, b: u8) {
    let mut pal_idx: Port<u8> = Port::new(0x3C8);
    let mut pal_dat: Port<u8> = Port::new(0x3C9);
    pal_idx.write(index);
    pal_dat.write(r & 0x3F);
    pal_dat.write(g & 0x3F);
    pal_dat.write(b & 0x3F);
}

// ============================================================
// РИСОВАНИЕ ПИКСЕЛЕЙ — прямой доступ к 0xA0000
// ============================================================

/// Вывод пикселя (без проверок — горячий путь).
///
/// # Safety
/// Вызывающий обязан гарантировать: x < 320, y < 200, Mode 13h активен.
/// VGA_FB (0xA0000) identity-mapped загрузчиком, размер буфера = 64 KB.
#[inline(always)]
pub unsafe fn put_pixel(x: usize, y: usize, color: u8) {
    debug_assert!(x < SCREEN_W, "put_pixel x={} >= SCREEN_W={}", x, SCREEN_W);
    debug_assert!(y < SCREEN_H, "put_pixel y={} >= SCREEN_H={}", y, SCREEN_H);
    // SAFETY: index = y*320+x < 200*320 = 64000 ≤ VGA Mode 13h buffer (64 KB).
    core::ptr::write_volatile(VGA_FB.add(y * SCREEN_W + x), color);
}

/// Заливка всего экрана одним цветом.
///
/// # Safety
/// Mode 13h должен быть активен. VGA_FB (0xA0000) identity-mapped.
pub unsafe fn clear_fb(color: u8) {
    // SAFETY: 320*200 = 64000 байт — ровно размер Mode 13h буфера.
    for i in 0..(SCREEN_W * SCREEN_H) {
        core::ptr::write_volatile(VGA_FB.add(i), color);
    }
}

/// Копирует буфер 320×200 прямо во фреймбуфер.
pub unsafe fn blit(fb: &[u8; SCREEN_W * SCREEN_H]) {
    for (i, &byte) in fb.iter().enumerate() {
        core::ptr::write_volatile(VGA_FB.add(i), byte);
    }
}

// ============================================================
// DOOM FIRE EFFECT — физическая симуляция (f32 тепло, диффузия, конвекция)
// ============================================================

/// Засевает LCG-генератор от RDTSC. Вызывать один раз на кадр/инит.
#[inline(always)]
unsafe fn lcg_reseed() {
    let mut lo: u32 = 0;
    let mut hi: u32 = 0;
    asm!("rdtsc", out("eax") lo, out("edx") hi, options(nomem, nostack));
    LCG_STATE ^= ((hi as u64) << 32) | (lo as u64);
}

/// Быстрый LCG — O(1), без RDTSC.
#[inline(always)]
unsafe fn rand_f32() -> f32 {
    LCG_STATE = LCG_STATE
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    ((LCG_STATE >> 33) as f32) * (1.0 / 2_147_483_648.0)
}

#[inline(always)]
fn clamp(v: f32, min: f32, max: f32) -> f32 {
    if v < min { min } else if v > max { max } else { v }
}

fn is_seed_pixel(mode: FireMode, x: usize, y: usize) -> bool {
    match mode {
        FireMode::Classic  => y == SCREEN_H - 1,
        FireMode::Dual     => y == 0 || y == SCREEN_H - 1,
        FireMode::Inferno  => y == 0 || y == SCREEN_H - 1 || x == 0 || x == SCREEN_W - 1,
        FireMode::FpuNoise => y == SCREEN_H - 1,
    }
}

fn fire_mode_side_width(mode: FireMode) -> usize {
    match mode {
        FireMode::Inferno => FIRE_SIDE_WIDTH,
        _ => 0,
    }
}

// Retained for API compatibility.
fn fire_mode_max_height(_mode: FireMode) -> usize {
    FIRE_MAX_HEIGHT
}

fn seed_edges_heat(mode: FireMode) {
    unsafe {
        let bottom = (SCREEN_H - 1) * SCREEN_W;
        match mode {
            FireMode::Classic | FireMode::FpuNoise => {
                for x in 0..SCREEN_W { HEAT_BUF[bottom + x] = SEED_HEAT; }
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
                for y in 0..SCREEN_H {
                    HEAT_BUF[y * SCREEN_W] = SEED_HEAT;
                    HEAT_BUF[y * SCREEN_W + (SCREEN_W - 1)] = SEED_HEAT;
                }
            }
        }
    }
}

fn seed_edges_next_heat(mode: FireMode) {
    unsafe {
        let bottom = (SCREEN_H - 1) * SCREEN_W;
        match mode {
            FireMode::Classic | FireMode::FpuNoise => {
                for x in 0..SCREEN_W { HEAT_NEXT[bottom + x] = SEED_HEAT; }
            }
            FireMode::Dual => {
                for x in 0..SCREEN_W {
                    HEAT_NEXT[bottom + x] = SEED_HEAT;
                    HEAT_NEXT[x] = SEED_HEAT;
                }
            }
            FireMode::Inferno => {
                for x in 0..SCREEN_W {
                    HEAT_NEXT[bottom + x] = SEED_HEAT;
                    HEAT_NEXT[x] = SEED_HEAT;
                }
                for y in 0..SCREEN_H {
                    HEAT_NEXT[y * SCREEN_W] = SEED_HEAT;
                    HEAT_NEXT[y * SCREEN_W + (SCREEN_W - 1)] = SEED_HEAT;
                }
            }
        }
    }
}

fn fire_simulate_step(mode: FireMode) {
    unsafe {
        // Обновляем LCG-сид раз в шаг — один RDTSC вместо 128 000
        lcg_reseed();

        // 1. Очистка следующего буфера
        for i in 0..(SCREEN_W * SCREEN_H) { HEAT_NEXT[i] = 0.0; }

        // 2. Источники тепла на краях
        seed_edges_next_heat(mode);

        // 3. Диффузия + конвекция + охлаждение
        for y in 0..SCREEN_H {
            for x in 0..SCREEN_W {
                if is_seed_pixel(mode, x, y) { continue; }

                let idx = y * SCREEN_W + x;

                // Диффузия: 4-точечный кросс (center, left, right, below)
                let c = HEAT_BUF[idx];
                let l = if x > 0            { HEAT_BUF[idx - 1] }             else { c };
                let r = if x < SCREEN_W - 1 { HEAT_BUF[idx + 1] }             else { c };
                let b = if y < SCREEN_H - 1 { HEAT_BUF[idx + SCREEN_W] }      else { c };
                let avg = (c + l + r + b) * 0.25;
                let diffused = c * (1.0 - DIFFUSION) + avg * DIFFUSION;

                // Конвекция: случайный сдвиг из строки ниже (целочисленный offset)
                let below_y = (y + 1).min(SCREEN_H - 1);
                // LCG даёт 0..1; масштабируем в -1..1 → округляем до -1,0,+1
                let rf = rand_f32();
                let drift_x = if rf < 0.33 { x.saturating_sub(1) }
                              else if rf > 0.66 { (x + 1).min(SCREEN_W - 1) }
                              else { x };
                let below_val = HEAT_BUF[below_y * SCREEN_W + drift_x];
                let convected = diffused * (1.0 - CONVECTION) + below_val * CONVECTION;

                // Охлаждение + опциональный шум
                let cooled = convected * (1.0 - COOLING);
                let noise = if mode == FireMode::FpuNoise {
                    (rand_f32() - 0.5) * 0.05
                } else { 0.0 };

                HEAT_NEXT[idx] = clamp(cooled + noise, 0.0, MAX_HEAT);
            }
        }

        // 4. Перекладываем буферы
        for i in 0..(SCREEN_W * SCREEN_H) { HEAT_BUF[i] = HEAT_NEXT[i]; }

        // 5. Переутверждаем источники тепла
        seed_edges_heat(mode);
    }
}

pub fn fire_mode() -> FireMode {
    unsafe { FIRE_MODE }
}

pub fn fire_mode_name(mode: FireMode) -> &'static str {
    match mode {
        FireMode::Classic  => "MODE 1 / CLASSIC",
        FireMode::Dual     => "MODE 2 / TOP+BOTTOM",
        FireMode::Inferno  => "MODE 3 / ALL SIDES",
        FireMode::FpuNoise => "MODE 4 / FPU NOISE",
    }
}

fn fire_mode_step_divider(mode: FireMode) -> u8 {
    match mode {
        FireMode::Classic  => 2,
        FireMode::Dual     => 3,
        FireMode::Inferno  => 4,
        FireMode::FpuNoise => 3,
    }
}

fn fire_should_step(mode: FireMode) -> bool {
    unsafe {
        FIRE_STEP_ACC = FIRE_STEP_ACC.wrapping_add(1);
        if FIRE_STEP_ACC >= fire_mode_step_divider(mode) {
            FIRE_STEP_ACC = 0;
            true
        } else {
            false
        }
    }
}

pub fn fire_set_mode(mode: FireMode) {
    unsafe {
        FIRE_MODE = mode;
        FIRE_STEP_ACC = 0;
    }
    fire_init();
}

/// Инициализирует буферы тепла и источники краёв для текущего режима.
pub fn fire_init() {
    unsafe {
        lcg_reseed();
        for i in 0..(SCREEN_W * SCREEN_H) {
            HEAT_BUF[i] = 0.0;
            HEAT_NEXT[i] = 0.0;
        }
    }
    seed_edges_heat(fire_mode());
}

/// Устанавливает тепло в колонке `x` нижней строки (вызывается из `chaos_ignite`).
pub fn fire_ignite_bottom(x: usize, bright: u8) {
    debug_assert!(x < SCREEN_W, "fire_ignite_bottom x={} >= SCREEN_W", x);
    unsafe {
        let idx = (SCREEN_H - 1) * SCREEN_W + x;
        let heat = (bright as f32) / (FIRE_LEVELS as f32 - 1.0);
        HEAT_BUF[idx] = clamp(heat, 0.0, 1.0);
    }
}

/// Один шаг распространения огня в зависимости от активного режима.
pub fn fire_update() {
    let mode = fire_mode();
    if !fire_should_step(mode) { return; }
    fire_simulate_step(mode);
}

/// Рендерит HEAT_BUF во фреймбуфер VGA с дизерингом (Mode 13h должен быть активен).
pub fn fire_render() {
    unsafe {
        for y in 0..SCREEN_H {
            for x in 0..SCREEN_W {
                let heat = HEAT_BUF[y * SCREEN_W + x];
                let dither = (rand_f32() - 0.5) / (FIRE_LEVELS as f32);
                let level_f = heat * (FIRE_LEVELS as f32 - 1.0) + dither;
                let level = clamp(level_f, 0.0, (FIRE_LEVELS - 1) as f32) as u8;
                put_pixel(x, y, level);
            }
        }
    }
}

