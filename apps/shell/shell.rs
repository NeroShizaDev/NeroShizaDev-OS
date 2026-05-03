// src/shell.rs — Ввод, история, clipboard, хоткеи, скроллбэк, диспетчер команд

use crate::print;
use crate::{
    apps, kernel_messages, locale, unicode, unicode_blocks, unicode_categories, unicode_scripts,
    validator, voodoo_engine,
};
use core::sync::atomic::{AtomicU8, Ordering};

const NHS_DEMO_PACKAGE: &[u8] = include_bytes!("../installer/nhsapps/demo.nhs");
const NHS_HELLO_PACKAGE: &[u8] = include_bytes!("../installer/nhsapps/hello.nhs");

// ============================================================
// ЕДИНАЯ СТРУКТУРА СОСТОЯНИЯ SHELL — все буферы в одном месте
// ============================================================
pub struct ShellState {
    pub buffer: [u32; 64],
    pub buffer_len: usize,
    pub cursor: usize,

    pub clipboard: [u32; 64],
    pub clipboard_len: usize,

    // 32 слота под историю + 1 слот (индекс 32) для HIST_SAVE
    pub history: [[u32; 64]; 33],
    pub history_lens: [usize; 33],
    pub history_count: usize,
    pub history_idx: usize,
    pub history_nav: isize,

    pub hotkeys: [[u32; 64]; 12],
    pub hotkey_lens: [usize; 12],
    pub recording_slot: Option<usize>,

    pub sel_active: bool,
    pub sel_start: usize,
    pub sel_end: usize,

    pub lang_rus: bool,
    pub confirm_pending: bool,
}

static mut SHELL: ShellState = ShellState {
    buffer: [0; 64],
    buffer_len: 0,
    cursor: 0,

    clipboard: [0; 64],
    clipboard_len: 0,

    history: [[0; 64]; 33],
    history_lens: [0; 33],
    history_count: 0,
    history_idx: 0,
    history_nav: -1,

    hotkeys: [[0; 64]; 12],
    hotkey_lens: [0; 12],
    recording_slot: None,

    sel_active: false,
    sel_start: 0,
    sel_end: 0,

    lang_rus: false,
    confirm_pending: false,
};

// ============================================================
// ГЛОБАЛЬНЫЕ ФЛАГИ МОДИФИКАТОРОВ (используются в обработчике прерываний)
// AtomicBool: пишется из ISR (keyboard_interrupt_handler), читается из основного потока.
// static mut bool — UB/data race, заменены на атомики.
// ============================================================
use core::sync::atomic::AtomicBool;
pub static ALT_HELD: AtomicBool = AtomicBool::new(false);
pub static CTRL_HELD: AtomicBool = AtomicBool::new(false);
pub static SHIFT_HELD: AtomicBool = AtomicBool::new(false);
pub static mut SCROLL_MODE: bool = false;
pub static mut SCROLL_OFFSET: usize = 0;
pub static mut HIST_NAV: isize = -1;
pub static mut CMD_TOTAL: u32 = 0;
pub static mut CMD_STATS: [u32; 9] = [0; 9];
static mut SERIAL_MON_TICK: u32 = 0;
static mut SERIAL_MON_EVERY: u32 = 600;
static mut LAST_IRQ_VIOLATIONS: u64 = 0;
static mut LAST_OOM_COUNT: usize = 0;
// Статистика и режим прокрутки оставлены как есть, если используются вне SHELL

const DEFERRED_NONE: u8 = 0;
const DEFERRED_SHUTDOWN: u8 = 1;
const DEFERRED_REBOOT: u8 = 2;
const DEFERRED_APPS: u8 = 4;

static DEFERRED_ACTION: AtomicU8 = AtomicU8::new(DEFERRED_NONE);

fn defer_action(code: u8) {
    DEFERRED_ACTION.store(code, Ordering::Release);
}

fn take_deferred_action() -> u8 {
    DEFERRED_ACTION.swap(DEFERRED_NONE, Ordering::AcqRel)
}

pub(crate) fn defer_shell_shutdown() {
    defer_action(DEFERRED_SHUTDOWN);
}

pub(crate) fn defer_shell_reboot() {
    defer_action(DEFERRED_REBOOT);
}

pub(crate) fn defer_shell_apps() {
    defer_action(DEFERRED_APPS);
}

pub(crate) fn bump_shell_command_total() {
    unsafe {
        CMD_TOTAL += 1;
    }
}

pub(crate) fn bump_shell_command_stat(index: usize) {
    unsafe {
        CMD_STATS[index] += 1;
    }
}

pub(crate) fn shell_command_total() -> u32 {
    unsafe { core::ptr::read_volatile(&raw const CMD_TOTAL) }
}

pub(crate) fn shell_command_stat(index: usize) -> u32 {
    unsafe { core::ptr::read_volatile(&raw const CMD_STATS[index]) }
}

pub(crate) fn shell_history_count() -> usize {
    unsafe { core::ptr::read_volatile(&raw const SHELL.history_count) }
}

pub(crate) fn shell_hotkey_len(slot: usize) -> usize {
    debug_assert!(slot < 12, "hotkey slot out of range: {}", slot);
    unsafe { core::ptr::read_volatile(&raw const SHELL.hotkey_lens[slot]) }
}

pub(crate) fn shell_hotkey_codepoint(slot: usize, index: usize) -> u32 {
    debug_assert!(slot < 12, "hotkey slot out of range: {}", slot);
    debug_assert!(index < 64, "hotkey index out of range: {}", index);
    unsafe { core::ptr::read_volatile(&raw const SHELL.hotkeys[slot][index]) }
}

fn shell_buffer_eq_ascii(expected: &str) -> bool {
    unsafe {
        if SHELL.buffer_len != expected.len() {
            return false;
        }

        for (i, &b) in expected.as_bytes().iter().enumerate() {
            let cp = SHELL.buffer[i];
            if cp > 0x7F {
                return false;
            }
            let mut actual = cp as u8;
            if actual >= b'A' && actual <= b'Z' {
                actual = actual - b'A' + b'a';
            }
            let mut target = b;
            if target >= b'A' && target <= b'Z' {
                target = target - b'A' + b'a';
            }
            if actual != target {
                return false;
            }
        }
        true
    }
}

fn handle_irq_guard_debug_command() -> bool {
    if shell_buffer_eq_ascii("irqdbg") {
        validator::display_irq_guard_status();
        return true;
    }

    if shell_buffer_eq_ascii("irqguard on") {
        crate::irq_guard::set_guard_enabled(true);
        locale::print_localized_line("[IRQGUARD] ON", 0x0A);
        return true;
    }

    if shell_buffer_eq_ascii("irqguard off") {
        crate::irq_guard::set_guard_enabled(false);
        locale::print_localized_line("[IRQGUARD] OFF", 0x0C);
        return true;
    }

    if shell_buffer_eq_ascii("irqguard reset") {
        crate::irq_guard::reset_counters();
        locale::print_localized_line("[IRQGUARD] counters reset", 0x0B);
        return true;
    }

    false
}

fn handle_log_debug_command() -> bool {
    let mut command = [0u8; 64];
    let len = match shell_buffer_to_ascii_lower(&mut command) {
        Some(len) => len,
        None => return false,
    };

    let line = match core::str::from_utf8(&command[..len]) {
        Ok(line) => line,
        Err(_) => return false,
    };

    let mut parts = line.split_whitespace();
    let Some(cmd) = parts.next() else {
        return false;
    };
    if cmd != "log" && cmd != "diag" {
        return false;
    }

    match parts.next() {
        Some("status") | None => {
            locale::print_localized_fmt(
                0x0B,
                format_args!(
                    "[LOG] level={} format={} dedup={} ps2={} apps={} games={} doom={} bite={} mem={} sys={} irqguard={} validator={} tribe={}",
                    crate::serial::min_level().as_str(),
                    crate::serial::format().as_str(),
                    if crate::serial::dedup_enabled() { 1 } else { 0 },
                    if crate::serial::subsys_enabled("PS2") {
                        1
                    } else {
                        0
                    },
                    if crate::serial::subsys_enabled("APPS") {
                        1
                    } else {
                        0
                    },
                    if crate::serial::subsys_enabled("GAMES") {
                        1
                    } else {
                        0
                    },
                    if crate::serial::subsys_enabled("DOOM") {
                        1
                    } else {
                        0
                    },
                    if crate::serial::subsys_enabled("BITE") {
                        1
                    } else {
                        0
                    },
                    if crate::serial::subsys_enabled("MEM") {
                        1
                    } else {
                        0
                    },
                    if crate::serial::subsys_enabled("SYS") {
                        1
                    } else {
                        0
                    },
                    if crate::serial::subsys_enabled("IRQGUARD") {
                        1
                    } else {
                        0
                    },
                    if crate::serial::subsys_enabled("VALIDATOR") {
                        1
                    } else {
                        0
                    },
                    if crate::serial::subsys_enabled("TRIBE") {
                        1
                    } else {
                        0
                    },
                ),
            );
            true
        }
        Some("level") => {
            let Some(level_name) = parts.next() else {
                locale::print_localized_line(
                    "[LOG] usage: log level trace|debug|info|warn|error|fatal",
                    0x0C,
                );
                return true;
            };
            match crate::serial::LogLevel::parse(level_name) {
                Some(level) => {
                    crate::serial::set_min_level(level);
                    locale::print_localized_fmt(
                        0x0A,
                        format_args!("[LOG] level={}", level.as_str()),
                    );
                }
                None => {
                    locale::print_localized_line("[LOG] bad level", 0x0C);
                }
            }
            true
        }
        Some("subsys") => {
            let Some(subsys) = parts.next() else {
                locale::print_localized_line("[LOG] usage: log subsys <name> on|off", 0x0C);
                return true;
            };
            let Some(state) = parts.next() else {
                locale::print_localized_line("[LOG] usage: log subsys <name> on|off", 0x0C);
                return true;
            };
            match state {
                "on" => {
                    crate::serial::set_subsys_enabled(subsys, true);
                    locale::print_localized_fmt(0x0A, format_args!("[LOG] subsys={} on", subsys));
                }
                "off" => {
                    crate::serial::set_subsys_enabled(subsys, false);
                    locale::print_localized_fmt(0x0A, format_args!("[LOG] subsys={} off", subsys));
                }
                _ => {
                    locale::print_localized_line("[LOG] usage: log subsys <name> on|off", 0x0C);
                }
            }
            true
        }
        Some("format") => {
            let Some(format_name) = parts.next() else {
                locale::print_localized_line(
                    "[LOG] usage: log format canonical|compact|logfmt",
                    0x0C,
                );
                return true;
            };
            match crate::serial::LogFormat::parse(format_name) {
                Some(format) => {
                    crate::serial::set_format(format);
                    locale::print_localized_fmt(
                        0x0A,
                        format_args!("[LOG] format={}", format.as_str()),
                    );
                }
                None => locale::print_localized_line("[LOG] bad format", 0x0C),
            }
            true
        }
        Some("dedup") => {
            let Some(state) = parts.next() else {
                locale::print_localized_line("[LOG] usage: log dedup on|off", 0x0C);
                return true;
            };
            match state {
                "on" => {
                    crate::serial::set_dedup_enabled(true);
                    locale::print_localized_line("[LOG] dedup=on", 0x0A);
                }
                "off" => {
                    crate::serial::set_dedup_enabled(false);
                    locale::print_localized_line("[LOG] dedup=off", 0x0A);
                }
                _ => locale::print_localized_line("[LOG] usage: log dedup on|off", 0x0C),
            }
            true
        }
        Some("preset") => {
            match parts.next() {
                Some("quiet") => {
                    crate::serial::set_min_level(crate::serial::LogLevel::Info);
                    crate::serial::set_subsys_enabled("PS2", false);
                    crate::serial::set_subsys_enabled("MEM", false);
                    crate::serial::set_format(crate::serial::LogFormat::Compact);
                    crate::serial::set_dedup_enabled(true);
                    locale::print_localized_line("[LOG] preset=quiet", 0x0A);
                }
                Some("debug") => {
                    crate::serial::set_min_level(crate::serial::LogLevel::Debug);
                    crate::serial::set_subsys_enabled("PS2", true);
                    crate::serial::set_subsys_enabled("MEM", true);
                    crate::serial::set_format(crate::serial::LogFormat::Canonical);
                    crate::serial::set_dedup_enabled(true);
                    locale::print_localized_line("[LOG] preset=debug", 0x0A);
                }
                Some("structured") => {
                    crate::serial::set_min_level(crate::serial::LogLevel::Info);
                    crate::serial::set_format(crate::serial::LogFormat::LogFmt);
                    crate::serial::set_dedup_enabled(true);
                    locale::print_localized_line("[LOG] preset=structured", 0x0A);
                }
                _ => locale::print_localized_line(
                    "[LOG] usage: log preset quiet|debug|structured",
                    0x0C,
                ),
            }
            true
        }
        _ => {
            locale::print_localized_line(
                "[LOG] usage: log status | log level ... | log subsys ... | log format ... | log dedup ... | log preset ...",
                0x0C,
            );
            true
        }
    }
}

fn shell_buffer_to_ascii_lower(out: &mut [u8; 64]) -> Option<usize> {
    unsafe {
        for i in 0..SHELL.buffer_len {
            let cp = SHELL.buffer[i];
            if cp > 0x7F {
                return None;
            }

            let mut byte = cp as u8;
            if byte >= b'A' && byte <= b'Z' {
                byte = byte - b'A' + b'a';
            }
            out[i] = byte;
        }

        Some(SHELL.buffer_len)
    }
}

fn print_nhs_usage() {
    locale::print_localized_line("[NHS] install demo | hello | serial | list", 0x0B);
    locale::print_localized_line("[NHS] listen  - alias for install serial", 0x08);
    locale::print_localized_line("[NHS] uninstall <slot>", 0x0B);
}

fn print_nhs_registry() {
    let installed = apps::installer::registry::installed_count();
    let free = apps::installer::slots::free_count();

    locale::print_localized_fmt(
        0x0B,
        format_args!("[NHS] installed={} free_slots={}", installed, free),
    );

    if installed == 0 {
        locale::print_localized_line("[NHS] registry is empty", 0x08);
        return;
    }

    apps::installer::registry::for_each(|slot, app| {
        let (major, minor, patch) = app.version_tuple();
        locale::print_localized_fmt(
            0x0E,
            format_args!(
                "[NHS] slot #{}  {}  v{}.{}.{}  {} bytes",
                slot,
                app.name_str(),
                major,
                minor,
                patch,
                app.installed_size,
            ),
        );
    });
}

fn run_nhs_install_serial() {
    locale::print_localized_line(
        "[NHS] Waiting for .nhs on COM1... (protocol: NHS_SYNC -> NHS_READY -> [size][data])",
        0x0E,
    );
    locale::print_localized_line(
        "[NHS] QEMU must use: -serial tcp:127.0.0.1:4321,server,nowait",
        0x08,
    );
    locale::print_localized_line(
        "[NHS] Host: python tools/send_nhs.py app.nhs --tcp localhost:4321",
        0x08,
    );
    match apps::installer::serial_recv::receive() {
        Ok(data) => run_nhs_install(data, "<serial>"),
        Err(e) => {
            locale::print_localized_fmt(0x0C, format_args!("[NHS] Receive error: {}", e.message()));
        }
    }
}

fn run_nhs_install(package: &[u8], label: &str) {
    let result = apps::installer::installer::install(package);
    crate::fb_buffer::clear_screen();
    locale::draw_locale_badge();

    match result {
        apps::installer::installer::InstallResult::Ok(slot) => {
            locale::print_localized_fmt(
                0x0A,
                format_args!("[NHS] Installed {} into slot #{}", label, slot),
            );
        }
        other => {
            locale::print_localized_fmt(
                0x0C,
                format_args!("[NHS] {}: {}", label, other.description()),
            );
        }
    }
}

fn handle_nhs_shell_command() -> bool {
    let mut command = [0u8; 64];
    let len = match shell_buffer_to_ascii_lower(&mut command) {
        Some(len) => len,
        None => return false,
    };

    let line = match core::str::from_utf8(&command[..len]) {
        Ok(line) => line,
        Err(_) => return false,
    };

    let mut parts = line.split_whitespace();
    let Some(cmd) = parts.next() else {
        return false;
    };

    match cmd {
        "install" => {
            unsafe {
                CMD_TOTAL += 1;
            }
            match parts.next() {
                Some("demo") => run_nhs_install(NHS_DEMO_PACKAGE, "demo.nhs"),
                Some("hello") => run_nhs_install(NHS_HELLO_PACKAGE, "hello.nhs"),
                Some("serial") => run_nhs_install_serial(),
                Some("list") => print_nhs_registry(),
                Some("help") | None => print_nhs_usage(),
                Some(other) => {
                    locale::print_localized_fmt(
                        0x0C,
                        format_args!("[NHS] unknown package: {}", other),
                    );
                    print_nhs_usage();
                }
            }
            true
        }
        "listen" => {
            unsafe {
                CMD_TOTAL += 1;
            }
            run_nhs_install_serial();
            true
        }
        "uninstall" => {
            unsafe {
                CMD_TOTAL += 1;
            }
            match parts.next().and_then(|slot| slot.parse::<usize>().ok()) {
                Some(slot) => {
                    if apps::installer::installer::uninstall(slot) {
                        locale::print_localized_fmt(
                            0x0A,
                            format_args!("[NHS] slot #{} removed", slot),
                        );
                    } else {
                        locale::print_localized_fmt(
                            0x0C,
                            format_args!("[NHS] slot #{} is not installed", slot),
                        );
                    }
                }
                None => print_nhs_usage(),
            }
            true
        }
        _ => false,
    }
}

enum FKeyMessage {
    Saved(usize),
    RecordPrompt,
    Empty,
}

fn print_fkey_message(slot: usize, message: FKeyMessage) {
    match (locale::get_locale(), message) {
        (kernel_messages::Locale::RuRu, FKeyMessage::Saved(len)) => locale::print_localized_fmt(
            0x0B,
            format_args!("[F{}: Принято! ({} симв.)]", slot + 1, len),
        ),
        (kernel_messages::Locale::EnUs, FKeyMessage::Saved(len)) => locale::print_localized_fmt(
            0x0B,
            format_args!("[F{}: Saved! ({} chars)]", slot + 1, len),
        ),
        (kernel_messages::Locale::ArEg, FKeyMessage::Saved(len)) => locale::print_localized_fmt(
            0x0B,
            format_args!("[F{}: تم الحفظ! ({} رمز)]", slot + 1, len),
        ),
        (kernel_messages::Locale::RuRu, FKeyMessage::RecordPrompt) => locale::print_localized_fmt(
            0x0B,
            format_args!("[F{}: Введи команду, потом Alt+F{}]", slot + 1, slot + 1),
        ),
        (kernel_messages::Locale::EnUs, FKeyMessage::RecordPrompt) => locale::print_localized_fmt(
            0x0B,
            format_args!("[F{}: Type command, then Alt+F{}]", slot + 1, slot + 1),
        ),
        (kernel_messages::Locale::ArEg, FKeyMessage::RecordPrompt) => locale::print_localized_fmt(
            0x0B,
            format_args!("[F{}: اكتب الأمر ثم Alt+F{}]", slot + 1, slot + 1),
        ),
        (kernel_messages::Locale::RuRu, FKeyMessage::Empty) => locale::print_localized_fmt(
            0x0B,
            format_args!("[F{}: Пусто. Alt+F{} для записи]", slot + 1, slot + 1),
        ),
        (kernel_messages::Locale::EnUs, FKeyMessage::Empty) => locale::print_localized_fmt(
            0x0B,
            format_args!("[F{}: Empty. Alt+F{} to record]", slot + 1, slot + 1),
        ),
        (kernel_messages::Locale::ArEg, FKeyMessage::Empty) => locale::print_localized_fmt(
            0x0B,
            format_args!("[F{}: فارغ. Alt+F{} للتسجيل]", slot + 1, slot + 1),
        ),
    }
}

fn show_irq_guard_first_hit_alert() {
    let msg = match locale::get_locale() {
        kernel_messages::Locale::RuRu => "[IRQGUARD] Опасный вызов из IRQ заблокирован и отложен.",
        kernel_messages::Locale::EnUs => "[IRQGUARD] Heavy call from IRQ blocked and deferred.",
        kernel_messages::Locale::ArEg => "[IRQGUARD] تم حظر نداء ثقيل من IRQ وتأجيله.",
    };
    let hits = crate::irq_guard::violation_count();

    crate::serial_println!(
        "[IRQGUARD][WARN] First hit captured: heavy operation from IRQ was blocked/deferred (violations={}).",
        hits
    );

    // Нижняя служебная строка: одноразовый алерт о перехвате опасного вызова.
    unsafe {
        locale::write_str_at_vga(
            "                                                                                ",
            23,
            0,
            0x07,
        );
        locale::write_str_at_vga(msg, 23, 0, 0x0E);
    }
}

fn emit_shell_serial_monitor() {
    unsafe {
        SERIAL_MON_TICK = SERIAL_MON_TICK.wrapping_add(1);
        if SERIAL_MON_TICK < SERIAL_MON_EVERY {
            return;
        }
        SERIAL_MON_TICK = 0;

        let act = apps::activity::debug_stats();
        let app = apps::activity::app_kind_name(act.current);
        let trace_depth = crate::trace::len();
        let irq_hits = crate::irq_guard::violation_count();

        // Doom heap-метрики (если Doom уже инициализировал свою кучу).
        let heap_used = apps::games::doom::stubs::heap_used_bytes();
        let heap_total = apps::games::doom::stubs::heap_total_bytes();
        let heap_peak = apps::games::doom::stubs::heap_peak_used_bytes();
        let heap_oom = apps::games::doom::stubs::heap_oom_count();
        let last_oom_req = apps::games::doom::stubs::heap_last_oom_request();
        let cmd_total = core::ptr::read_volatile(&raw const CMD_TOTAL);
        let history_count = core::ptr::read_volatile(&raw const SHELL.history_count);

        let irq_delta = irq_hits.saturating_sub(LAST_IRQ_VIOLATIONS);
        LAST_IRQ_VIOLATIONS = irq_hits;
        let oom_delta = heap_oom.saturating_sub(LAST_OOM_COUNT);
        LAST_OOM_COUNT = heap_oom;

        let hidden_err = heap_oom > 0 || irq_delta > 0;

        crate::serial_println!(
            "[SYS][INFO] +-------------------------------------------------------------+"
        );
        crate::serial_println!(
            "[SYS][INFO] | App {:<10} d={}/{} up={} sw={} p/r/pop={}/{}/{} |",
            app,
            act.depth,
            act.max_depth,
            act.updates,
            act.switches,
            act.pushes,
            act.replaces,
            act.pops
        );
        crate::serial_println!(
            "[SYS][INFO] | Shell cmd_total={} hist={} trace={} irq_hits={} (+{}) |",
            cmd_total,
            history_count,
            trace_depth,
            irq_hits,
            irq_delta
        );
        crate::serial_println!(
            "[SYS][INFO] | Heap used={}/{} peak={} oom={} (+{}) last_req={} |",
            heap_used,
            heap_total,
            heap_peak,
            heap_oom,
            oom_delta,
            last_oom_req
        );
        crate::serial_println!(
            "[SYS][INFO] | HiddenErrFlag {} |",
            if hidden_err { 1 } else { 0 }
        );
        crate::serial_println!(
            "[SYS][INFO] +-------------------------------------------------------------+"
        );

        // При тревоге сводку делаем чаще.
        SERIAL_MON_EVERY = if hidden_err { 180 } else { 600 };
    }
}

pub fn process_deferred_actions() {
    crate::ps2::flush_debug_serial(64);
    emit_shell_serial_monitor();

    if crate::irq_guard::take_first_hit_alert() {
        show_irq_guard_first_hit_alert();
    }

    match take_deferred_action() {
        DEFERRED_NONE => {}
        DEFERRED_SHUTDOWN => {
            kernel_messages::print_exit_phrase();
            unsafe {
                x86_64::instructions::port::Port::<u16>::new(0x604).write(0x2000);
            }
            x86_64::instructions::interrupts::disable();
            loop {
                x86_64::instructions::hlt();
            }
        }
        DEFERRED_REBOOT => {
            locale::render_event_auto(kernel_messages::KernelEvent::ShellReboot);
            let mut port = x86_64::instructions::port::Port::new(0x64);
            unsafe {
                port.write(0xfeu8);
            }
        }
        DEFERRED_APPS => {
            apps::activity::run_activity_manager();
            print!("> ");
        }
        _ => {}
    }
}

// ============================================================
// ПЕРЕРИСОВКА СТРОКИ ВВОДА — framebuffer-only
// Рисуем BUFFER[0..INDEX] на строке 24 начиная с колонки 2
// Подсветка выделения: Black on Yellow (0xE0)
// ============================================================
fn redraw_input() {
    // SAFETY: Инварианты INDEX ≤ 63, CURSOR ≤ INDEX поддерживаются всеми
    // писателями этих статиков; normal path рендерит только в framebuffer.
    unsafe {
        let _idx = SHELL.buffer_len;
        let _cur = SHELL.cursor;
        debug_assert!(_idx <= 63, "INPUT INDEX overflow: {}", _idx);
        debug_assert!(
            _cur <= _idx,
            "CURSOR past INDEX: cursor={} index={}",
            _cur,
            _idx
        );
        crate::fb_buffer::draw_shell_input_line(
            &SHELL.buffer[..SHELL.buffer_len],
            SHELL.cursor,
            SHELL.sel_active,
            SHELL.sel_start,
            SHELL.sel_end,
            SHELL.lang_rus,
        );
    }
}

// ============================================================
// УДАЛЕНИЕ ВЫДЕЛЕНИЯ
// ============================================================
fn delete_selection() {
    // SAFETY: чтение/запись глобальных статиков только из обработчика прерывания (однопоточно).
    unsafe {
        if !SHELL.sel_active {
            return;
        }
        let start = SHELL.sel_start;
        let end = SHELL.sel_end;
        let _idx2 = SHELL.buffer_len; // локальная копия для debug_assert (Rust 2024: no &static_mut)
        debug_assert!(
            start <= end,
            "invalid selection: start={} end={}",
            start,
            end
        );
        debug_assert!(
            end <= _idx2,
            "selection end past INDEX: end={} index={}",
            end,
            _idx2
        );
        let len = end - start;
        if len == 0 {
            SHELL.sel_active = false;
            return;
        }

        remove_buffer_range(start, len);
        SHELL.cursor = start;
        SHELL.sel_active = false;
    }
}

// ============================================================
// CLIPBOARD: copy / paste / cut / select_all
// ============================================================
fn do_copy() {
    unsafe {
        if SHELL.sel_active && SHELL.sel_end > SHELL.sel_start {
            let len = SHELL.sel_end - SHELL.sel_start;
            copy_buffer_range_to(&raw mut SHELL.clipboard, SHELL.sel_start, len);
            SHELL.clipboard_len = len;
        } else if SHELL.buffer_len > 0 {
            copy_buffer_range_to(&raw mut SHELL.clipboard, 0, SHELL.buffer_len);
            SHELL.clipboard_len = SHELL.buffer_len;
        }
    }
}

fn do_paste() {
    // SAFETY: однопоточный доступ из обработчика прерывания клавиатуры.
    unsafe {
        if SHELL.clipboard_len == 0 {
            return;
        }
        let _cl = SHELL.clipboard_len;
        let _idx3 = SHELL.buffer_len;
        let _cur3 = SHELL.cursor;
        debug_assert!(_cl <= 64, "CLIP_LEN overflow: {}", _cl);
        debug_assert!(_idx3 <= 63, "INDEX overflow: {}", _idx3);
        debug_assert!(
            _cur3 <= _idx3,
            "CURSOR past INDEX: cursor={} index={}",
            _cur3,
            _idx3
        );

        if SHELL.sel_active {
            delete_selection();
        }

        let space = 63 - SHELL.buffer_len;
        let paste_len = SHELL.clipboard_len.min(space);
        if paste_len == 0 {
            return;
        }

        shift_buffer_right_from_cursor(paste_len);
        for i in 0..paste_len {
            SHELL.buffer[SHELL.cursor + i] = SHELL.clipboard[i];
        }
        SHELL.buffer_len += paste_len;
        SHELL.cursor += paste_len;
        redraw_input();
    }
}

fn do_cut() {
    unsafe {
        if SHELL.sel_active && SHELL.sel_end > SHELL.sel_start {
            do_copy();
            delete_selection();
            redraw_input();
        }
    }
}

fn do_select_all() {
    unsafe {
        if SHELL.buffer_len > 0 {
            SHELL.sel_active = true;
            SHELL.sel_start = 0;
            SHELL.sel_end = SHELL.buffer_len;
            redraw_input();
        }
    }
}

fn copy_buffer_range_to(dst: *mut [u32; 64], start: usize, len: usize) {
    unsafe {
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!(SHELL.buffer[start]),
            (&mut *dst).as_mut_ptr(),
            len,
        );
    }
}

fn shift_buffer_right_from_cursor(count: usize) {
    unsafe {
        core::ptr::copy(
            core::ptr::addr_of!(SHELL.buffer[SHELL.cursor]),
            core::ptr::addr_of_mut!(SHELL.buffer[SHELL.cursor + count]),
            SHELL.buffer_len - SHELL.cursor,
        );
    }
}

fn remove_buffer_range(start: usize, len: usize) {
    unsafe {
        if len == 0 {
            return;
        }
        core::ptr::copy(
            core::ptr::addr_of!(SHELL.buffer[start + len]),
            core::ptr::addr_of_mut!(SHELL.buffer[start]),
            SHELL.buffer_len - start - len,
        );
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!(SHELL.buffer[SHELL.buffer_len - len]),
            0,
            len,
        );
        SHELL.buffer_len -= len;
    }
}

fn delete_char_before_cursor() {
    unsafe {
        if SHELL.cursor == 0 {
            return;
        }
        SHELL.cursor -= 1;
        remove_buffer_range(SHELL.cursor, 1);
    }
}

fn delete_char_at_cursor() {
    unsafe {
        if SHELL.cursor >= SHELL.buffer_len {
            return;
        }
        remove_buffer_range(SHELL.cursor, 1);
    }
}

fn insert_codepoint_at_cursor(cp: u32) {
    unsafe {
        shift_buffer_right_from_cursor(1);
        SHELL.buffer[SHELL.cursor] = cp;
        SHELL.buffer_len += 1;
        SHELL.cursor += 1;
    }
}

fn save_buffer_to_history_slot(slot: usize) {
    unsafe {
        let len = SHELL.buffer_len;
        for i in 0..len {
            SHELL.history[slot][i] = SHELL.buffer[i];
        }
        for i in len..64 {
            SHELL.history[slot][i] = 0;
        }
        SHELL.history_lens[slot] = len;
    }
}

fn restore_buffer_from_history_slot(slot: usize) {
    unsafe {
        let len = SHELL.history_lens[slot];
        for i in 0..len {
            SHELL.buffer[i] = core::ptr::read_volatile(&raw const SHELL.history[slot][i]);
        }
        for i in len..64 {
            SHELL.buffer[i] = 0;
        }
        SHELL.buffer_len = len;
        SHELL.cursor = len;
    }
}

// ============================================================
// ИСТОРИЯ КОМАНД
// ============================================================
fn push_history() {
    unsafe {
        if SHELL.buffer_len == 0 {
            return;
        }
        let idx = SHELL.history_idx;
        save_buffer_to_history_slot(idx);
        SHELL.history_idx = (idx + 1) % 32;
        if SHELL.history_count < 32 {
            SHELL.history_count += 1;
        }
        SHELL.history_nav = -1;
    }
}

fn history_up() {
    unsafe {
        if SHELL.history_count == 0 {
            return;
        }
        if SHELL.history_nav == -1 {
            // Сохраняем текущий ввод в history[32]
            save_buffer_to_history_slot(32);
            SHELL.cursor = SHELL.buffer_len;
            SHELL.history_nav = 0;
        } else {
            // Переходим к предыдущей команде в истории
            SHELL.history_nav += 1;
            if SHELL.history_nav as usize >= SHELL.history_count {
                // ничего не делаем
            }
            let ring_idx = (SHELL.history_idx + 32 - 1 - SHELL.history_nav as usize) % 32;
            restore_buffer_from_history_slot(ring_idx);
        }
        SHELL.sel_active = false;
        redraw_input();
    }
}

fn history_down() {
    unsafe {
        if SHELL.history_nav < 0 {
            return;
        }
        SHELL.history_nav -= 1;
        if SHELL.history_nav < 0 {
            // Восстанавливаем сохранённый ввод из history[32]
            restore_buffer_from_history_slot(32);
        } else {
            let ring_idx = (SHELL.history_idx + 32 - 1 - SHELL.history_nav as usize) % 32;
            restore_buffer_from_history_slot(ring_idx);
        }
        SHELL.sel_active = false;
        redraw_input();
    }
}

// ============================================================
// СКРОЛЛБЭК — PageUp/PageDown
// ============================================================
fn enter_scroll_mode() {
    unsafe {
        if SCROLL_MODE {
            return;
        }
        let total = crate::fb_buffer::scroll_total();
        if total == 0 {
            return;
        }
        // Только снимок экрана для восстановления по Esc.
        // В кольцо НЕ пишем — строки там уже есть от new_line().
        crate::fb_buffer::save_screen_snapshot();
        SCROLL_MODE = true;
        SCROLL_OFFSET = 0;
        crate::fb_buffer::show_scrollback(SCROLL_OFFSET);
    }
}

fn scroll_page_up() {
    unsafe {
        let total = crate::fb_buffer::scroll_total();
        if SCROLL_OFFSET + 12 < total {
            SCROLL_OFFSET += 12;
        } else if total > 0 {
            SCROLL_OFFSET = total - 1;
        }
        crate::fb_buffer::show_scrollback(SCROLL_OFFSET);
    }
}

fn scroll_page_down() {
    unsafe {
        if SCROLL_OFFSET >= 12 {
            SCROLL_OFFSET -= 12;
            crate::fb_buffer::show_scrollback(SCROLL_OFFSET);
        } else {
            exit_scroll_mode();
        }
    }
}

fn exit_scroll_mode() {
    unsafe {
        if !SCROLL_MODE {
            return;
        }
        SCROLL_MODE = false;
        // Восстанавливаем экран из снимка (не из scrollback — чтобы вернуть точный вид)
        crate::fb_buffer::restore_saved_screen();
        redraw_input();
    }
}

fn handle_fkey(slot: usize) {
    debug_assert!(slot < 12, "F-key slot out of range: {}", slot);
    unsafe {
        if ALT_HELD.load(Ordering::Acquire) {
            if let Some(s) = SHELL.recording_slot {
                if s == slot {
                    let len = SHELL.buffer_len.min(64);
                    copy_buffer_range_to(&raw mut SHELL.hotkeys[slot], 0, len);
                    core::ptr::write_volatile(&raw mut SHELL.hotkey_lens[slot], len);
                    SHELL.recording_slot = None;
                    locale::print_localized_line("", 0x0E);
                    print_fkey_message(slot, FKeyMessage::Saved(len));
                    reset_buffer();
                    print!("> ");
                    return;
                }
            }
            SHELL.recording_slot = Some(slot);
            reset_buffer();
            locale::print_localized_line("", 0x0E);
            print_fkey_message(slot, FKeyMessage::RecordPrompt);
            print!("F{}> ", slot + 1);
        } else {
            let rec = core::ptr::read_volatile(&raw const SHELL.recording_slot);
            if rec.is_some() {
                return;
            }
            let len = core::ptr::read_volatile(&raw const SHELL.hotkey_lens[slot]);
            if len == 0 {
                locale::print_localized_line("", 0x0E);
                print_fkey_message(slot, FKeyMessage::Empty);
                print!("> ");
                return;
            }
            for i in 0..len {
                let val = core::ptr::read_volatile(&raw const SHELL.hotkeys[slot][i]);
                SHELL.buffer[i] = val;
            }
            SHELL.buffer_len = len;
            SHELL.cursor = len;
            print!("[F{}] ", slot + 1);
            handle_keyboard_input('\n');
        }
    }
}

// ============================================================
// ОБРАБОТКА RawKey (CapsLock, ScrollLock, PauseBreak, стрелки, Home/End)
// ============================================================
pub fn handle_raw_key(key: pc_keyboard::KeyCode) {
    use pc_keyboard::KeyCode;
    unsafe {
        // Скроллбэк: только PgUp/PgDn, остальное — выход
        if SCROLL_MODE {
            match key {
                KeyCode::PageUp => {
                    scroll_page_up();
                    return;
                }
                KeyCode::PageDown => {
                    scroll_page_down();
                    return;
                }
                _ => {
                    exit_scroll_mode();
                    return;
                }
            }
        }

        match key {
            KeyCode::CapsLock => {
                if SHELL.buffer_len > 0 && !SHELL.confirm_pending {
                    SHELL.confirm_pending = true;
                    locale::print_localized_line("", 0x0E);
                    locale::render_event_auto(kernel_messages::KernelEvent::ShellConfirmPrompt);
                }
            }
            KeyCode::ScrollLock => {
                SHELL.lang_rus = !SHELL.lang_rus;
                redraw_input();
            }
            KeyCode::PauseBreak => {
                if SHELL.confirm_pending {
                    SHELL.confirm_pending = false;
                    locale::render_event_auto(kernel_messages::KernelEvent::ShellCanceled);
                    reset_buffer();
                    print!("> ");
                }
            }
            // ← → стрелки + Shift-выделение
            KeyCode::ArrowLeft => {
                if SHELL.cursor > 0 {
                    if SHIFT_HELD.load(Ordering::Acquire) {
                        if !SHELL.sel_active {
                            SHELL.sel_active = true;
                            SHELL.sel_start = SHELL.cursor - 1;
                            SHELL.sel_end = SHELL.cursor;
                        } else if SHELL.sel_start == SHELL.cursor {
                            SHELL.sel_start = SHELL.cursor - 1;
                        } else if SHELL.sel_end == SHELL.cursor {
                            SHELL.sel_end = SHELL.cursor - 1;
                            if SHELL.sel_start == SHELL.sel_end {
                                SHELL.sel_active = false;
                            }
                        }
                    } else {
                        SHELL.sel_active = false;
                    }
                    SHELL.cursor -= 1;
                    redraw_input();
                }
            }
            KeyCode::ArrowRight => {
                if SHELL.cursor < SHELL.buffer_len {
                    if SHIFT_HELD.load(Ordering::Acquire) {
                        if !SHELL.sel_active {
                            SHELL.sel_active = true;
                            SHELL.sel_start = SHELL.cursor;
                            SHELL.sel_end = SHELL.cursor + 1;
                        } else if SHELL.sel_end == SHELL.cursor {
                            SHELL.sel_end = SHELL.cursor + 1;
                        } else if SHELL.sel_start == SHELL.cursor {
                            SHELL.sel_start = SHELL.cursor + 1;
                            if SHELL.sel_start == SHELL.sel_end {
                                SHELL.sel_active = false;
                            }
                        }
                    } else {
                        SHELL.sel_active = false;
                    }
                    SHELL.cursor += 1;
                    redraw_input();
                }
            }
            KeyCode::Home => {
                if SHELL.cursor > 0 {
                    if SHIFT_HELD.load(Ordering::Acquire) {
                        if !SHELL.sel_active {
                            SHELL.sel_active = true;
                            SHELL.sel_start = 0;
                            SHELL.sel_end = SHELL.cursor;
                        } else {
                            SHELL.sel_start = 0;
                        }
                    } else {
                        SHELL.sel_active = false;
                    }
                    SHELL.cursor = 0;
                    redraw_input();
                }
            }
            KeyCode::End => {
                if SHELL.cursor < SHELL.buffer_len {
                    if SHIFT_HELD.load(Ordering::Acquire) {
                        if !SHELL.sel_active {
                            SHELL.sel_active = true;
                            SHELL.sel_start = SHELL.cursor;
                            SHELL.sel_end = SHELL.buffer_len;
                        } else {
                            SHELL.sel_end = SHELL.buffer_len;
                        }
                    } else {
                        SHELL.sel_active = false;
                    }
                    SHELL.cursor = SHELL.buffer_len;
                    redraw_input();
                }
            }
            // ↑↓ история команд
            KeyCode::ArrowUp => {
                history_up();
            }
            KeyCode::ArrowDown => {
                history_down();
            }
            // PageUp/PageDown — прокрутка экрана
            KeyCode::PageUp => {
                enter_scroll_mode();
            }
            KeyCode::PageDown => {} // вне scroll mode — ничего
            // Delete — удалить символ ПОД курсором
            KeyCode::Delete => {
                if SHELL.sel_active {
                    delete_selection();
                    HIST_NAV = -1;
                    redraw_input();
                } else if SHELL.cursor < SHELL.buffer_len {
                    delete_char_at_cursor();
                    HIST_NAV = -1;
                    redraw_input();
                }
            }
            // F1-F12: Alt+Fn = запись, Fn = выполнение
            KeyCode::F1 => handle_fkey(0),
            KeyCode::F2 => handle_fkey(1),
            KeyCode::F3 => handle_fkey(2),
            KeyCode::F4 => handle_fkey(3),
            KeyCode::F5 => handle_fkey(4),
            KeyCode::F6 => handle_fkey(5),
            KeyCode::F7 => handle_fkey(6),
            KeyCode::F8 => handle_fkey(7),
            KeyCode::F9 => handle_fkey(8),
            KeyCode::F10 => handle_fkey(9),
            KeyCode::F11 => handle_fkey(10),
            KeyCode::F12 => handle_fkey(11),
            _ => {}
        }
    }
}

// ============================================================
// СБРОС БУФЕРА + КУРСОР
// ============================================================
fn reset_buffer() {
    unsafe {
        SHELL.buffer_len = 0;
        SHELL.cursor = 0;
        SHELL.sel_active = false;
        for i in 0..64 {
            SHELL.buffer[i] = 0;
        }
    }
}

pub fn show_shell_prompt() {
    reset_buffer();
    print!("> ");
    redraw_input();
}

// ============================================================
// ГЛАВНАЯ ЛОГИКА — Unicode Intent Engine
// Ctrl+C/V/X/A, cursor-aware insert/delete, redraw_input
// ============================================================

pub fn handle_keyboard_input(c: char) {
    unsafe {
        // Выход из скроллбэка на любую клавишу
        if SCROLL_MODE {
            exit_scroll_mode();
            if c == '\x1B' {
                return;
            }
        }

        // === Ctrl+key: clipboard ===
        if CTRL_HELD.load(Ordering::Acquire) {
            match c {
                'c' => {
                    do_copy();
                    return;
                }
                'v' => {
                    do_paste();
                    return;
                }
                'x' => {
                    do_cut();
                    return;
                }
                'a' => {
                    do_select_all();
                    return;
                }
                'l' => {
                    crate::fb_buffer::clear_screen();
                    locale::draw_locale_badge();
                    print!("> ");
                    redraw_input();
                    return;
                }
                _ => {
                    return;
                }
            }
        }

        // === Escape: сброс буфера / отмена подтверждения ===
        if c == '\x1B' {
            if SHELL.confirm_pending {
                SHELL.confirm_pending = false;
                locale::render_event_auto(kernel_messages::KernelEvent::ShellCanceled);
            }
            reset_buffer();
            print!("> ");
            redraw_input();
            return;
        }

        if c == '\n' {
            if SHELL.confirm_pending {
                SHELL.confirm_pending = false;
            }
            SHELL.sel_active = false;
            locale::print_localized_line("", 0x0E);

            if SHELL.buffer_len > 0 {
                push_history();
                if handle_irq_guard_debug_command()
                    || handle_log_debug_command()
                    || handle_nhs_shell_command()
                {
                    reset_buffer();
                    print!("> ");
                    redraw_input();
                    return;
                }

                let intent = unicode::lookup_intent(&SHELL.buffer[..SHELL.buffer_len]);
                kernel_messages::dispatch_shell_intent(intent, &SHELL.buffer[..SHELL.buffer_len]);
            }

            reset_buffer();
            print!("> ");
            redraw_input();
        } else if c == '\x08' {
            // Backspace
            HIST_NAV = -1;
            if SHELL.confirm_pending {
                SHELL.confirm_pending = false;
                locale::render_event_auto(kernel_messages::KernelEvent::ShellCanceled);
                reset_buffer();
                print!("> ");
                redraw_input();
            } else if SHELL.sel_active {
                // Удаляем выделение
                delete_selection();
                redraw_input();
            } else if SHELL.cursor > 0 {
                delete_char_before_cursor();
                redraw_input();
            }
        } else if SHELL.buffer_len < 63 {
            if SHELL.confirm_pending {
                return;
            }
            HIST_NAV = -1;

            // Удаляем выделение если есть
            if SHELL.sel_active {
                delete_selection();
            }

            // Применяем языковую раскладку
            let mapped = locale::map_shell_input_char(c, SHELL.lang_rus);
            let cp = unicode::char_to_codepoint(mapped);
            insert_codepoint_at_cursor(cp);
            redraw_input();
        }
    }
}
