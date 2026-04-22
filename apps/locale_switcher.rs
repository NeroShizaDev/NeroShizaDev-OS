/// Locale Switcher — APPS activity.
///
/// Показывает меню выбора языка (RU / EN / AR).
/// После выбора: устанавливает локаль, обновляет badge, Pop.

use x86_64::instructions::hlt;
use crate::apps::activity::ActivityIntent;
use crate::kernel_messages::Locale;

// ── Direct VGA (same helpers as launcher) ─────────────────────────────────────
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

// ── Colors ─────────────────────────────────────────────────────────────────────
const BG:     u8 = 0x10;
const BORDER: u8 = 0x1B;
const TITLE:  u8 = 0x1E;
const NORMAL: u8 = 0x17;
const HILIT:  u8 = 0x70;
const HINT:   u8 = 0x18;
const TAG:    u8 = 0x1A;
const ARW:    u8 = 0x10; // ►
const HZ:     u8 = 0xCD;
const VT:     u8 = 0xBA;
const TL:     u8 = 0xC9;
const TR:     u8 = 0xBB;
const BL:     u8 = 0xC8;
const BR:     u8 = 0xBC;
const ML:     u8 = 0xCC;
const MR:     u8 = 0xB9;

// ── Menu items ────────────────────────────────────────────────────────────────
struct LocaleEntry {
    locale: Locale,
    name:   &'static [u8],
    desc:   &'static [u8],
    tag:    &'static [u8],
}

const ITEMS: &[LocaleEntry] = &[
    LocaleEntry { locale: Locale::RuRu, name: b"RU", desc: b"Russian  \xB7  Cyrillic, LTR",  tag: b"[RU]" },
    LocaleEntry { locale: Locale::EnUs, name: b"EN", desc: b"English  \xB7  ASCII, LTR",      tag: b"[EN]" },
    LocaleEntry { locale: Locale::ArEg, name: b"AR", desc: b"Arabic   \xB7  RTL",             tag: b"[AR]" },
];

const BOX_COL: usize = 22;
const BOX_W:   usize = 34;
const BOX_ROW: usize = 7;

// ── Lifecycle ─────────────────────────────────────────────────────────────────

pub fn on_start() {
    unsafe { render(0); }
}

pub fn on_resume() {
    unsafe { render(0); }
}

pub fn on_pause() {}

/// Blocking poll — returns Pop after user picks a locale (or presses Esc).
pub fn update(_depth: usize) -> ActivityIntent {
    let mut sel: usize = 0;

    // Highlight current locale
    let cur = crate::locale::get_locale();
    for (i, e) in ITEMS.iter().enumerate() {
        if e.locale == cur { sel = i; break; }
    }

    unsafe { render(sel); }

    loop {
        unsafe {
        if crate::ps2::has_scancode() {
            let sc = crate::ps2::read_scancode();
            if sc & 0x80 != 0 { continue; } // key-up

            match sc {
                0x48 => { // ↑
                    sel = if sel == 0 { ITEMS.len() - 1 } else { sel - 1 };
                    render(sel);
                }
                0x50 => { // ↓
                    sel = (sel + 1) % ITEMS.len();
                    render(sel);
                }
                0x1C => { // Enter — apply & back
                    crate::locale::set_locale(ITEMS[sel].locale);
                    crate::locale::draw_locale_badge();
                    return ActivityIntent::Pop;
                }
                0x01 => return ActivityIntent::Pop, // Esc
                _ => {}
            }
        }
        } // unsafe
        hlt();
    }
}

// ── Rendering ─────────────────────────────────────────────────────────────────

unsafe fn render(sel: usize) {
    fill_screen(BG);

    // Breadcrumb
    puts(0, 2, b"[ NeroShizaDev-OS ]", TITLE);
    puts(0, 21, b" \xB7 APPS \xB7 Language", BORDER);

    // Top border
    put(BOX_ROW, BOX_COL, TL, BORDER);
    for c in 1..=BOX_W { put(BOX_ROW, BOX_COL + c, HZ, BORDER); }
    put(BOX_ROW, BOX_COL + BOX_W + 1, TR, BORDER);

    // Title bar
    let h = BOX_ROW + 1;
    put(h, BOX_COL, VT, BORDER);
    fill_seg(h, BOX_COL + 1, BOX_W, TITLE);
    puts(h, BOX_COL + 7, b"LANGUAGE / YAZYK / LUGHA", TITLE);
    put(h, BOX_COL + BOX_W + 1, VT, BORDER);

    // Divider
    let div = BOX_ROW + 2;
    put(div, BOX_COL, ML, BORDER);
    for c in 1..=BOX_W { put(div, BOX_COL + c, HZ, BORDER); }
    put(div, BOX_COL + BOX_W + 1, MR, BORDER);

    // Items
    for (i, e) in ITEMS.iter().enumerate() {
        let row    = BOX_ROW + 3 + i;
        let is_sel = i == sel;
        let fg     = if is_sel { HILIT } else { NORMAL };
        let rb     = if is_sel { HILIT } else { BG };

        put(row, BOX_COL, VT, BORDER);
        fill_seg(row, BOX_COL + 1, BOX_W, rb);

        put(row, BOX_COL + 2, if is_sel { ARW } else { b' ' }, fg);
        put(row, BOX_COL + 3, b' ', fg);

        // Name (2 chars)
        puts(row, BOX_COL + 4, e.name, fg);
        put(row, BOX_COL + 6, b' ', fg);
        put(row, BOX_COL + 7, b'\xB3', if is_sel { HILIT } else { BORDER }); // │

        // Desc
        puts(row, BOX_COL + 9, e.desc, fg);

        // Tag (right-align)
        let tc = BOX_COL + BOX_W - e.tag.len();
        puts(row, tc, e.tag, if is_sel { HILIT } else { TAG });

        put(row, BOX_COL + BOX_W + 1, VT, BORDER);
    }

    // Bottom border
    let bot = BOX_ROW + 3 + ITEMS.len();
    put(bot, BOX_COL, VT, BORDER);
    fill_seg(bot, BOX_COL + 1, BOX_W, BG);
    put(bot, BOX_COL + BOX_W + 1, VT, BORDER);

    let bot2 = bot + 1;
    put(bot2, BOX_COL, BL, BORDER);
    for c in 1..=BOX_W { put(bot2, BOX_COL + c, HZ, BORDER); }
    put(bot2, BOX_COL + BOX_W + 1, BR, BORDER);

    // Hint
    puts(bot2 + 2, 15, b"\x18\x19 select    Enter apply    Esc cancel", HINT);
}
