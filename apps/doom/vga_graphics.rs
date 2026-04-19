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

use x86_64::instructions::port::Port;

use crate::vga_hw;

pub const SCREEN_W: usize = 320;
pub const SCREEN_H: usize = 200;

/// Физический адрес VGA graphics framebuffer (Mode 13h)
const VGA_FB: *mut u8 = 0xA0000 as *mut u8;

// ============================================================
// Режим DOOM Fire — 37 уровней яркости (чёрный → красный → белый)
// Палитра из оригинального Doom (делённые на 4 для VGA DAC 0-63)
// ============================================================
pub const FIRE_LEVELS: usize = 37;

/// RGB-компоненты палитры огня (0..63 для VGA DAC)
static FIRE_PAL: [[u8; 3]; FIRE_LEVELS] = [
    [0x01, 0x01, 0x01], // 0  — почти чёрный
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
// Буфер огня — 320×200 байт (64 KB, в BSS)
// ============================================================
static mut FIRE_BUF: [u8; SCREEN_W * SCREEN_H] = [0; SCREEN_W * SCREEN_H];

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
// DOOM FIRE EFFECT — классический алгоритм Natascha (Fabien Sanglard)
// ============================================================

/// Инициализирует нижнюю строку максимальной яркостью — источник огня.
pub fn fire_init() {
    // SAFETY: FIRE_BUF — 64 KB static в BSS. Вызывается из основного цикла doom,
    // не конкурирует с обработчиками прерываний (doom работает с отключёнными IRQ).
    unsafe {
        let base = (SCREEN_H - 1) * SCREEN_W;
        for x in 0..SCREEN_W {
            FIRE_BUF[base + x] = (FIRE_LEVELS - 1) as u8;
        }
    }
}

/// Устанавливает яркость `bright` в колонке `x` нижней строки FIRE_BUF.
/// Вызывается из `chaos_ignite` при удержании клавиши FIRE.
///
/// # Safety
/// Отдельная функция нужна потому что FIRE_BUF — `static mut` и недоступен
/// снаружи модуля. `x < SCREEN_W` проверяется вызывающим (debug_assert).
pub fn fire_ignite_bottom(x: usize, bright: u8) {
    debug_assert!(x < SCREEN_W, "fire_ignite_bottom x={} >= SCREEN_W", x);
    // SAFETY: FIRE_BUF — 64 KB static в BSS; idx = (SCREEN_H-1)*SCREEN_W + x,
    // при x < SCREEN_W это всегда < SCREEN_W*SCREEN_H = 64000.
    // Вызывается из главного цикла Doom — нет конкурентного доступа.
    unsafe {
        let idx = (SCREEN_H - 1) * SCREEN_W + x;
        FIRE_BUF[idx] = bright;
    }
}

/// Один шаг распространения огня снизу вверх (алгоритм Sanglard).
pub fn fire_update() {
    // SAFETY: FIRE_BUF — 64 KB статик. Индексы: src = y*320+x (y∈1..200, x∈0..320),
    // dst = (y-1)*320 + dst_x — всегда в пределах [0, 64000).
    unsafe {
        for y in 1..SCREEN_H {
            for x in 0..SCREEN_W {
                let src   = y * SCREEN_W + x;
                let pixel = FIRE_BUF[src];

                if pixel == 0 {
                    // Холодный пиксель — гасит пиксель над собой
                    FIRE_BUF[src - SCREEN_W] = 0;
                } else {
                    // Случайное горизонтальное смещение [0..3] и затухание на 0 или 1
                    let rand  = crate::rng::random_range(4) as usize;
                    let dst_x = (x + SCREEN_W + 1 - (rand & 1)) % SCREEN_W;
                    let decay = (rand & 1) as u8;
                    FIRE_BUF[(y - 1) * SCREEN_W + dst_x] = pixel.saturating_sub(decay);
                }
            }
        }
    }
}

/// Рендерит FIRE_BUF во фреймбуфер VGA (Mode 13h должен быть активен).
pub fn fire_render() {
    // SAFETY: put_pixel вызывается с x<320 y<200 — пределы из цикла.
    // FIRE_BUF[y*320+x] всегда в пределах [0, 64000).
    unsafe {
        for y in 0..SCREEN_H {
            for x in 0..SCREEN_W {
                let color = FIRE_BUF[y * SCREEN_W + x];
                put_pixel(x, y, color);
            }
        }
    }
}

