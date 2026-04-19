// ============================================================
// VGA HW — общие примитивы записи в VGA-регистры
// ============================================================
// Устраняет дублирование между font.rs, vga_unicode.rs и doom/vga_graphics.rs.
// Все функции unsafe: прямой доступ к аппаратным I/O-портам VGA.
// ============================================================

use x86_64::instructions::port::Port;

// ============================================================
// Вспомогательные записи в пары index/data портов
// ============================================================

/// Записывает (index, data) в Sequencer (0x3C4/0x3C5).
///
/// # Safety
/// Порты 0x3C4/0x3C5 — VGA Sequencer. Переключает Map Mask / Memory Mode;
/// некорректный порядок записей оставит VGA в неконсистентном состоянии.
/// Bare-metal: единственный потребитель, нет конкурентных драйверов.
#[inline(always)]
pub unsafe fn write_seq(index: u8, data: u8) {
    Port::<u8>::new(0x3C4).write(index);
    Port::<u8>::new(0x3C5).write(data);
}

/// Записывает (index, data) в Graphics Controller (0x3CE/0x3CF).
///
/// # Safety
/// Порты 0x3CE/0x3CF — VGA Graphics Controller. Управляет Read Map / Write Mode.
/// Bare-metal: нет конкурентных драйверов.
#[inline(always)]
pub unsafe fn write_gc(index: u8, data: u8) {
    Port::<u8>::new(0x3CE).write(index);
    Port::<u8>::new(0x3CF).write(data);
}

/// Записывает (index, data) в Attribute Controller (0x3C0).
/// Flip-flop должен быть сброшен чтением 0x3DA перед первым вызовом.
///
/// # Safety
/// Порт 0x3C0 — Attribute Controller, чередует index/data через внутренний flip-flop.
/// Вызывающий обязан сбросить flip-flop чтением 0x3DA перед серией вызовов.
#[inline(always)]
pub unsafe fn write_ac(index: u8, data: u8) {
    let mut ac: Port<u8> = Port::new(0x3C0);
    ac.write(index);
    ac.write(data);
}

/// Записывает (index, data) в произвольную пару index/data портов.
/// Используется для CRTC (0x3D4/0x3D5) и других пар.
///
/// # Safety
/// idx_addr/dat_addr должны быть валидными VGA-регистрами (0x3D4/0x3D5 и т.п.).
/// Прямой доступ к портам с аппаратными побочными эффектами (RFC #2873).
#[inline(always)]
pub unsafe fn write_reg(idx_addr: u16, dat_addr: u16, index: u8, data: u8) {
    Port::<u8>::new(idx_addr).write(index);
    Port::<u8>::new(dat_addr).write(data);
}

// ============================================================
// Переключение VGA в font plane 2 и обратно
// ============================================================

/// Переключает VGA в режим доступа к plane 2 (шрифтовая плоскость).
///
/// # Safety
/// После вызова 0xA0000 отображает plane 2 (шрифты), а не видеопамять.
/// Обязательно вызывать exit_font_mode() перед любой записью в 0xB8000.
/// Вызывать под without_interrupts — иначе вывод текста во время font mode
/// повредит глифы в plane 2.
pub unsafe fn enter_font_mode() {
    write_seq(0x02, 0x04); // Map Mask: plane 2 only
    write_seq(0x04, 0x06); // Memory Mode: sequential, no odd/even
    write_gc(0x04, 0x02);  // Read Map Select: plane 2
    write_gc(0x05, 0x00);  // Graphics Mode: write mode 0, read mode 0
    write_gc(0x06, 0x00);  // Misc: A0000-BFFFF, no chain odd/even
}

/// Восстанавливает нормальный текстовый режим VGA после enter_font_mode().
///
/// # Safety
/// Восстанавливает Map Mask=0x03 (плоскости 0+1), GC Mode=0x10 (текст).
/// После выхода 0xB8000 снова является текстовым буфером.
pub unsafe fn exit_font_mode() {
    write_seq(0x02, 0x03); // Map Mask: planes 0+1
    write_seq(0x04, 0x02); // Memory Mode: odd/even
    write_gc(0x04, 0x00);  // Read Map Select: plane 0
    write_gc(0x05, 0x10);  // Graphics Mode: odd/even (text)
    write_gc(0x06, 0x0E);  // Misc: B8000-BFFFF, text mode
}

// ============================================================
// VGA MODE DETECTION — probe_vga() + попытка расширенного режима
// ============================================================

/// Текстовый режим VGA после детекта и попытки переключения.
#[derive(Debug, Clone, Copy)]
pub enum VgaTextMode {
    /// VGA не отвечает (0xFF на 0x3DA) — нет дисплея.
    None,
    /// Стандартный 80×25 — CRTC не принял расширенные параметры.
    Mode80x25,
    /// Расширенный 90×30 — CRTC успешно перепрограммирован.
    Mode90x30,
}

/// Размеры VGA буфера в зависимости от текущего режима
pub const VGA_COLUMNS: usize = 80;  // Стандартный размер 80 колонок
pub const VGA_ROWS: usize = 25;     // Стандартный размер 25 строк

/// Глобальная переменная для хранения текущего режима VGA
static mut CURRENT_VGA_MODE: VgaTextMode = VgaTextMode::Mode80x25;

/// Пробует перепрограммировать CRTC для режима 90×30.
/// Алгоритм: разблокировать CRTC → записать → перечитать → сравнить.
/// Если значение не сохранилось — откатываемся, возвращаем Mode80x25.
///
/// SAFETY: Только VGA CRTC регистры (0x3D4/0x3D5). Чтение 0x3DA безопасно.
/// Запись CRTC без предварительного разблокирования (reg 0x11 бит 7=0)
/// будет проигнорирована защищённым контроллером.
unsafe fn try_90x30() -> VgaTextMode {
    // Шаг 1: разблокируем защиту CRTC (reg 0x11, сбрасываем бит 7)
    Port::<u8>::new(0x3D4).write(0x11u8);
    let old_11 = Port::<u8>::new(0x3D5).read();
    Port::<u8>::new(0x3D4).write(0x11u8);
    Port::<u8>::new(0x3D5).write(old_11 & 0x7F);

    // Шаг 2: пишем 89 в регистр 0x01 (Horizontal Display End = 90 col - 1)
    Port::<u8>::new(0x3D4).write(0x01u8);
    Port::<u8>::new(0x3D5).write(89u8);

    // Шаг 3: перечитываем — принял ли CRTC значение?
    Port::<u8>::new(0x3D4).write(0x01u8);
    let readback = Port::<u8>::new(0x3D5).read();

    if readback == 89 {
        // CRTC принял значение, но это НЕ гарантирует, что эмулятор
        // реально отображает 90 колонок (QEMU принимает запись в регистр,
        // но рендерит только 80). Нужна дополнительная верификация:
        // читаем текущий Horizontal Total (reg 0x00) — для 90 колонок
        // он должен быть >= 99 (обычно ~107). Если он остался в диапазоне
        // 80-колоночного режима (~95), CRTC не перестроился реально.
        Port::<u8>::new(0x3D4).write(0x00u8);
        let h_total = Port::<u8>::new(0x3D5).read();

        if h_total >= 99 {
            // Меняем Vertical Display End для 30 строк
            Port::<u8>::new(0x3D4).write(0x12u8);
            Port::<u8>::new(0x3D5).write(0xDFu8); // 480 строк пикселей
            VgaTextMode::Mode90x30
        } else {
            // Эмулятор: readback прошёл, но Horizontal Total не обновился.
            // Откатываем на 80 колонок.
            Port::<u8>::new(0x3D4).write(0x01u8);
            Port::<u8>::new(0x3D5).write(79u8);
            VgaTextMode::Mode80x25
        }
    } else {
        // CRTC не принял значение — восстанавливаем 80 колонок.
        Port::<u8>::new(0x3D4).write(0x01u8);
        Port::<u8>::new(0x3D5).write(79u8);
        VgaTextMode::Mode80x25
    }
}

/// Детектирует VGA и пробует переключить в расширенный текстовый режим.
///
/// Цепочка:
/// 1. validator::probe_vga() (0x3DA) — есть ли VGA вообще?
/// 2. try_90x30() — CRTC принимает 90×30?
/// 3. Возвращает достигнутый режим.
pub fn detect_and_switch() -> VgaTextMode {
    if !crate::validator::probe_vga() {
        return VgaTextMode::None;
    }
    // 90×30 отключён: QEMU принимает CRTC-записи (readback проходит),
    // но реально не меняет геометрию — экран остаётся 80×25,
    // а код пишет 90 символов в строку → текст нечитаем.
    // TODO: реализовать надёжную верификацию (например, пиксельный тест)
    //       перед повторным включением расширенного режима.
    let mode = VgaTextMode::Mode80x25;
    unsafe { CURRENT_VGA_MODE = mode; }
    mode
}

/// Возвращает количество колонок в текущем VGA режиме
pub fn get_columns() -> usize {
    unsafe {
        match CURRENT_VGA_MODE {
            VgaTextMode::Mode90x30 => 90,
            VgaTextMode::Mode80x25 => 80,
            VgaTextMode::None => 80, // fallback
        }
    }
}

/// Возвращает количество строк в текущем VGA режиме
pub fn get_rows() -> usize {
    unsafe {
        match CURRENT_VGA_MODE {
            VgaTextMode::Mode90x30 => 30,
            VgaTextMode::Mode80x25 => 25,
            VgaTextMode::None => 25, // fallback
        }
    }
}

// ============================================================
// VGA MODE 3 RESET — программный сброс в стандартный текстовый режим
// ============================================================
// Bootloader 0.11 переключает дисплей в VBE/framebuffer при загрузке.
// INT 10h (AX=0x0003) недоступен в long mode, поэтому перепрограммируем
// все VGA-регистры вручную. Значения — стандарт VGA Mode 3, 80×25, 16 цветов.
// ============================================================

/// Принудительно переключает VGA в стандартный текстовый режим 3 (80×25).
///
/// # Safety
/// Перепрограммирует ВСЕ VGA-регистры. После вызова 0xB8000 снова является
/// текстовым буфером, а дисплей отображает текстовый режим.
/// Вызывать ТОЛЬКО после того, как VGA-память замаплена в page tables.
pub unsafe fn force_text_mode_3() {
    // Стандартные значения регистров для VGA Mode 3 (80×25 text, 720×400, 16-color)
    // Источник: VGA hardware reference / Bochs VGA BIOS / SeaBIOS

    // --- ПЕРВЫМ ДЕЛОМ: отключаем Bochs VBE (BGA/VBE extensions) ---
    // Bootloader переключает дисплей через Bochs VBE (порты 0x01CE/0x01CF).
    // Пока VBE активен, стандартные VGA-регистры ИГНОРИРУЮТСЯ эмулятором.
    // VBE_DISPI_INDEX_ENABLE = 0x04, значение 0x00 = отключить VBE.
    Port::<u16>::new(0x01CE).write(0x0004); // index = VBE_DISPI_INDEX_ENABLE
    Port::<u16>::new(0x01CF).write(0x0000); // value = disabled

    // --- Miscellaneous Output Register (write: 0x3C2, read: 0x3CC) ---
    Port::<u8>::new(0x3C2).write(0x67);

    // --- Sequencer (0x3C4/0x3C5) ---
    static SEQ_REGS: [u8; 5] = [0x03, 0x00, 0x03, 0x00, 0x02];
    for (i, &val) in SEQ_REGS.iter().enumerate() {
        write_seq(i as u8, val);
    }

    // --- Разблокируем CRTC (reg 0x11, бит 7 = 0) ---
    Port::<u8>::new(0x3D4).write(0x11u8);
    let cr11 = Port::<u8>::new(0x3D5).read();
    Port::<u8>::new(0x3D4).write(0x11u8);
    Port::<u8>::new(0x3D5).write(cr11 & 0x7F);

    // --- CRTC (0x3D4/0x3D5) ---
    static CRTC_REGS: [u8; 25] = [
        0x5F, // 0x00: Horizontal Total
        0x4F, // 0x01: Horizontal Display End
        0x50, // 0x02: Start Horizontal Blanking
        0x82, // 0x03: End Horizontal Blanking
        0x55, // 0x04: Start Horizontal Retrace
        0x81, // 0x05: End Horizontal Retrace
        0xBF, // 0x06: Vertical Total
        0x1F, // 0x07: Overflow
        0x00, // 0x08: Preset Row Scan
        0x4F, // 0x09: Maximum Scan Line
        0x0D, // 0x0A: Cursor Start
        0x0E, // 0x0B: Cursor End
        0x00, // 0x0C: Start Address High
        0x00, // 0x0D: Start Address Low
        0x00, // 0x0E: Cursor Location High
        0x00, // 0x0F: Cursor Location Low
        0x9C, // 0x10: Vertical Retrace Start
        0x0E, // 0x11: Vertical Retrace End (+ CRTC protect)
        0x8F, // 0x12: Vertical Display End
        0x28, // 0x13: Offset (logical line width / 2)
        0x1F, // 0x14: Underline Location
        0x96, // 0x15: Start Vertical Blanking
        0xB9, // 0x16: End Vertical Blanking
        0xA3, // 0x17: CRTC Mode Control
        0xFF, // 0x18: Line Compare
    ];
    for (i, &val) in CRTC_REGS.iter().enumerate() {
        write_reg(0x3D4, 0x3D5, i as u8, val);
    }

    // --- Graphics Controller (0x3CE/0x3CF) ---
    static GC_REGS: [u8; 9] = [
        0x00, // 0x00: Set/Reset
        0x00, // 0x01: Enable Set/Reset
        0x00, // 0x02: Color Compare
        0x00, // 0x03: Data Rotate
        0x00, // 0x04: Read Map Select
        0x10, // 0x05: Graphics Mode (odd/even = text)
        0x0E, // 0x06: Miscellaneous (B8000, text mode)
        0x00, // 0x07: Color Don't Care
        0xFF, // 0x08: Bit Mask
    ];
    for (i, &val) in GC_REGS.iter().enumerate() {
        write_gc(i as u8, val);
    }

    // --- Attribute Controller (0x3C0) ---
    // Сначала сбрасываем flip-flop чтением 0x3DA
    let _ = Port::<u8>::new(0x3DA).read();
    static AC_REGS: [u8; 21] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x14, 0x07,
        0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F,
        0x0C, // 0x10: Attribute Mode Control
        0x00, // 0x11: Overscan Color
        0x0F, // 0x12: Color Plane Enable
        0x08, // 0x13: Horizontal Pixel Panning
        0x00, // 0x14: Color Select
    ];
    for (i, &val) in AC_REGS.iter().enumerate() {
        write_ac(i as u8, val);
    }
    // Включаем дисплей: бит 5 = PAS (Palette Address Source)
    Port::<u8>::new(0x3C0).write(0x20);

    // --- Загружаем стандартную VGA text-mode палитру (16 цветов EGA/VGA) ---
    set_default_text_palette();

    // --- Загружаем стандартный VGA 8×16 ROM font в plane 2 ---
    // VBE перезаписал plane 2 данными framebuffer'а, поэтому все глифы — мусор.
    // Загружаем базовый ASCII шрифт (0-127) вручную.
    load_bios_font_8x16();

    // --- Очищаем VGA text buffer (0xB8000) ---
    // После VBE в буфере мусор от framebuffer'а. Заполняем пробелами
    // с атрибутом 0x07 (light gray on black) — стандартный BIOS default.
    let buf = 0xB8000 as *mut u16;
    for i in 0..(80 * 25) {
        core::ptr::write_volatile(buf.add(i), 0x0720); // ' ' + attribute 0x07
    }

    // Обновляем глобальный режим
    CURRENT_VGA_MODE = VgaTextMode::Mode80x25;
}

/// Устанавливает стандартную 16-цветную палитру VGA для текстового режима.
pub unsafe fn set_default_text_palette() {
    // Стандартная EGA/VGA палитра: 16 цветов × RGB (6-бит VGA DAC)
    static PALETTE: [[u8; 3]; 16] = [
        [0x00, 0x00, 0x00], // 0: Black
        [0x00, 0x00, 0x2A], // 1: Blue
        [0x00, 0x2A, 0x00], // 2: Green
        [0x00, 0x2A, 0x2A], // 3: Cyan
        [0x2A, 0x00, 0x00], // 4: Red
        [0x2A, 0x00, 0x2A], // 5: Magenta
        [0x2A, 0x15, 0x00], // 6: Brown
        [0x2A, 0x2A, 0x2A], // 7: Light Gray
        [0x15, 0x15, 0x15], // 8: Dark Gray
        [0x15, 0x15, 0x3F], // 9: Light Blue
        [0x15, 0x3F, 0x15], // 10: Light Green
        [0x15, 0x3F, 0x3F], // 11: Light Cyan
        [0x3F, 0x15, 0x15], // 12: Light Red
        [0x3F, 0x15, 0x3F], // 13: Pink
        [0x3F, 0x3F, 0x15], // 14: Yellow
        [0x3F, 0x3F, 0x3F], // 15: White
    ];

    // DAC запись: index → 0x3C8, затем 3 байта R,G,B → 0x3C9
    Port::<u8>::new(0x3C8).write(0);
    for rgb in &PALETTE {
        Port::<u8>::new(0x3C9).write(rgb[0]);
        Port::<u8>::new(0x3C9).write(rgb[1]);
        Port::<u8>::new(0x3C9).write(rgb[2]);
    }
}

/// Загружает стандартный VGA 8×16 BIOS ROM шрифт (ASCII 0-127) в plane 2.
///
/// После VBE (framebuffer mode) plane 2 содержит мусор от линейного framebuffer'а.
/// INT 10h (AX=0x1114) недоступен в long mode — грузим шрифт вручную.
///
/// Формат VGA font plane: каждый символ занимает 32 байта (даже для 8×16):
/// 16 байт битмап + 16 байт padding. Символ N начинается с offset = N * 32.
///
/// # Safety
/// Переключает VGA в font mode (plane 2), пишет в 0xA0000, восстанавливает text mode.
pub unsafe fn load_bios_font_8x16() {
    use crate::fonts::bios_font::VGA_FONT_8X16;

    enter_font_mode();

    let font_base = 0xA0000 as *mut u8;
    for ch in 0..128usize {
        let glyph = &VGA_FONT_8X16[ch];
        let dst = font_base.add(ch * 32); // 32 bytes per character slot
        for row in 0..16usize {
            core::ptr::write_volatile(dst.add(row), glyph[row]);
        }
        // Оставшиеся 16 байт (padding) обнуляем
        for row in 16..32usize {
            core::ptr::write_volatile(dst.add(row), 0);
        }
    }

    exit_font_mode();
}

// ============================================================
// RESTORE TEXT MODE — полный возврат из Mode 13h / любого графического режима
// ============================================================
// Шаг 1: Перепрограммируем все VGA-регистры (Misc/SEQ/CRTC/GC/AC) для Mode 3.
// Шаг 2: Восстанавливаем стандартную 16-цветную EGA/VGA палитру DAC (0x3C8/0x3C9).
// Шаг 3: Загружаем ASCII шрифт (0-127) в Plane 2 по адресу 0xA0000.
// Шаг 4: Очищаем текстовый буфер 0xB8000 (80×25 = 2000 ячеек, attr 0x07).
//
// После возврата из Doom вызывать:
//   unsafe { crate::vga_hw::restore_text_mode(); }
//   unsafe { crate::vga_unicode::load_static_glyphs(); } // кириллица 128-191
// ============================================================

/// Полный возврат VGA в текстовый режим Mode 3 (80×25) из Mode 13h или любого графического режима.
///
/// Выполняет 4 шага Ритуала Восстановления:
/// 1. Перепрограммирует все VGA-регистры (Misc / SEQ / CRTC / GC / AC)
/// 2. Восстанавливает стандартную EGA/VGA палитру DAC (16 цветов)
/// 3. Загружает ASCII 8×16 шрифт в Plane 2 (0xA0000)
/// 4. Очищает текстовый буфер 0xB8000 пробелами (attr 0x07)
///
/// # Safety
/// Перепрограммирует все VGA-регистры и пишет в VGA-память.
/// Вызывать только когда Mode 13h (или другой граф. режим) активен.
/// После вызова 0xB8000 снова является текстовым буфером.
pub unsafe fn restore_text_mode() {
    // Шаг 1: Регистры — reuse force_text_mode_3() который уже делает шаги 1+2+3+4
    // НО force_text_mode_3() пишет VBE disable (0x01CE/0x01CF) — это безвредно
    // после Mode 13h, т.к. VBE уже был отключён при входе в Mode 13h.
    force_text_mode_3();
    // force_text_mode_3() уже делает:
    //   - Шаг 1: все VGA-регистры (Misc/SEQ/CRTC/GC/AC)
    //   - Шаг 2: set_default_text_palette() — 16 EGA цветов
    //   - Шаг 3: load_bios_font_8x16() — ASCII шрифт в Plane 2
    //   - Шаг 4: очистку 0xB8000 (0x0720 = пробел + attr gray-on-black)
    // Ничего дополнительного не нужно.
}
