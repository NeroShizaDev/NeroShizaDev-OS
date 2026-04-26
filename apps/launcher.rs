use crate::apps::activity::{ActivityIntent, AppKind};
use crate::kernel_messages::Locale;
/// APPS Launcher — TUI home screen activity.
///
/// Lifecycle (called by ActivityManager):
///   on_start()       — первый запуск, рисуем экран
///   update(depth)    — блокирующий опрос клавиш → ActivityIntent
///   on_pause()       — нас перекрыли сверху (ничего не делаем)
///   on_resume()      — вернулись к нам, перерисовываем экран
use x86_64::instructions::hlt;

// ── Direct VGA helpers (no WRITER lock — launcher owns the screen) ───────────
const VGA: *mut u8 = 0xb8000 as *mut u8;

unsafe fn put(row: usize, col: usize, ch: u8, color: u8) {
    if row >= 25 || col >= 80 {
        return;
    }
    let off = (row * 80 + col) * 2;
    core::ptr::write_volatile(VGA.add(off), ch);
    core::ptr::write_volatile(VGA.add(off + 1), color);
}

unsafe fn fill_seg(row: usize, col: usize, len: usize, color: u8) {
    for c in 0..len {
        put(row, col + c, b' ', color);
    }
}

unsafe fn puts(row: usize, col: usize, s: &[u8], color: u8) {
    for (i, &b) in s.iter().enumerate() {
        put(row, col + i, b, color);
    }
}

unsafe fn puts_utf8(row: usize, col: usize, s: &str, color: u8) -> usize {
    let mut x = col;
    for ch in s.chars() {
        if row >= 25 || x >= 80 {
            break;
        }
        x += crate::vga_unicode::print_char(ch as u32, x, row, color).max(1);
    }
    x.saturating_sub(col)
}

fn utf8_cell_len(s: &str) -> usize {
    s.chars().count()
}

unsafe fn fill_screen(color: u8) {
    for r in 0..25usize {
        fill_seg(r, 0, 80, color);
    }
}

// ── ASCII frame chars ────────────────────────────────────────────────────────
const TL: u8 = b'+';
const TR: u8 = b'+';
const BL: u8 = b'+';
const BR: u8 = b'+';
const HZ: u8 = b'-';
const VT: u8 = b'|';
const ML: u8 = b'+';
const MR: u8 = b'+';
const SL: u8 = b'+';
const SR: u8 = b'+';
const SEP: u8 = b'|';
const ARW: u8 = 0x10; // ►

// ── Colors (Blue bg palette) ──────────────────────────────────────────────────
const BG: u8 = 0x10; // Black  on Blue
const BORDER: u8 = 0x1B; // Cyan   on Blue
const TITLE: u8 = 0x1E; // Yellow on Blue
const NORMAL: u8 = 0x17; // LtGray on Blue
const HILIT: u8 = 0x70; // Black  on White (selected bar)
const HINT: u8 = 0x18; // DkGray on Blue
const TAG: u8 = 0x1A; // LtGreen on Blue

// ── Menu entries ─────────────────────────────────────────────────────────────
// app: None = "back to shell" (Pop), Some(X) = Push(X)

struct Entry {
    app: Option<AppKind>,
    name: &'static [u8],
    desc: &'static [u8],
    tag: &'static [u8],
    is_header: bool, // true — секция-разделитель, не выбирается
}

fn current_locale() -> Locale {
    crate::locale::get_locale()
}

fn apps_breadcrumb_text() -> &'static str {
    match current_locale() {
        Locale::RuRu => " · ПРИЛОЖЕНИЯ",
        Locale::EnUs => " · APPS",
        Locale::ArEg => " · التطبيقات",
    }
}

fn launchpad_title_text() -> &'static str {
    match current_locale() {
        Locale::RuRu => "ЦЕНТР ЗАПУСКА  ·  NeroShizaDev-OS  ·  HYBRID",
        Locale::EnUs => "LAUNCH PAD  ·  NeroShizaDev-OS  ·  HYBRID",
        Locale::ArEg => "مركز التشغيل  ·  NeroShizaDev-OS  ·  HYBRID",
    }
}

fn controls_hint_text() -> &'static str {
    match current_locale() {
        Locale::RuRu => "^v выбор    Enter запуск    Esc назад в shell",
        Locale::EnUs => "^v navigate    Enter launch    Esc back to shell",
        Locale::ArEg => "^v تنقل    Enter تشغيل    Esc رجوع إلى shell",
    }
}

fn header_text(name: &[u8]) -> &'static str {
    match (current_locale(), name) {
        (Locale::RuRu, b" GAMES") => " ИГРЫ",
        (Locale::RuRu, b" TOOLS") => " ИНСТРУМЕНТЫ",
        (Locale::RuRu, b" SCIENCE") => " НАУКА",
        (Locale::RuRu, b" SYSTEM") => " СИСТЕМА",
        (Locale::RuRu, b" INSTALLED") => " УСТАНОВЛЕНО",
        (Locale::EnUs, b" GAMES") => " GAMES",
        (Locale::EnUs, b" TOOLS") => " TOOLS",
        (Locale::EnUs, b" SCIENCE") => " SCIENCE",
        (Locale::EnUs, b" SYSTEM") => " SYSTEM",
        (Locale::EnUs, b" INSTALLED") => " INSTALLED",
        (Locale::ArEg, b" GAMES") => " الألعاب",
        (Locale::ArEg, b" TOOLS") => " الأدوات",
        (Locale::ArEg, b" SCIENCE") => " العلوم",
        (Locale::ArEg, b" SYSTEM") => " النظام",
        (Locale::ArEg, b" INSTALLED") => " المثبت",
        _ => "",
    }
}

fn entry_text(e: &Entry) -> (&'static str, &'static str, &'static str) {
    match (current_locale(), e.app) {
        (Locale::RuRu, Some(AppKind::Games)) => {
            ("Игры", "Doom, Племя, аркады, карты и ещё", "[7в1]")
        }
        (Locale::RuRu, Some(AppKind::Jackal)) => {
            ("Jackal", "Анализатор и архиватор Jackal", "[Инстр]")
        }
        (Locale::RuRu, Some(AppKind::Menger)) => {
            ("Menger", "3D губка Менгера и ray marching", "[3D]")
        }
        (Locale::RuRu, Some(AppKind::Calculator)) => (
            "Калькулятор",
            "IEEE 754 / ISO 60559 инженерный режим",
            "[Наука]",
        ),
        (Locale::RuRu, Some(AppKind::Fpu)) => ("FPU", "x87 log2, sqrt, энтропия", "[Матем]"),
        (Locale::RuRu, Some(AppKind::Voodoo)) => ("Voodoo", "Байесов оракул и автоматы", "[ИИ]"),
        (Locale::RuRu, Some(AppKind::Rng)) => ("RNG", "RDRAND и броски d6", "[RNG]"),
        (Locale::RuRu, Some(AppKind::Chronos)) => {
            ("Chronos", "HEX-часы и время Psychotown", "[Часы]")
        }
        (Locale::RuRu, Some(AppKind::Rtc)) => ("RTC", "Аппаратный RTC, CMOS, батарея", "[Железо]"),
        (Locale::RuRu, Some(AppKind::Beeper)) => ("Beeper", "PC Speaker и гексатоника", "[Звук]"),
        (Locale::RuRu, Some(AppKind::Locale)) => ("Язык", "Переключить RU / EN / AR", "[Язык]"),
        (Locale::RuRu, None) if !e.is_header => ("Shell", "Назад в терминал IBIP", "[Выход]"),
        (Locale::ArEg, Some(AppKind::Games)) => {
            ("الألعاب", "Doom و Tribe وخمس ألعاب أخرى", "[7في1]")
        }
        (Locale::ArEg, Some(AppKind::Jackal)) => ("Jackal", "محلل وأرشيف Jackal", "[أداة]"),
        (Locale::ArEg, Some(AppKind::Menger)) => ("Menger", "إسفنجة منغر ثلاثية الأبعاد", "[3D]"),
        (Locale::ArEg, Some(AppKind::Calculator)) => {
            ("حاسبة", "هندسية IEEE 754 / ISO 60559", "[علم]")
        }
        (Locale::ArEg, Some(AppKind::Fpu)) => ("FPU", "x87 log2 و sqrt والانتروبيا", "[رياضيات]"),
        (Locale::ArEg, Some(AppKind::Voodoo)) => ("Voodoo", "أوراكل بايزي وآلات", "[ذكاء]"),
        (Locale::ArEg, Some(AppKind::Rng)) => ("RNG", "RDRAND ونرد d6", "[RNG]"),
        (Locale::ArEg, Some(AppKind::Chronos)) => ("Chronos", "ساعة HEX ووقت Psychotown", "[وقت]"),
        (Locale::ArEg, Some(AppKind::Rtc)) => ("RTC", "RTC عتادي و CMOS والبطارية", "[عتاد]"),
        (Locale::ArEg, Some(AppKind::Beeper)) => ("Beeper", "PC Speaker وسلم سداسي", "[صوت]"),
        (Locale::ArEg, Some(AppKind::Locale)) => ("اللغة", "بدل واجهة RU / EN / AR", "[لغة]"),
        (Locale::ArEg, None) if !e.is_header => ("Shell", "رجوع إلى طرفية IBIP", "[خروج]"),
        _ => (
            core::str::from_utf8(e.name).unwrap_or(""),
            core::str::from_utf8(e.desc).unwrap_or(""),
            core::str::from_utf8(e.tag).unwrap_or(""),
        ),
    }
}

fn debug_item_name(item: &LItem) -> &'static str {
    match item.kind {
        IKind::Stat => {
            let e = &ENTRIES[item.index];
            if e.is_header {
                header_text(e.name)
            } else {
                entry_text(e).0
            }
        }
        IKind::NhsH => nhs_header_text(),
        IKind::NhsA => "NHS",
    }
}

fn nhs_header_text() -> &'static str {
    match current_locale() {
        Locale::RuRu => " УСТАНОВЛЕНО",
        Locale::EnUs => " INSTALLED",
        Locale::ArEg => " المثبت",
    }
}

fn nhs_desc_text(flags: u32) -> &'static str {
    let has_script = flags & crate::apps::installer::header::FLAG_HAS_SCRIPT != 0;
    let has_native = flags & crate::apps::installer::header::FLAG_HAS_NATIVE != 0;
    match (current_locale(), has_script, has_native) {
        (Locale::RuRu, true, _) => "NHS сценарий",
        (Locale::RuRu, _, true) => "NHS native модуль",
        (Locale::RuRu, _, _) => "NHS пакет",
        (Locale::EnUs, true, _) => "NHS script app",
        (Locale::EnUs, _, true) => "NHS native app",
        (Locale::EnUs, _, _) => "NHS package",
        (Locale::ArEg, true, _) => "تطبيق NHS سكربت",
        (Locale::ArEg, _, true) => "تطبيق NHS أصلي",
        (Locale::ArEg, _, _) => "حزمة NHS",
    }
}

fn nhs_tag_text(flags: u32) -> &'static str {
    let has_script = flags & crate::apps::installer::header::FLAG_HAS_SCRIPT != 0;
    let has_native = flags & crate::apps::installer::header::FLAG_HAS_NATIVE != 0;
    match (current_locale(), has_script, has_native) {
        (Locale::RuRu, true, _) => "[СЦН]",
        (Locale::RuRu, _, true) => "[BIN]",
        (Locale::RuRu, _, _) => "[NHS]",
        (Locale::EnUs, true, _) => "[SCR]",
        (Locale::EnUs, _, true) => "[BIN]",
        (Locale::EnUs, _, _) => "[NHS]",
        (Locale::ArEg, true, _) => "[نص]",
        (Locale::ArEg, _, true) => "[BIN]",
        (Locale::ArEg, _, _) => "[NHS]",
    }
}

// Группы как Windows Start — 4 секции + разделитель + выход
const ENTRIES: &[Entry] = &[
    // ─── GAMES ───────────────────────────────────────────────────────────────
    Entry {
        is_header: true,
        app: None,
        name: b" GAMES",
        desc: b"",
        tag: b"",
    },
    Entry {
        is_header: false,
        app: Some(AppKind::Games),
        name: b"Games",
        desc: b"Doom, Tribe, Dodge, Cards & more",
        tag: b"[7in1]  ",
    },
    // ─── TOOLS ───────────────────────────────────────────────────────────────
    Entry {
        is_header: true,
        app: None,
        name: b" TOOLS",
        desc: b"",
        tag: b"",
    },
    Entry {
        is_header: false,
        app: Some(AppKind::Jackal),
        name: b"Jackal",
        desc: b"Jackal analyzer & archiver",
        tag: b"[Tool]  ",
    },
    // ─── SCIENCE ─────────────────────────────────────────────────────────────
    Entry {
        is_header: true,
        app: None,
        name: b" SCIENCE",
        desc: b"",
        tag: b"",
    },
    Entry {
        is_header: false,
        app: Some(AppKind::Menger),
        name: b"Menger",
        desc: b"3D Menger Sponge, ray marching",
        tag: b"[3D]    ",
    },
    Entry {
        is_header: false,
        app: Some(AppKind::Calculator),
        name: b"Calculator",
        desc: b"Engineering IEEE 754 / ISO 60559",
        tag: b"[Calc]  ",
    },
    Entry {
        is_header: false,
        app: Some(AppKind::Fpu),
        name: b"FPU",
        desc: b"x87 log2, sqrt, entropy",
        tag: b"[Math]  ",
    },
    Entry {
        is_header: false,
        app: Some(AppKind::Voodoo),
        name: b"Voodoo",
        desc: b"Bayesian oracle + automata",
        tag: b"[AI]    ",
    },
    Entry {
        is_header: false,
        app: Some(AppKind::Rng),
        name: b"RNG",
        desc: b"RDRAND random + d6 dice",
        tag: b"[RNG]   ",
    },
    // ─── SYSTEM ──────────────────────────────────────────────────────────────
    Entry {
        is_header: true,
        app: None,
        name: b" SYSTEM",
        desc: b"",
        tag: b"",
    },
    Entry {
        is_header: false,
        app: Some(AppKind::Chronos),
        name: b"Chronos",
        desc: b"HEX clock + Psychotown time",
        tag: b"[Clock] ",
    },
    Entry {
        is_header: false,
        app: Some(AppKind::Rtc),
        name: b"RTC",
        desc: b"Hardware RTC, CMOS, battery",
        tag: b"[HW]    ",
    },
    Entry {
        is_header: false,
        app: Some(AppKind::Beeper),
        name: b"Beeper",
        desc: b"PC Speaker hexatonic scale",
        tag: b"[Sound] ",
    },
    Entry {
        is_header: false,
        app: Some(AppKind::Locale),
        name: b"Language",
        desc: b"Switch RU / EN / AR interface",
        tag: b"[Lang]  ",
    },
    // ─────────────────────────────────────────────────────────────────────────
    Entry {
        is_header: true,
        app: None,
        name: b"",
        desc: b"",
        tag: b"",
    },
    Entry {
        is_header: false,
        app: None,
        name: b"Shell",
        desc: b"Back to IBIP terminal",
        tag: b"[Exit]  ",
    },
];

// ── Layout constants ──────────────────────────────────────────────────────────
const BOX_COL: usize = 0; // полный экран: бокс от col 0
const BOX_W: usize = 78; // inner width: cols 1..78, border at 0 и 79
const BOX_ROW: usize = 1; // breadcrumb на row 0, бокс с row 1
const VISIBLE_ITEMS: usize = 19; // rows 4..22 — без gap-строки, bot=23, hint=24

// ── Merged item list — static + dynamic NHS ───────────────────────────────────

/// Индекс NHS-слота для запуска. Устанавливается update() до Push(AppKind::Nhs).
pub static mut LAUNCH_NHS_SLOT: u8 = 0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum IKind {
    Stat, // ENTRIES[index]
    NhsH, // заголовок секции NHS INSTALLED
    NhsA, // NHS-приложение из реестра, index = номер слота
}

#[derive(Clone, Copy)]
struct LItem {
    kind: IKind,
    index: usize,
    is_header: bool,
}

const MAX_ITEMS: usize = 36;

/// Строим объединённый список: статические ENTRIES + NHS-секция из реестра.
/// NHS-секция вставляется перед последними 2 записями (разделитель + Shell).
fn build_items(buf: &mut [LItem; MAX_ITEMS]) -> usize {
    let mut n = 0;

    // Статические записи до последних 2 (пустой разделитель + Shell)
    let split_at = ENTRIES.len().saturating_sub(2);
    for i in 0..split_at {
        if n >= MAX_ITEMS {
            break;
        }
        buf[n] = LItem {
            kind: IKind::Stat,
            index: i,
            is_header: ENTRIES[i].is_header,
        };
        n += 1;
    }

    // NHS-секция — только если есть установленные пакеты
    let mut has_nhs = false;
    for slot in 0..crate::apps::installer::slots::MAX_SLOTS {
        if crate::apps::installer::registry::get(slot).is_some() {
            has_nhs = true;
            break;
        }
    }
    if has_nhs {
        if n < MAX_ITEMS {
            buf[n] = LItem {
                kind: IKind::NhsH,
                index: 0,
                is_header: true,
            };
            n += 1;
        }
        for slot in 0..crate::apps::installer::slots::MAX_SLOTS {
            if crate::apps::installer::registry::get(slot).is_some() {
                if n < MAX_ITEMS {
                    buf[n] = LItem {
                        kind: IKind::NhsA,
                        index: slot,
                        is_header: false,
                    };
                    n += 1;
                }
            }
        }
    }

    // Последние 2 статические записи (разделитель + Shell)
    for i in split_at..ENTRIES.len() {
        if n >= MAX_ITEMS {
            break;
        }
        buf[n] = LItem {
            kind: IKind::Stat,
            index: i,
            is_header: ENTRIES[i].is_header,
        };
        n += 1;
    }

    n
}

fn first_selectable(items: &[LItem; MAX_ITEMS], n: usize) -> usize {
    let mut s = 0;
    while s < n && items[s].is_header {
        s += 1;
    }
    s
}

// ── Lifecycle hooks (called by ActivityManager) ───────────────────────────────

pub fn on_start() {
    let mut items = [LItem {
        kind: IKind::Stat,
        index: 0,
        is_header: false,
    }; MAX_ITEMS];
    let n = build_items(&mut items);
    let sel = first_selectable(&items, n);
    unsafe {
        render(&items, n, sel, 0, 1);
    }
}

pub fn on_resume() {
    // Drain any scancodes accumulated while another app was running.
    // Without this, keys pressed during FPU/Menger/etc. animation phases
    // leak into the Launcher and can trigger accidental Esc → Pop → shell.
    crate::ps2::clear_scancode_queue();
    let mut items = [LItem {
        kind: IKind::Stat,
        index: 0,
        is_header: false,
    }; MAX_ITEMS];
    let n = build_items(&mut items);
    let sel = first_selectable(&items, n);
    unsafe {
        render(&items, n, sel, 0, 1);
    }
}

pub fn on_pause() {
    // Stateless launcher — nothing to save
}

/// Blocking poll: wait for user action, return what the manager should do.
/// `depth` = current ActivityStack depth (for breadcrumb dots).
pub fn update(depth: usize) -> ActivityIntent {
    // Собираем объединённый список (статика + NHS-реестр)
    let mut items = [LItem {
        kind: IKind::Stat,
        index: 0,
        is_header: false,
    }; MAX_ITEMS];
    let n = build_items(&mut items);

    let mut sel: usize = first_selectable(&items, n);
    let mut scroll: usize = 0;
    unsafe {
        render(&items, n, sel, scroll, depth);
        crate::serial_println!(
            "[APPS][SEL] init sel={} item={} scroll={} depth={}",
            sel,
            debug_item_name(&items[sel]),
            scroll,
            depth
        );
    }

    unsafe {
        loop {
            if crate::ps2::has_scancode() {
                let mut sc = crate::ps2::read_scancode();
                if sc & 0x80 != 0 {
                    continue;
                } // key-up

                if sc == 0xE0 {
                    if !crate::ps2::has_scancode() {
                        continue;
                    }
                    sc = crate::ps2::read_scancode();
                    if sc & 0x80 != 0 {
                        continue;
                    }
                }

                match sc {
                    0x48 => {
                        // ↑ — пропускаем заголовки
                        let mut s = if sel == 0 { n - 1 } else { sel - 1 };
                        while s > 0 && items[s].is_header {
                            s -= 1;
                        }
                        if items[s].is_header {
                            // wrap-around всё ещё на заголовке
                            s = n - 1;
                            while s > 0 && items[s].is_header {
                                s -= 1;
                            }
                        }
                        sel = s;
                        if sel < scroll {
                            scroll = sel;
                        }
                        crate::serial_println!(
                            "[APPS][SEL] up sel={} item={} scroll={}",
                            sel,
                            debug_item_name(&items[sel]),
                            scroll
                        );
                        render(&items, n, sel, scroll, depth);
                    }
                    0x50 => {
                        // ↓ — пропускаем заголовки
                        let mut s = sel + 1;
                        if s >= n {
                            s = 0;
                        }
                        while s < n && items[s].is_header {
                            s += 1;
                        }
                        if s >= n {
                            s = 0;
                            while s < n && items[s].is_header {
                                s += 1;
                            }
                        }
                        sel = s;
                        if sel >= scroll + VISIBLE_ITEMS {
                            scroll = sel + 1 - VISIBLE_ITEMS;
                        }
                        crate::serial_println!(
                            "[APPS][SEL] down sel={} item={} scroll={}",
                            sel,
                            debug_item_name(&items[sel]),
                            scroll
                        );
                        render(&items, n, sel, scroll, depth);
                    }
                    0x1C => {
                        // Enter
                        let item = items[sel];
                        crate::serial_println!(
                            "[APPS][CONFIRM] sel={} item={} kind={}",
                            sel,
                            debug_item_name(&item),
                            match item.kind {
                                IKind::Stat => "stat",
                                IKind::NhsH => "nhs-header",
                                IKind::NhsA => "nhs-app",
                            }
                        );
                        return match item.kind {
                            IKind::Stat => match ENTRIES[item.index].app {
                                Some(kind) => ActivityIntent::Push(kind),
                                None => ActivityIntent::Pop,
                            },
                            IKind::NhsA => {
                                LAUNCH_NHS_SLOT = item.index as u8;
                                ActivityIntent::Push(AppKind::Nhs)
                            }
                            IKind::NhsH => ActivityIntent::Continue,
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

unsafe fn render(items: &[LItem; MAX_ITEMS], n: usize, sel: usize, scroll: usize, depth: usize) {
    fill_screen(BG);

    // Breadcrumb: на row 0
    puts_utf8(0, 2, "[ NeroShizaDev-OS ]", TITLE);
    puts_utf8(0, 21, apps_breadcrumb_text(), BORDER);
    if depth > 1 {
        let d = (depth as u8).min(8) as usize;
        for i in 0..d {
            put(0, 68 + i, 0xF9, HINT);
        }
    }

    // Top border
    put(BOX_ROW, BOX_COL, TL, BORDER);
    for c in 1..=BOX_W {
        put(BOX_ROW, BOX_COL + c, HZ, BORDER);
    }
    put(BOX_ROW, BOX_COL + BOX_W + 1, TR, BORDER);

    // Header row
    let h = BOX_ROW + 1;
    put(h, BOX_COL, VT, BORDER);
    fill_seg(h, BOX_COL + 1, BOX_W, TITLE);
    puts_utf8(h, BOX_COL + 10, launchpad_title_text(), TITLE);
    put(h, BOX_COL + BOX_W + 1, VT, BORDER);

    // Divider
    let div = BOX_ROW + 2;
    put(div, BOX_COL, ML, BORDER);
    for c in 1..=BOX_W {
        put(div, BOX_COL + c, HZ, BORDER);
    }
    put(div, BOX_COL + BOX_W + 1, MR, BORDER);

    // Menu items (с прокруткой)
    for vis in 0..VISIBLE_ITEMS {
        let i = scroll + vis;
        let row = BOX_ROW + 3 + vis;

        if i >= n {
            // Строки за пределами списка — пустые
            vt_blank(row);
            continue;
        }

        let item = items[i];
        match item.kind {
            IKind::Stat => {
                let e = &ENTRIES[item.index];
                if e.is_header {
                    render_section_header(row, e.name);
                } else {
                    render_static_entry(row, e, i == sel);
                }
            }
            IKind::NhsH => render_section_header(row, nhs_header_text().as_bytes()),
            IKind::NhsA => render_nhs_entry(row, item.index, i == sel),
        }
    }

    // Индикаторы прокрутки ▲ ▼
    if scroll > 0 {
        put(BOX_ROW + 3, BOX_COL + BOX_W, 0x1E, BORDER); // ▲
    }
    if scroll + VISIBLE_ITEMS < n {
        put(
            BOX_ROW + 3 + VISIBLE_ITEMS - 1,
            BOX_COL + BOX_W,
            0x1F,
            BORDER,
        ); // ▼
    }

    // Bottom border (row 23, вплотную к items — gap убран для fullscreen)
    let bot = BOX_ROW + 3 + VISIBLE_ITEMS;
    put(bot, BOX_COL, BL, BORDER);
    for c in 1..=BOX_W {
        put(bot, BOX_COL + c, HZ, BORDER);
    }
    put(bot, BOX_COL + BOX_W + 1, BR, BORDER);

    // Controls hint (row 24 — последняя строка экрана)
    puts_utf8(bot + 1, 13, controls_hint_text(), HINT);

    // Locale badge (row 0, col 72..79) — перерисовываем после fill_screen
    crate::locale::draw_locale_badge();
}

unsafe fn render_section_header(row: usize, name: &[u8]) {
    put(row, BOX_COL, SL, BORDER);
    for c in 1..=BOX_W {
        put(row, BOX_COL + c, b'-', HINT);
    }
    if !name.is_empty() {
        put(row, BOX_COL + 2, b' ', HINT);
        let text = header_text(name);
        let width = puts_utf8(row, BOX_COL + 3, text, TAG);
        let end = BOX_COL + 3 + width;
        if end < BOX_COL + BOX_W {
            put(row, end, b' ', HINT);
        }
    }
    put(row, BOX_COL + BOX_W + 1, SR, BORDER);
}

unsafe fn render_static_entry(row: usize, e: &Entry, is_sel: bool) {
    let fg = if is_sel { HILIT } else { NORMAL };
    let row_bg = if is_sel { HILIT } else { BG };

    put(row, BOX_COL, VT, BORDER);
    fill_seg(row, BOX_COL + 1, BOX_W, row_bg);
    put(row, BOX_COL + 2, if is_sel { ARW } else { b' ' }, fg);
    put(row, BOX_COL + 3, b' ', fg);

    let nc = BOX_COL + 4;
    let (name, desc, tag) = entry_text(e);
    let name_w = puts_utf8(row, nc, name, fg);
    for p in name_w..9 {
        put(row, nc + p, b' ', fg);
    }
    put(row, BOX_COL + 13, SEP, if is_sel { HILIT } else { BORDER });
    puts_utf8(row, BOX_COL + 15, desc, fg);
    let tc = BOX_COL + BOX_W + 1 - utf8_cell_len(tag) - 1;
    puts_utf8(row, tc, tag, if is_sel { HILIT } else { TAG });
    put(row, BOX_COL + BOX_W + 1, VT, BORDER);
}

unsafe fn render_nhs_entry(row: usize, slot: usize, is_sel: bool) {
    let fg = if is_sel { HILIT } else { NORMAL };
    let row_bg = if is_sel { HILIT } else { BG };

    put(row, BOX_COL, VT, BORDER);
    fill_seg(row, BOX_COL + 1, BOX_W, row_bg);
    put(row, BOX_COL + 2, if is_sel { ARW } else { b' ' }, fg);
    put(row, BOX_COL + 3, b' ', fg);

    let nc = BOX_COL + 4;

    if let Some(app) = crate::apps::installer::registry::get(slot) {
        // Имя приложения (до 9 символов)
        let name = app.name_str().as_bytes();
        let name_len = name.len().min(9);
        puts(row, nc, &name[..name_len], fg);
        for p in name_len..9 {
            put(row, nc + p, b' ', fg);
        }

        put(row, BOX_COL + 13, SEP, if is_sel { HILIT } else { BORDER });

        // Описание: тип пакета
        let desc = nhs_desc_text(app.flags);
        puts_utf8(row, BOX_COL + 15, desc, fg);

        // Тег в правой части
        let vtag = nhs_tag_text(app.flags);
        let tc = BOX_COL + BOX_W + 1 - utf8_cell_len(vtag) - 1;
        puts_utf8(row, tc, vtag, if is_sel { HILIT } else { TAG });
    } else {
        puts(row, nc, b"<empty>  ", fg);
    }

    put(row, BOX_COL + BOX_W + 1, VT, BORDER);
}

unsafe fn vt_blank(row: usize) {
    put(row, BOX_COL, VT, BORDER);
    fill_seg(row, BOX_COL + 1, BOX_W, BG);
    put(row, BOX_COL + BOX_W + 1, VT, BORDER);
}
