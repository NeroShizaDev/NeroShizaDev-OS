/// APPS Launcher — TUI home screen activity.
///
/// Lifecycle (called by ActivityManager):
///   on_start()       — первый запуск, рисуем экран
///   update(depth)    — блокирующий опрос клавиш → ActivityIntent
///   on_pause()       — нас перекрыли сверху (ничего не делаем)
///   on_resume()      — вернулись к нам, перерисовываем экран

use x86_64::instructions::hlt;
use crate::apps::activity::{AppKind, ActivityIntent};

// ── Direct VGA helpers (no WRITER lock — launcher owns the screen) ───────────
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

// ── CP437 box-drawing ────────────────────────────────────────────────────────
const TL:  u8 = 0xC9; // ╔
const TR:  u8 = 0xBB; // ╗
const BL:  u8 = 0xC8; // ╚
const BR:  u8 = 0xBC; // ╝
const HZ:  u8 = 0xCD; // ═
const VT:  u8 = 0xBA; // ║
const ML:  u8 = 0xCC; // ╠
const MR:  u8 = 0xB9; // ╣
const SEP: u8 = 0xB3; // │
const ARW: u8 = 0x10; // ►

// ── Colors (Blue bg palette) ──────────────────────────────────────────────────
const BG:     u8 = 0x10; // Black  on Blue
const BORDER: u8 = 0x1B; // Cyan   on Blue
const TITLE:  u8 = 0x1E; // Yellow on Blue
const NORMAL: u8 = 0x17; // LtGray on Blue
const HILIT:  u8 = 0x70; // Black  on White (selected bar)
const HINT:   u8 = 0x18; // DkGray on Blue
const TAG:    u8 = 0x1A; // LtGreen on Blue

// ── Menu entries ─────────────────────────────────────────────────────────────
// app: None = "back to shell" (Pop), Some(X) = Push(X)

struct Entry {
    app:       Option<AppKind>,
    name:      &'static [u8],
    desc:      &'static [u8],
    tag:       &'static [u8],
    is_header: bool,   // true — секция-разделитель, не выбирается
}

// Группы как Windows Start — 4 секции + разделитель + выход
const ENTRIES: &[Entry] = &[
    // ─── GAMES ───────────────────────────────────────────────────────────────
    Entry { is_header: true,  app: None,                   name: b" GAMES",   desc: b"", tag: b"" },
    Entry { is_header: false, app: Some(AppKind::Games),   name: b"Games",   desc: b"Dodge, Cards, Clicker & more",    tag: b"[5in1]  " },
    Entry { is_header: false, app: Some(AppKind::Doom),    name: b"Doom",    desc: b"Doom Fire / WAD / Mode 13h demo", tag: b"[Demo]  " },
    // ─── TOOLS ───────────────────────────────────────────────────────────────
    Entry { is_header: true,  app: None,                   name: b" TOOLS",   desc: b"", tag: b"" },
    Entry { is_header: false, app: Some(AppKind::Jackal),  name: b"Jackal",  desc: b"Jackal analyzer & archiver",      tag: b"[Tool]  " },
    // ─── SCIENCE ─────────────────────────────────────────────────────────────
    Entry { is_header: true,  app: None,                   name: b" SCIENCE", desc: b"", tag: b"" },
    Entry { is_header: false, app: Some(AppKind::Menger),  name: b"Menger",  desc: b"3D Menger Sponge, ray marching",  tag: b"[3D]    " },
    Entry { is_header: false, app: Some(AppKind::Fpu),     name: b"FPU",     desc: b"x87 log2, sqrt, entropy",         tag: b"[Math]  " },
    Entry { is_header: false, app: Some(AppKind::Voodoo),  name: b"Voodoo",  desc: b"Bayesian oracle + automata",      tag: b"[AI]    " },
    Entry { is_header: false, app: Some(AppKind::Rng),     name: b"RNG",     desc: b"RDRAND random + d6 dice",         tag: b"[RNG]   " },
    // ─── SYSTEM ──────────────────────────────────────────────────────────────
    Entry { is_header: true,  app: None,                   name: b" SYSTEM",  desc: b"", tag: b"" },
    Entry { is_header: false, app: Some(AppKind::Chronos), name: b"Chronos", desc: b"HEX clock + Psychotown time",     tag: b"[Clock] " },
    Entry { is_header: false, app: Some(AppKind::Rtc),     name: b"RTC",     desc: b"Hardware RTC, CMOS, battery",     tag: b"[HW]    " },
    Entry { is_header: false, app: Some(AppKind::Beeper),  name: b"Beeper",  desc: b"PC Speaker hexatonic scale",      tag: b"[Sound] " },
    Entry { is_header: false, app: Some(AppKind::Locale),  name: b"Language",desc: b"Switch RU / EN / AR interface",   tag: b"[Lang]  " },
    // ─────────────────────────────────────────────────────────────────────────
    Entry { is_header: true,  app: None,                   name: b"",         desc: b"", tag: b"" },
    Entry { is_header: false, app: Some(AppKind::Shell),  name: b"Shell",   desc: b"Back to IBIP terminal",           tag: b"[Exit]  " },
];

// ── Layout constants ──────────────────────────────────────────────────────────
const BOX_COL: usize = 13;
const BOX_W:   usize = 53;
const BOX_ROW: usize = 1;   // breadcrumb на row 0, бокс с row 1 — влезает 16 строк в 25

// ── Lifecycle hooks (called by ActivityManager) ───────────────────────────────

pub fn on_start() {
    draw(0, 1);
}

pub fn on_resume() {
    draw(0, 1);
}

pub fn on_pause() {
    // Stateless launcher — nothing to save
}

/// Blocking poll: wait for user action, return what the manager should do.
/// `depth` = current ActivityStack depth (for breadcrumb dots).
pub fn update(depth: usize) -> ActivityIntent {
    // Начинаем с первой выбираемой записи (не заголовок)
    let mut sel: usize = 0;
    while sel < ENTRIES.len() && ENTRIES[sel].is_header { sel += 1; }
    draw(sel, depth);

    unsafe {
        loop {
            if crate::ps2::has_scancode() {
                let sc = crate::ps2::read_scancode();
                if sc & 0x80 != 0 { continue; } // key-up

                match sc {
                    0x48 => { // ↑ — пропускаем заголовки
                        let mut s = if sel == 0 { ENTRIES.len() - 1 } else { sel - 1 };
                        while ENTRIES[s].is_header { s = if s == 0 { ENTRIES.len() - 1 } else { s - 1 }; }
                        sel = s;
                        draw(sel, depth);
                    }
                    0x50 => { // ↓ — пропускаем заголовки
                        let mut s = (sel + 1) % ENTRIES.len();
                        while ENTRIES[s].is_header { s = (s + 1) % ENTRIES.len(); }
                        sel = s;
                        draw(sel, depth);
                    }
                    0x1C => { // Enter
                        return match ENTRIES[sel].app {
                            Some(kind) => ActivityIntent::Push(kind),
                            None       => ActivityIntent::Pop,
                        };
                    }
                    0x01 => return ActivityIntent::Pop, // Esc
                    _ => {}
                }
            }
            hlt();
        }
    }
}

// ── Rendering ─────────────────────────────────────────────────────────────────

fn draw(sel: usize, depth: usize) {
    unsafe { render(sel, depth); }
}

unsafe fn render(sel: usize, depth: usize) {
    fill_screen(BG);

    // Breadcrumb: на row 0 (бокс с row 1)
    puts(0, 2, b"[ NeroShizaDev-OS ]", TITLE);
    puts(0, 21, b" \xB7 APPS", BORDER);
    if depth > 1 {
        let d = (depth as u8).min(8) as usize;
        for i in 0..d { put(0, 68 + i, 0xF9, HINT); }
    }

    // Top border
    put(BOX_ROW, BOX_COL, TL, BORDER);
    for c in 1..=BOX_W { put(BOX_ROW, BOX_COL + c, HZ, BORDER); }
    put(BOX_ROW, BOX_COL + BOX_W + 1, TR, BORDER);

    // Header row
    let h = BOX_ROW + 1;
    put(h, BOX_COL, VT, BORDER);
    fill_seg(h, BOX_COL + 1, BOX_W, TITLE);
    puts(h, BOX_COL + 11, b"LAUNCH PAD  \xB7  NeroShizaDev-OS  \xB7  HYBRID", TITLE);
    put(h, BOX_COL + BOX_W + 1, VT, BORDER);

    // Divider
    let div = BOX_ROW + 2;
    put(div, BOX_COL, ML, BORDER);
    for c in 1..=BOX_W { put(div, BOX_COL + c, HZ, BORDER); }
    put(div, BOX_COL + BOX_W + 1, MR, BORDER);

    // Gap before items — убран, заголовки секций заменяют пустую строку

    // Menu items (с заголовками секций как Windows Start)
    for (i, e) in ENTRIES.iter().enumerate() {
        let row = BOX_ROW + 3 + i;  // items с BOX_ROW+3 (header+divider заняли +2)

        if e.is_header {
            // Секция-разделитель: заполняем горизонтальными линиями, поверх пишем название
            put(row, BOX_COL, VT, BORDER);
            for c in 1..=BOX_W { put(row, BOX_COL + c, 0xC4, HINT); }
            if !e.name.is_empty() {
                put(row, BOX_COL + 2, b' ', HINT);
                puts(row, BOX_COL + 3, e.name, TAG);
                let end = BOX_COL + 3 + e.name.len();
                if end < BOX_COL + BOX_W { put(row, end, b' ', HINT); }
            }
            put(row, BOX_COL + BOX_W + 1, VT, BORDER);
        } else {
            let is_sel = i == sel;
            let fg     = if is_sel { HILIT } else { NORMAL };
            let row_bg = if is_sel { HILIT } else { BG };

            put(row, BOX_COL, VT, BORDER);
            fill_seg(row, BOX_COL + 1, BOX_W, row_bg);

            put(row, BOX_COL + 2, if is_sel { ARW } else { b' ' }, fg);
            put(row, BOX_COL + 3, b' ', fg);

            let nc = BOX_COL + 4;
            puts(row, nc, e.name, fg);
            for p in e.name.len()..9 { put(row, nc + p, b' ', fg); }

            put(row, BOX_COL + 13, SEP, if is_sel { HILIT } else { BORDER });

            puts(row, BOX_COL + 15, e.desc, fg);

            let tc = BOX_COL + BOX_W + 1 - e.tag.len() - 1;
            puts(row, tc, e.tag, if is_sel { HILIT } else { TAG });

            put(row, BOX_COL + BOX_W + 1, VT, BORDER);
        }
    }

    // Gap after items
    let gap2 = BOX_ROW + 3 + ENTRIES.len();
    vt_blank(gap2);

    // Bottom border
    let bot = gap2 + 1;
    put(bot, BOX_COL, BL, BORDER);
    for c in 1..=BOX_W { put(bot, BOX_COL + c, HZ, BORDER); }
    put(bot, BOX_COL + BOX_W + 1, BR, BORDER);

    // Controls hint
    puts(bot + 2, 17, b"\x18\x19 navigate    Enter launch    Esc back to shell", HINT);
}

unsafe fn vt_blank(row: usize) {
    put(row, BOX_COL, VT, BORDER);
    fill_seg(row, BOX_COL + 1, BOX_W, BG);
    put(row, BOX_COL + BOX_W + 1, VT, BORDER);
}
