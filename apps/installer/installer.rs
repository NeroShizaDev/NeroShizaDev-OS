// ============================================================
// NHS INSTALLER — Мастер установки .nhs пакетов
// ============================================================
//
// Аналог: setup.exe в Windows, но в VGA text mode.
//
// Поток:
//   1. Validate: проверка magic + CRC32
//   2. Info:     показ имени, автора, версии, размера
//   3. Slot:     поиск свободного слота (автоматический)
//   4. Copy:     копирование полного .nhs blob в AppSlot
//   5. Register: запись в AppRegistry
//   6. Done:     "Installation complete!"
//
// TUI-визард рисуется напрямую в VGA 0xB8000 (как launcher).
//
// Вызов из shell:
//   `install` или `install demo` → installer::run(data, label)
// ============================================================

use x86_64::instructions::hlt;
use super::header::{self, NhsHeader, NhsManifest, HEADER_SIZE, MANIFEST_SIZE};
use super::crc32;
use super::slots;
use super::registry;

// ── Результат установки ──────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallResult {
    Ok(usize),          // успех, slot_index
    BadMagic,           // не .nhs файл
    BadChecksum,        // CRC32 не совпадает
    TooLarge,           // > 64KB
    NoFreeSlot,         // все 16 слотов заняты
    BadManifest,        // manifest не читается
    AlreadyInstalled,   // приложение с таким именем уже есть
    UserCanceled,       // Esc в визарде
}

impl InstallResult {
    pub fn description(self) -> &'static str {
        match self {
            Self::Ok(_)            => "Installation complete",
            Self::BadMagic         => "Bad magic: not a .nhs file",
            Self::BadChecksum      => "CRC32 checksum mismatch",
            Self::TooLarge         => "Package > 64KB (slot overflow)",
            Self::NoFreeSlot       => "No free slots (16/16 used)",
            Self::BadManifest      => "Corrupt manifest section",
            Self::AlreadyInstalled => "App already installed",
            Self::UserCanceled     => "Installation canceled by user",
        }
    }
}

// ── VGA helpers (тот же стиль что launcher) ──────────────────

const VGA: *mut u8 = 0xb8000 as *mut u8;

unsafe fn put(row: usize, col: usize, ch: u8, color: u8) {
    if row >= 25 || col >= 80 { return; }
    let off = (row * 80 + col) * 2;
    core::ptr::write_volatile(VGA.add(off), ch);
    core::ptr::write_volatile(VGA.add(off + 1), color);
}

unsafe fn fill_seg(row: usize, col: usize, len: usize, color: u8) {
    for c in 0..len { put(row, col + c, b' ', color); }
}

unsafe fn puts(row: usize, col: usize, s: &[u8], color: u8) {
    for (i, &b) in s.iter().enumerate() { put(row, col + i, b, color); }
}

unsafe fn fill_screen(color: u8) {
    for r in 0..25usize { fill_seg(r, 0, 80, color); }
}

// ── Цвета (зелёная тема — как инсталлятор) ──────────────────

const BG:     u8 = 0x20;  // Black on Green (фон)
const BORDER: u8 = 0x2F;  // White on Green
const TITLE:  u8 = 0x2E;  // Yellow on Green
const FIELD:  u8 = 0x2F;  // White on Green
const VALUE:  u8 = 0x2B;  // Cyan on Green
const ERROR:  u8 = 0x4F;  // White on Red
const OK:     u8 = 0x2A;  // LtGreen on Green
const HINT:   u8 = 0x28;  // DkGray on Green
const BAR_FG: u8 = 0x2A;  // LtGreen on Green (filled)
const BAR_BG: u8 = 0x28;  // DkGray on Green (empty)

// ── CP437 box ────────────────────────────────────────────────

const TL: u8 = 0xC9; const TR: u8 = 0xBB;
const BL: u8 = 0xC8; const BR: u8 = 0xBC;
const HZ: u8 = 0xCD; const VT: u8 = 0xBA;
const ML: u8 = 0xCC; const MR: u8 = 0xB9;

// ── Layout ───────────────────────────────────────────────────

const BOX_COL: usize = 16;
const BOX_W:   usize = 47;
const BOX_ROW: usize = 3;

// ============================================================
// ГЛАВНАЯ ФУНКЦИЯ УСТАНОВКИ
// ============================================================

/// Устанавливает .nhs пакет из сырых байтов.
///
/// Показывает TUI-визард → валидация → копирование → регистрация.
/// Возвращает результат.
///
/// Вызов:
///   let result = installer::install(nhs_bytes);
pub fn install(data: &[u8]) -> InstallResult {
    // ── Step 1: Parse header ─────────────────────────────────
    let hdr = match header::parse_header(data) {
        Some(h) => h,
        None    => {
            show_error(b"Not a valid .nhs file", b"Magic bytes NHS\\x1A not found");
            wait_key();
            return InstallResult::BadMagic;
        }
    };

    // ── Step 2: Parse manifest ───────────────────────────────
    let manifest = match header::parse_manifest(data, hdr.manifest_offset) {
        Some(m) => m,
        None    => {
            show_error(b"Corrupt manifest", b"Cannot read app metadata");
            wait_key();
            return InstallResult::BadManifest;
        }
    };

    // ── Step 3: Check CRC32 ──────────────────────────────────
    // CRC32 считается от всего файла кроме поля checksum (байты 60..64).
    // Для простоты: считаем CRC32 от payload (всё после header).
    if data.len() > HEADER_SIZE {
        let payload = &data[HEADER_SIZE..];
        let computed = crc32::crc32(payload);
        if computed != hdr.checksum {
            show_error(b"CRC32 mismatch", b"File is corrupted");
            wait_key();
            return InstallResult::BadChecksum;
        }
    }

    // ── Step 4: Check size ───────────────────────────────────
    let footprint = hdr.memory_footprint();
    if data.len() > slots::SLOT_SIZE || footprint as usize > slots::SLOT_SIZE {
        show_error(b"Package too large", b"> 64KB slot capacity");
        wait_key();
        return InstallResult::TooLarge;
    }

    // ── Step 5: Check duplicate ──────────────────────────────
    let name_end = manifest.app_name.iter()
        .position(|&b| b == 0).unwrap_or(32);
    if registry::find_by_name(&manifest.app_name[..name_end]).is_some() {
        show_error(b"Already installed", manifest.app_name[..name_end].as_ref());
        wait_key();
        return InstallResult::AlreadyInstalled;
    }

    // ── Step 6: Find free slot ───────────────────────────────
    let slot = match slots::find_free_slot() {
        Some(s) => s,
        None    => {
            show_error(b"No free slots", b"All 16 slots occupied");
            wait_key();
            return InstallResult::NoFreeSlot;
        }
    };

    // ── Step 7: Show confirmation screen ─────────────────────
    let (v_major, v_minor, v_patch) = manifest.version_tuple();
    show_info_screen(&hdr, &manifest, slot, v_major, v_minor, v_patch);

    // Wait for Enter or Esc
    if !wait_confirm() {
        return InstallResult::UserCanceled;
    }

    // ── Step 8: Copy full blob to slot with progress ─────────
    show_progress_screen(&manifest);
    update_progress(33);
    update_progress(66);

    // Install to slot
    if !slots::install_to_slot(slot, data) {
        show_error(b"Slot write failed", b"Internal error");
        wait_key();
        return InstallResult::NoFreeSlot;
    }
    update_progress(90);

    // ── Step 9: Register ─────────────────────────────────────
    registry::register(
        slot,
        &manifest,
        hdr.flags,
        data.len() as u32,
        hdr.lump_count as u16,
        hdr.entry_point,
    );
    update_progress(100);

    // ── Step 10: Success screen ──────────────────────────────
    show_success_screen(&manifest, slot, data.len());
    wait_key();

    InstallResult::Ok(slot)
}

/// Удаляет установленное приложение.
/// Аналог: "Программы и компоненты" → Удалить.
pub fn uninstall(slot: usize) -> bool {
    if !slots::is_occupied(slot) { return false; }

    let name = match registry::get(slot) {
        Some(r) => {
            let mut n = [0u8; 32];
            n.copy_from_slice(&r.name);
            n
        }
        None => return false,
    };

    slots::uninstall_slot(slot);
    registry::unregister(slot);

    // Показываем подтверждение
    let name_end = name.iter().position(|&b| b == 0).unwrap_or(32);
    crate::locale::print_localized_fmt(
        0x0A,
        format_args!("[NHS] Uninstalled: {}", core::str::from_utf8(&name[..name_end]).unwrap_or("?")),
    );

    true
}

// ============================================================
// TUI SCREENS
// ============================================================

fn show_info_screen(
    hdr: &NhsHeader,
    manifest: &NhsManifest,
    slot: usize,
    v_major: u8, v_minor: u8, v_patch: u8,
) {
    unsafe {
        fill_screen(BG);

        // Title bar
        puts(1, 24, b"NHS INSTALLER v1.0", TITLE);

        // Box
        draw_box(BOX_ROW, BOX_COL, BOX_W, 14);

        // Header inside box
        let h = BOX_ROW + 1;
        put(h, BOX_COL, VT, BORDER);
        fill_seg(h, BOX_COL + 1, BOX_W, TITLE);
        puts(h, BOX_COL + 10, b"INSTALL NeroShizaDev PACKAGE", TITLE);
        put(h, BOX_COL + BOX_W + 1, VT, BORDER);

        // Divider
        div_row(BOX_ROW + 2);

        // Fields
        let mut row = BOX_ROW + 3;
        field_row(row, b"Name:",    manifest.name_str().as_bytes());   row += 1;
        field_row(row, b"Author:",  manifest.author_str().as_bytes()); row += 1;

        // Version: manual formatting (no format! in bare-metal for bytes)
        let mut vbuf = [0u8; 12];
        let vlen = fmt_version(&mut vbuf, v_major, v_minor, v_patch);
        field_row(row, b"Version:", &vbuf[..vlen]); row += 1;

        // Size
        let mut sbuf = [0u8; 12];
        let slen = fmt_u32(&mut sbuf, hdr.memory_footprint());
        // Append " bytes"
        let mut full = [0u8; 20];
        full[..slen].copy_from_slice(&sbuf[..slen]);
        full[slen..slen+6].copy_from_slice(b" bytes");
        field_row(row, b"Size:", &full[..slen + 6]); row += 1;

        // Slot
        let mut slbuf = [0u8; 4];
        let sllen = fmt_u32(&mut slbuf, slot as u32);
        let mut slfull = [0u8; 16];
        slfull[0] = b'#';
        slfull[1..1+sllen].copy_from_slice(&slbuf[..sllen]);
        let rest = b" (free)";
        slfull[1+sllen..1+sllen+rest.len()].copy_from_slice(rest);
        field_row(row, b"Slot:", &slfull[..1+sllen+rest.len()]); row += 1;

        // Lumps
        let mut lbuf = [0u8; 4];
        let llen = fmt_u32(&mut lbuf, hdr.lump_count);
        field_row(row, b"Lumps:", &lbuf[..llen]); row += 1;

        // Flags
        field_row(row, b"Flags:", if hdr.flags & super::header::FLAG_HAS_SCRIPT != 0 {
            b"NeroShizaScript"
        } else {
            b"native"
        }); row += 1;

        // Empty + hint
        blank_row(row); row += 1;

        // Buttons hint
        put(row, BOX_COL, VT, BORDER);
        fill_seg(row, BOX_COL + 1, BOX_W, BG);
        puts(row, BOX_COL + 5, b"Enter = Install    Esc = Cancel", HINT);
        put(row, BOX_COL + BOX_W + 1, VT, BORDER);

        // Bottom hint
        puts(22, 20, b"NeroShizaDev-OS  \xB7  Hybrid Package Manager", HINT);
    }
}

fn show_progress_screen(manifest: &NhsManifest) {
    unsafe {
        fill_screen(BG);
        puts(1, 24, b"NHS INSTALLER v1.0", TITLE);
        draw_box(BOX_ROW + 2, BOX_COL, BOX_W, 6);

        let row = BOX_ROW + 3;
        put(row, BOX_COL, VT, BORDER);
        fill_seg(row, BOX_COL + 1, BOX_W, BG);
        puts(row, BOX_COL + 3, b"Installing: ", FIELD);
        puts(row, BOX_COL + 15, manifest.name_str().as_bytes(), VALUE);
        put(row, BOX_COL + BOX_W + 1, VT, BORDER);

        blank_row(row + 1);
        update_progress(0);
        blank_row(row + 3);
    }
}

fn update_progress(pct: u32) {
    let pct = pct.min(100);
    let row = BOX_ROW + 5;
    let bar_start = BOX_COL + 3;
    let bar_len: usize = 36;
    let filled = (pct as usize * bar_len) / 100;

    unsafe {
        put(row, BOX_COL, VT, BORDER);
        fill_seg(row, BOX_COL + 1, BOX_W, BG);

        // [################............] 65%
        put(row, bar_start, b'[', FIELD);
        for i in 0..bar_len {
            let ch = if i < filled { 0xDB } else { 0xB0 };  // █ vs ░
            let color = if i < filled { BAR_FG } else { BAR_BG };
            put(row, bar_start + 1 + i, ch, color);
        }
        put(row, bar_start + bar_len + 1, b']', FIELD);

        // Percentage
        let mut pbuf = [0u8; 4];
        let plen = fmt_u32(&mut pbuf, pct);
        puts(row, bar_start + bar_len + 3, &pbuf[..plen], VALUE);
        put(row, bar_start + bar_len + 3 + plen, b'%', VALUE);

        put(row, BOX_COL + BOX_W + 1, VT, BORDER);
    }

    // Fake delay for visual feedback
    for _ in 0..5_000_000u64 { core::hint::spin_loop(); }
}

fn show_success_screen(manifest: &NhsManifest, slot: usize, size: usize) {
    unsafe {
        fill_screen(BG);
        puts(1, 24, b"NHS INSTALLER v1.0", TITLE);
        draw_box(BOX_ROW + 2, BOX_COL, BOX_W, 8);

        let row = BOX_ROW + 3;
        put(row, BOX_COL, VT, BORDER);
        fill_seg(row, BOX_COL + 1, BOX_W, OK);
        puts(row, BOX_COL + 8, b"\x10 INSTALLATION COMPLETE \x11", OK);
        put(row, BOX_COL + BOX_W + 1, VT, BORDER);

        div_row(row + 1);

        field_row(row + 2, b"App:",  manifest.name_str().as_bytes());

        let mut sbuf = [0u8; 4];
        let slen = fmt_u32(&mut sbuf, slot as u32);
        field_row(row + 3, b"Slot:", &sbuf[..slen]);

        let mut zbuf = [0u8; 12];
        let zlen = fmt_u32(&mut zbuf, size as u32);
        let mut zfull = [0u8; 20];
        zfull[..zlen].copy_from_slice(&zbuf[..zlen]);
        zfull[zlen..zlen+6].copy_from_slice(b" bytes");
        field_row(row + 4, b"Size:", &zfull[..zlen + 6]);

        blank_row(row + 5);

        put(row + 6, BOX_COL, VT, BORDER);
        fill_seg(row + 6, BOX_COL + 1, BOX_W, BG);
        puts(row + 6, BOX_COL + 5, b"Stored in NHS registry and slot", HINT);
        put(row + 6, BOX_COL + BOX_W + 1, VT, BORDER);

        puts(22, 25, b"Press any key to continue...", HINT);
    }
}

fn show_error(title: &[u8], detail: &[u8]) {
    unsafe {
        fill_screen(BG);
        puts(1, 24, b"NHS INSTALLER v1.0", TITLE);
        draw_box(BOX_ROW + 4, BOX_COL, BOX_W, 6);

        let row = BOX_ROW + 5;
        put(row, BOX_COL, VT, BORDER);
        fill_seg(row, BOX_COL + 1, BOX_W, ERROR);
        puts(row, BOX_COL + 3, b"\x10 ERROR", ERROR);
        put(row, BOX_COL + BOX_W + 1, VT, BORDER);

        div_row(row + 1);

        put(row + 2, BOX_COL, VT, BORDER);
        fill_seg(row + 2, BOX_COL + 1, BOX_W, BG);
        puts(row + 2, BOX_COL + 3, title, FIELD);
        put(row + 2, BOX_COL + BOX_W + 1, VT, BORDER);

        put(row + 3, BOX_COL, VT, BORDER);
        fill_seg(row + 3, BOX_COL + 1, BOX_W, BG);
        puts(row + 3, BOX_COL + 3, detail, HINT);
        put(row + 3, BOX_COL + BOX_W + 1, VT, BORDER);

        puts(22, 25, b"Press any key to return...", HINT);
    }
}

// ── Box drawing helpers ──────────────────────────────────────

unsafe fn draw_box(top: usize, col: usize, w: usize, h: usize) {
    // Top border
    put(top, col, TL, BORDER);
    for c in 1..=w { put(top, col + c, HZ, BORDER); }
    put(top, col + w + 1, TR, BORDER);

    // Sides
    for r in 1..h {
        put(top + r, col, VT, BORDER);
        put(top + r, col + w + 1, VT, BORDER);
    }

    // Bottom border
    put(top + h, col, BL, BORDER);
    for c in 1..=w { put(top + h, col + c, HZ, BORDER); }
    put(top + h, col + w + 1, BR, BORDER);
}

unsafe fn div_row(row: usize) {
    put(row, BOX_COL, ML, BORDER);
    for c in 1..=BOX_W { put(row, BOX_COL + c, HZ, BORDER); }
    put(row, BOX_COL + BOX_W + 1, MR, BORDER);
}

unsafe fn blank_row(row: usize) {
    put(row, BOX_COL, VT, BORDER);
    fill_seg(row, BOX_COL + 1, BOX_W, BG);
    put(row, BOX_COL + BOX_W + 1, VT, BORDER);
}

unsafe fn field_row(row: usize, label: &[u8], value: &[u8]) {
    put(row, BOX_COL, VT, BORDER);
    fill_seg(row, BOX_COL + 1, BOX_W, BG);
    puts(row, BOX_COL + 3, label, FIELD);
    let val_col = BOX_COL + 3 + label.len() + 1;
    let max_val = (BOX_COL + BOX_W).saturating_sub(val_col);
    let vlen = value.len().min(max_val);
    puts(row, val_col, &value[..vlen], VALUE);
    put(row, BOX_COL + BOX_W + 1, VT, BORDER);
}

// ── Input ────────────────────────────────────────────────────

fn wait_key() {
    unsafe {
        loop {
            if crate::ps2::has_scancode() {
                let sc = crate::ps2::read_scancode();
                if sc & 0x80 == 0 { break; }
            }
            hlt();
        }
    }
}

/// Ждёт Enter (true) или Esc (false).
fn wait_confirm() -> bool {
    unsafe {
        loop {
            if crate::ps2::has_scancode() {
                let sc = crate::ps2::read_scancode();
                if sc & 0x80 != 0 { continue; }
                match sc {
                    0x1C => return true,   // Enter
                    0x01 => return false,  // Esc
                    _ => {}
                }
            }
            hlt();
        }
    }
}

// ── Formatting (no alloc) ────────────────────────────────────

fn fmt_u32(buf: &mut [u8], val: u32) -> usize {
    if val == 0 { buf[0] = b'0'; return 1; }
    let mut tmp = [0u8; 10];
    let mut n = 0usize;
    let mut v = val;
    while v > 0 { tmp[n] = b'0' + (v % 10) as u8; v /= 10; n += 1; }
    for i in 0..n { buf[i] = tmp[n - 1 - i]; }
    n
}

fn fmt_version(buf: &mut [u8], major: u8, minor: u8, patch: u8) -> usize {
    let mut pos = 0;
    pos += fmt_u32(&mut buf[pos..], major as u32);
    buf[pos] = b'.'; pos += 1;
    pos += fmt_u32(&mut buf[pos..], minor as u32);
    buf[pos] = b'.'; pos += 1;
    pos += fmt_u32(&mut buf[pos..], patch as u32);
    pos
}
