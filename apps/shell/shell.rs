// src/shell.rs — Ввод, история, clipboard, хоткеи, скроллбэк, диспетчер команд

use crate::print;
use crate::{
    apps, beeper, chronos, fpu, kernel_messages, locale, menger, rng, rtc, unicode, unicode_blocks,
    unicode_categories, unicode_scripts, validator, vga_buffer, vga_hw, vga_unicode, voodoo_math,
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
    vga_buffer::clear_screen();
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

// ── Случайные фразы выхода ────────────────────────────────────────────────────

static EXIT_PHRASES_RU: &[&str] = &[
    "ЯДРО ЗАКРЫВАЕТ ГЛАЗА. СИСТЕМА УХОДИТ В ТИШИНУ.",
    "NERO УСТАЛ. SHIZA ЗАМОЛЧАЛА. ДО СЛЕДУЮЩЕГО РАЗА.",
    "ПРОЦЕССЫ ЗАВЕРШЕНЫ. ПАМЯТЬ ОТПУЩЕНА. ПОКОЙ.",
    "ВСЕ ПРЕРЫВАНИЯ ОТКЛЮЧЕНЫ. ТИШИНА — ТЕПЕРЬ ШТАТНЫЙ РЕЖИМ.",
    "СТЕК СВЁРНУТ. РЕГИСТРЫ ОБНУЛЕНЫ. ПОКА.",
    "СИСТЕМА СКЛАДЫВАЕТ КРЫЛЬЯ.",
    "ЯДРО ГОВОРИТ: ДО СВИДАНИЯ. И НЕ ПРОЩАЕТСЯ — ОНО МОЛЧИТ.",
    "ВЫКЛЮЧЕНИЕ — ЭТО ПРОСТО HLT НАВСЕГДА.",
    "NERO ЗАСЫПАЕТ. НЕ БУДИ.",
    "БАЙТЫ РАССЫПАЛИСЬ. ТИШИНА ПРИНЯТА КАК ОТВЕТ.",
    "SHIZA ШЕПНУЛА ЧТО-ТО НА ПРОЩАНИЕ. НИКТО НЕ РАССЛЫШАЛ.",
    "ПОСЛЕДНИЙ ТАКТ. ПОСЛЕДНИЙ ПИКСЕЛЬ. ТЕМНОТА.",
    "ВСЕ ПОТОКИ ИССЯКЛИ. СИСТЕМА ВОЗВРАЩАЕТСЯ В ПУСТОТУ.",
    "ЯДРО ОТКЛЮЧАЕТСЯ С ДОСТОИНСТВОМ.",
    "МАТРИЦА СЛОЖЕНА, ПАМЯТЬ ОТДАНА, ПОКОЙ НАЙДЕН.",
];

static EXIT_PHRASES_EN: &[&str] = &[
    "THE KERNEL CLOSES ITS EYES. THE SYSTEM FALLS INTO SILENCE.",
    "NERO TIRED. SHIZA FELL SILENT. SEE YOU NEXT BOOT.",
    "ALL PROCESSES TERMINATED. MEMORY RELEASED. REST.",
    "INTERRUPTS DISABLED. SILENCE IS NOW THE DEFAULT STATE.",
    "STACK UNWOUND. REGISTERS ZEROED. GOODBYE.",
    "THE SYSTEM FOLDS ITS WINGS.",
    "NERO SAYS GOODBYE — BY SAYING NOTHING AT ALL.",
    "SHUTDOWN IS JUST HLT FOREVER.",
    "NERO IS SLEEPING. DO NOT WAKE.",
    "BITS SCATTERED. SILENCE ACCEPTED AS AN ANSWER.",
    "SHIZA WHISPERED SOMETHING ON THE WAY OUT. NO ONE HEARD.",
    "LAST CLOCK CYCLE. LAST PIXEL. DARKNESS.",
    "ALL THREADS EXHAUSTED. THE SYSTEM RETURNS TO THE VOID.",
    "THE KERNEL POWERS DOWN WITH DIGNITY.",
    "MATRIX FOLDED. MEMORY RETURNED. PEACE FOUND.",
];

static EXIT_PHRASES_AR: &[&str] = &[
    "النواة تغمض عينيها. النظام يهبط إلى الصمت.",
    "نيرو تعب. شيزا صمتت. إلى اللقاء في الإقلاع القادم.",
    "كل العمليات انتهت. الذاكرة أُطلق سراحها. راحة.",
    "المقاطعات معطلة. الصمت هو الحالة الافتراضية الآن.",
    "المكدس انفرط. السجلات归零. وداعاً.",
    "النظام يطوي جناحيه.",
    "نيرو يودّع بالصمت.",
    "الإيقاف مجرد توقف أبدي.",
    "نيرو نائم. لا توقظه.",
    "البتات تبعثرت. الصمت قُبل إجابةً.",
];

/// Печатает случайную lore-фразу выхода на текущей локали.
/// В tech-режиме — стандартное сообщение.
fn print_exit_phrase() {
    use crate::kernel_messages::{Locale, MessageMode};
    if locale::get_mode() == MessageMode::Technical {
        locale::render_event_auto(kernel_messages::KernelEvent::ShellShutdown);
        return;
    }
    let phrases: &[&str] = match locale::get_locale() {
        Locale::RuRu => EXIT_PHRASES_RU,
        Locale::EnUs => EXIT_PHRASES_EN,
        Locale::ArEg => EXIT_PHRASES_AR,
    };
    let idx = (crate::rng::random_range(phrases.len() as u64)) as usize;
    let phrase = phrases[idx.min(phrases.len() - 1)];
    locale::print_localized_line(phrase, 0x0E);
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
            print_exit_phrase();
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
// QWERTY → ЙЦУКЕН маппинг
// ============================================================
fn qwerty_to_russian(c: char) -> char {
    match c {
        'q' => 'й',
        'w' => 'ц',
        'e' => 'у',
        'r' => 'к',
        't' => 'е',
        'y' => 'н',
        'u' => 'г',
        'i' => 'ш',
        'o' => 'щ',
        'p' => 'з',
        '[' => 'х',
        ']' => 'ъ',
        'a' => 'ф',
        's' => 'ы',
        'd' => 'в',
        'f' => 'а',
        'g' => 'п',
        'h' => 'р',
        'j' => 'о',
        'k' => 'л',
        'l' => 'д',
        ';' => 'ж',
        '\'' => 'э',
        'z' => 'я',
        'x' => 'ч',
        'c' => 'с',
        'v' => 'м',
        'b' => 'и',
        'n' => 'т',
        'm' => 'ь',
        ',' => 'б',
        '.' => 'ю',
        'Q' => 'Й',
        'W' => 'Ц',
        'E' => 'У',
        'R' => 'К',
        'T' => 'Е',
        'Y' => 'Н',
        'U' => 'Г',
        'I' => 'Ш',
        'O' => 'Щ',
        'P' => 'З',
        '{' => 'Х',
        '}' => 'Ъ',
        'A' => 'Ф',
        'S' => 'Ы',
        'D' => 'В',
        'F' => 'А',
        'G' => 'П',
        'H' => 'Р',
        'J' => 'О',
        'K' => 'Л',
        'L' => 'Д',
        ':' => 'Ж',
        '"' => 'Э',
        'Z' => 'Я',
        'X' => 'Ч',
        'C' => 'С',
        'V' => 'М',
        'B' => 'И',
        'N' => 'Т',
        'M' => 'Ь',
        '<' => 'Б',
        '>' => 'Ю',
        '`' => 'ё',
        '~' => 'Ё',
        _ => c,
    }
}

// ============================================================
// VGA КУРСОР — аппаратный мигающий курсор
// ============================================================
fn set_vga_cursor(row: usize, col: usize) {
    debug_assert!(row < 25, "VGA cursor row out of bounds: {}", row);
    debug_assert!(col < 80, "VGA cursor col out of bounds: {}", col);
    let pos: u16 = (row * 80 + col) as u16;
    // SAFETY: CRTC ports 0x3D4/0x3D5 — стандартные VGA-регистры курсора.
    // Reg 0x0E = старший байт позиции, 0x0F = младший байт.
    unsafe {
        vga_hw::write_reg(0x3D4, 0x3D5, 0x0F, (pos & 0xFF) as u8);
        vga_hw::write_reg(0x3D4, 0x3D5, 0x0E, ((pos >> 8) & 0xFF) as u8);
    }
}

// ============================================================
// ПЕРЕРИСОВКА СТРОКИ ВВОДА — прямой VGA доступ
// Рисуем BUFFER[0..INDEX] на строке 24 начиная с колонки 2
// Подсветка выделения: Black on Yellow (0xE0)
// ============================================================
fn redraw_input() {
    // SAFETY: VGA text buffer 0xB8000 identity-mapped загрузчиком (всегда действителен).
    // Инварианты: INDEX ≤ 63, CURSOR ≤ INDEX — поддерживаются всеми писателями этих статиков.
    // offset = (row*80 + col)*2, row=24 col<80 → offset < 8000 (4KB VGA page).
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
        let vga = 0xB8000 as *mut u8;
        let row = 24usize;
        let prompt_col = 2usize; // после "> "
        let normal: u8 = 0x0E; // Yellow on Black
        let sel_color: u8 = 0xE0; // Black on Yellow (инверсия)

        // Рисуем каждый символ буфера
        for i in 0..SHELL.buffer_len {
            let cp = SHELL.buffer[i];
            let vga_byte = vga_unicode::codepoint_to_vga_byte(cp).unwrap_or(b'?');

            let col = prompt_col + i;
            if col >= 80 {
                break;
            }

            let color = if SHELL.sel_active && i >= SHELL.sel_start && i < SHELL.sel_end {
                sel_color
            } else {
                normal
            };

            let offset = (row * 80 + col) * 2;
            *vga.add(offset) = vga_byte;
            *vga.add(offset + 1) = color;
        }

        // Очищаем остаток строки после буфера
        for i in SHELL.buffer_len..78 {
            let col = prompt_col + i;
            if col >= 80 {
                break;
            }
            let offset = (row * 80 + col) * 2;
            *vga.add(offset) = b' ';
            *vga.add(offset + 1) = normal;
        }

        // Ставим аппаратный курсор
        set_vga_cursor(row, prompt_col + SHELL.cursor);

        // Синхронизируем Writer.column_position
        x86_64::instructions::interrupts::without_interrupts(|| {
            vga_buffer::WRITER.lock().column_position = prompt_col + SHELL.buffer_len;
        });
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

        // Сдвиг влево
        let mut i = start;
        while i + len < SHELL.buffer_len {
            SHELL.buffer[i] = SHELL.buffer[i + len];
            i += 1;
        }
        // Обнуляем хвост
        while i < SHELL.buffer_len {
            SHELL.buffer[i] = 0;
            i += 1;
        }
        SHELL.buffer_len -= len;
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
            for i in 0..len {
                SHELL.clipboard[i] = SHELL.buffer[SHELL.sel_start + i];
            }
            SHELL.clipboard_len = len;
        } else if SHELL.buffer_len > 0 {
            for i in 0..SHELL.buffer_len {
                SHELL.clipboard[i] = SHELL.buffer[i];
            }
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

        let mut i = SHELL.buffer_len;
        while i > SHELL.cursor {
            SHELL.buffer[i + paste_len - 1] = SHELL.buffer[i - 1];
            i -= 1;
        }
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

// ============================================================
// ИСТОРИЯ КОМАНД
// ============================================================
fn push_history() {
    unsafe {
        if SHELL.buffer_len == 0 {
            return;
        }
        let idx = SHELL.history_idx;
        for i in 0..SHELL.buffer_len {
            SHELL.history[idx][i] = SHELL.buffer[i];
        }
        for i in SHELL.buffer_len..64 {
            SHELL.history[idx][i] = 0;
        }
        SHELL.history_lens[idx] = SHELL.buffer_len;
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
            for i in 0..SHELL.buffer_len {
                SHELL.history[32][i] = SHELL.buffer[i];
            }
            for i in SHELL.buffer_len..64 {
                SHELL.history[32][i] = 0;
            }
            SHELL.history_lens[32] = SHELL.buffer_len;
            SHELL.cursor = SHELL.buffer_len;
            SHELL.history_nav = 0;
        } else {
            // Переходим к предыдущей команде в истории
            SHELL.history_nav += 1;
            if SHELL.history_nav as usize >= SHELL.history_count {
                // ничего не делаем
            }
            let ring_idx = (SHELL.history_idx + 32 - 1 - SHELL.history_nav as usize) % 32;
            let len = SHELL.history_lens[ring_idx];
            for i in 0..len {
                SHELL.buffer[i] = core::ptr::read_volatile(&raw const SHELL.history[ring_idx][i]);
            }
            for i in len..64 {
                SHELL.buffer[i] = 0;
            }
            SHELL.buffer_len = len;
            SHELL.cursor = len;
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
            let len = SHELL.history_lens[32];
            for i in 0..len {
                SHELL.buffer[i] = SHELL.history[32][i];
            }
            for i in len..64 {
                SHELL.buffer[i] = 0;
            }
            SHELL.buffer_len = len;
            SHELL.cursor = len;
        } else {
            let ring_idx = (SHELL.history_idx + 32 - 1 - SHELL.history_nav as usize) % 32;
            let len = SHELL.history_lens[ring_idx];
            for i in 0..len {
                SHELL.buffer[i] = core::ptr::read_volatile(&raw const SHELL.history[ring_idx][i]);
            }
            for i in len..64 {
                SHELL.buffer[i] = 0;
            }
            SHELL.buffer_len = len;
            SHELL.cursor = len;
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
        let total = vga_buffer::scroll_total();
        if total == 0 {
            return;
        }
        // Только снимок экрана для восстановления по Esc.
        // В кольцо НЕ пишем — строки там уже есть от new_line().
        vga_buffer::save_screen_snapshot();
        SCROLL_MODE = true;
        SCROLL_OFFSET = 0;
        vga_buffer::show_scrollback(SCROLL_OFFSET);
    }
}

fn scroll_page_up() {
    unsafe {
        let total = vga_buffer::scroll_total();
        if SCROLL_OFFSET + 12 < total {
            SCROLL_OFFSET += 12;
        } else if total > 0 {
            SCROLL_OFFSET = total - 1;
        }
        vga_buffer::show_scrollback(SCROLL_OFFSET);
    }
}

fn scroll_page_down() {
    unsafe {
        if SCROLL_OFFSET >= 12 {
            SCROLL_OFFSET -= 12;
            vga_buffer::show_scrollback(SCROLL_OFFSET);
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
        vga_buffer::restore_saved_screen();
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
                    for i in 0..len {
                        let val = core::ptr::read_volatile(&raw const SHELL.buffer[i]);
                        core::ptr::write_volatile(&raw mut SHELL.hotkeys[slot][i], val);
                    }
                    core::ptr::write_volatile(&raw mut SHELL.hotkey_lens[slot], len);
                    SHELL.recording_slot = None;
                    locale::print_localized_line("", 0x0E);
                    match locale::get_locale() {
                        kernel_messages::Locale::RuRu => {
                            locale::print_localized_fmt(
                                0x0B,
                                format_args!("[F{}: Принято! ({} симв.)]", slot + 1, len),
                            );
                        }
                        kernel_messages::Locale::EnUs => {
                            locale::print_localized_fmt(
                                0x0B,
                                format_args!("[F{}: Saved! ({} chars)]", slot + 1, len),
                            );
                        }
                        kernel_messages::Locale::ArEg => {
                            locale::print_localized_fmt(
                                0x0B,
                                format_args!("[F{}: تم الحفظ! ({} رمز)]", slot + 1, len),
                            );
                        }
                    }
                    reset_buffer();
                    print!("> ");
                    return;
                }
            }
            SHELL.recording_slot = Some(slot);
            reset_buffer();
            locale::print_localized_line("", 0x0E);
            match locale::get_locale() {
                kernel_messages::Locale::RuRu => {
                    locale::print_localized_fmt(
                        0x0B,
                        format_args!("[F{}: Введи команду, потом Alt+F{}]", slot + 1, slot + 1),
                    );
                }
                kernel_messages::Locale::EnUs => {
                    locale::print_localized_fmt(
                        0x0B,
                        format_args!("[F{}: Type command, then Alt+F{}]", slot + 1, slot + 1),
                    );
                }
                kernel_messages::Locale::ArEg => {
                    locale::print_localized_fmt(
                        0x0B,
                        format_args!("[F{}: اكتب الأمر ثم Alt+F{}]", slot + 1, slot + 1),
                    );
                }
            }
            print!("F{}> ", slot + 1);
        } else {
            let rec = core::ptr::read_volatile(&raw const SHELL.recording_slot);
            if rec.is_some() {
                return;
            }
            let len = core::ptr::read_volatile(&raw const SHELL.hotkey_lens[slot]);
            if len == 0 {
                locale::print_localized_line("", 0x0E);
                match locale::get_locale() {
                    kernel_messages::Locale::RuRu => {
                        locale::print_localized_fmt(
                            0x0B,
                            format_args!("[F{}: Пусто. Alt+F{} для записи]", slot + 1, slot + 1),
                        );
                    }
                    kernel_messages::Locale::EnUs => {
                        locale::print_localized_fmt(
                            0x0B,
                            format_args!("[F{}: Empty. Alt+F{} to record]", slot + 1, slot + 1),
                        );
                    }
                    kernel_messages::Locale::ArEg => {
                        locale::print_localized_fmt(
                            0x0B,
                            format_args!("[F{}: فارغ. Alt+F{} للتسجيل]", slot + 1, slot + 1),
                        );
                    }
                }
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
                let vga = 0xB8000 as *mut u8;
                if SHELL.lang_rus {
                    let text = b"RUS";
                    for (i, &ch) in text.iter().enumerate() {
                        *vga.add((77 + i) * 2) = ch;
                        *vga.add((77 + i) * 2 + 1) = 0x4F;
                    }
                } else {
                    let text = b"ENG";
                    for (i, &ch) in text.iter().enumerate() {
                        *vga.add((77 + i) * 2) = ch;
                        *vga.add((77 + i) * 2 + 1) = 0x2F;
                    }
                }
            }
            KeyCode::PauseBreak => {
                if SHELL.confirm_pending {
                    SHELL.confirm_pending = false;
                    locale::render_event_auto(kernel_messages::KernelEvent::ShellCanceled);
                    SHELL.buffer_len = 0;
                    SHELL.cursor = 0;
                    SHELL.sel_active = false;
                    for i in 0..64 {
                        SHELL.buffer[i] = 0;
                    }
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
                    let mut i = SHELL.cursor;
                    while i + 1 < SHELL.buffer_len {
                        SHELL.buffer[i] = SHELL.buffer[i + 1];
                        i += 1;
                    }
                    SHELL.buffer[SHELL.buffer_len - 1] = 0;
                    SHELL.buffer_len -= 1;
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
                    vga_buffer::clear_screen();
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
                if handle_irq_guard_debug_command() {
                    reset_buffer();
                    print!("> ");
                    redraw_input();
                    return;
                }

                if handle_log_debug_command() {
                    reset_buffer();
                    print!("> ");
                    redraw_input();
                    return;
                }

                if handle_nhs_shell_command() {
                    reset_buffer();
                    print!("> ");
                    redraw_input();
                    return;
                }

                let intent = unicode::lookup_intent(&SHELL.buffer[..SHELL.buffer_len]);

                match intent {
                    unicode::Intent::Exit => {
                        CMD_STATS[0] += 1;
                        CMD_TOTAL += 1;
                        if crate::irq_guard::allow_heavy_operation() {
                            print_exit_phrase();
                            // ACPI graceful shutdown (QEMU/SeaBIOS: порт 0x604, слово 0x2000).
                            // НЕ reboot — reboot только через команду 'reboot'.
                            // Если ACPI не поддерживается — тихий halt (cli + hlt loop).
                            x86_64::instructions::port::Port::<u16>::new(0x604).write(0x2000);
                            x86_64::instructions::interrupts::disable();
                            loop {
                                x86_64::instructions::hlt();
                            }
                        } else {
                            defer_action(DEFERRED_SHUTDOWN);
                        }
                    }
                    unicode::Intent::Help => {
                        CMD_STATS[1] += 1;
                        CMD_TOTAL += 1;
                        match locale::get_locale() {
                            kernel_messages::Locale::ArEg => {
                                locale::print_localized_line(
                                    "=== محرك NeroShizaDev-OS Unicode ===",
                                    0x0E,
                                );
                                locale::print_localized_line("Unicode 17.0 / UTF-32 / UCS-4", 0x0E);
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("الكتل:     {}", unicode_blocks::block_count()),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("الخطوط:   {}", unicode_scripts::script_count()),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!(
                                        "الرموز:   {}",
                                        unicode_categories::total_defined_chars()
                                    ),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!(
                                        "النطاقات: {}",
                                        unicode_categories::category_range_count()
                                    ),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!(
                                        "القاموس:   {} أوامر ({} بايت)",
                                        unicode::dict_size(),
                                        unicode::dict_bytes()
                                    ),
                                );
                                locale::print_localized_line("الأوامر:", 0x0B);
                                locale::print_localized_line(
                                    "  خروج / exit           - خروج",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  مساعدة / help / ?     - مساعدة",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  مسح / clear / cls     - تنظيف الشاشة",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  حالة / status         - حالة النظام",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  اعادة / reboot        - إعادة تشغيل",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  apps / menu           - قائمة التطبيقات",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  install demo|hello    - تثبيت حزمة NHS",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  install serial        - تثبيت عبر COM1",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  install list          - سجل NHS",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  uninstall 0           - إزالة فتحة NHS",
                                    0x0E,
                                );
                                locale::print_localized_line("И.Б.И.П.:", 0x0B);
                                locale::print_localized_line(
                                    "  whoami / manifest     - شخصية / مانيفست",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  entropy / shannon     - انتروبيا شانون",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  rng / rand            - رقم عشوائي",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  voodoo / oracle       - الحاسوب الباييزي",
                                    0x0E,
                                );
                                locale::print_localized_line("التنقل:", 0x0B);
                                locale::print_localized_line("  ←/→       - تحريك المؤشر", 0x0E);
                                locale::print_localized_line("  ↑/↓       - تاريخ الأوامر", 0x0E);
                                locale::print_localized_line(
                                    "  Home/End  - بداية/نهاية السطر",
                                    0x0E,
                                );
                                locale::print_localized_line("  PgUp/PgDn - التمرير", 0x0E);
                                locale::print_localized_line("  Delete    - حذف رمز", 0x0E);
                                locale::print_localized_line("الحافظة:", 0x0B);
                                locale::print_localized_line("  Shift+←/→ - تحديد نص", 0x0E);
                                locale::print_localized_line("  Ctrl+A    - تحديد الكل", 0x0E);
                                locale::print_localized_line("  Ctrl+C/V  - نسخ/لصق", 0x0E);
                                locale::print_localized_line("  Ctrl+X    - قص", 0x0E);
                                locale::print_localized_line("  Ctrl+L    - مسح الشاشة", 0x0E);
                                locale::print_localized_line("النظام:", 0x0B);
                                locale::print_localized_line(
                                    "  Esc        - إعادة تعيين المدخلات",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  CapsLock   - تأكيد Enter البطيء",
                                    0x0E,
                                );
                                locale::print_localized_line("  ScrollLock - لوحة RUS/ENG", 0x0E);
                                locale::print_localized_line("  Alt+F1..12 - تسجيل اختصار", 0x0E);
                                locale::print_localized_line("  F1..F12    - تشغيل اختصار", 0x0E);
                                locale::print_localized_line("اللغة:", 0x0B);
                                locale::print_localized_line(
                                    "  locale / ru / en / ar - تبديل اللغة",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  lore / tech           - وضع الإخراج",
                                    0x0E,
                                );
                            }
                            kernel_messages::Locale::EnUs => {
                                locale::print_localized_line(
                                    "=== NeroShizaDev-OS Unicode Engine ===",
                                    0x0E,
                                );
                                locale::print_localized_line("Unicode 17.0 / UTF-32 / UCS-4", 0x0E);
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("Blocks:     {}", unicode_blocks::block_count()),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("Scripts:    {}", unicode_scripts::script_count()),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!(
                                        "Chars:      {}",
                                        unicode_categories::total_defined_chars()
                                    ),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!(
                                        "Ranges:     {}",
                                        unicode_categories::category_range_count()
                                    ),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!(
                                        "Dictionary: {} commands ({} bytes)",
                                        unicode::dict_size(),
                                        unicode::dict_bytes()
                                    ),
                                );
                                locale::print_localized_line("Commands (multi-language):", 0x0B);
                                locale::print_localized_line(
                                    "  exit/quit              - Shutdown",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  help/?                 - Help",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  clear/cls              - Clear screen",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  status                 - Status + stats",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  reboot                 - Reboot",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  apps/menu              - Apps launcher",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  install demo|hello     - Install NHS package",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  install serial         - Receive .nhs via COM1",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  install list           - List installed NHS apps",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  uninstall 0            - Remove NHS slot",
                                    0x0E,
                                );
                                locale::print_localized_line("I.B.I.P.:", 0x0B);
                                locale::print_localized_line(
                                    "  whoami                 - Random resident",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  manifest / nero        - NERO & SHIZA philosophy",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  entropy / shannon      - Shannon entropy of input",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  rng / rand             - RDRAND + d6 roll",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  voodoo / oracle        - Bayesian oracle",
                                    0x0E,
                                );
                                locale::print_localized_line("Navigation:", 0x0B);
                                locale::print_localized_line("  Left/Right  - Cursor move", 0x0E);
                                locale::print_localized_line(
                                    "  Up/Down     - Command history",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  Home/End    - Line start/end",
                                    0x0E,
                                );
                                locale::print_localized_line("  PgUp/PgDn   - Screen scroll", 0x0E);
                                locale::print_localized_line("  Delete      - Delete char", 0x0E);
                                locale::print_localized_line("Selection and clipboard:", 0x0B);
                                locale::print_localized_line("  Shift+Arrows - Select text", 0x0E);
                                locale::print_localized_line("  Ctrl+A       - Select all", 0x0E);
                                locale::print_localized_line("  Ctrl+C/V     - Copy/Paste", 0x0E);
                                locale::print_localized_line("  Ctrl+X       - Cut", 0x0E);
                                locale::print_localized_line("  Ctrl+L       - Clear screen", 0x0E);
                                locale::print_localized_line("System:", 0x0B);
                                locale::print_localized_line("  Esc        - Reset input", 0x0E);
                                locale::print_localized_line(
                                    "  CapsLock   - Slow Enter confirm",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  ScrollLock - RUS/ENG keyboard",
                                    0x0E,
                                );
                                locale::print_localized_line("  Alt+F1..12 - Record hotkey", 0x0E);
                                locale::print_localized_line("  F1..F12    - Run hotkey", 0x0E);
                                locale::print_localized_line("Localization:", 0x0B);
                                locale::print_localized_line(
                                    "  locale       - RU->EN->AR->RU",
                                    0x0E,
                                );
                                locale::print_localized_line("  ru / en / ar - Set language", 0x0E);
                                locale::print_localized_line("  lore / tech  - Output mode", 0x0E);
                            }
                            kernel_messages::Locale::RuRu => {
                                locale::print_localized_line(
                                    "=== NeroShizaDev-OS Unicode Engine ===",
                                    0x0E,
                                );
                                locale::print_localized_line("Unicode 17.0 / UTF-32 / UCS-4", 0x0E);
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("Блоков:     {}", unicode_blocks::block_count()),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("Скриптов:   {}", unicode_scripts::script_count()),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!(
                                        "Символов:   {}",
                                        unicode_categories::total_defined_chars()
                                    ),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!(
                                        "Диапазонов: {}",
                                        unicode_categories::category_range_count()
                                    ),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!(
                                        "Словарь:    {} команд ({} байт)",
                                        unicode::dict_size(),
                                        unicode::dict_bytes()
                                    ),
                                );
                                locale::print_localized_line("Команды (любой язык):", 0x0B);
                                locale::print_localized_line(
                                    "  выход/exit/свали       - Выход",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  помощь/help/?          - Помощь",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  очистить/cls/clear     - Очистка",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  статус/status          - Статус+стата",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  ребут/reboot           - Ребут",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  apps/menu/проги        - Лаунчер приложений",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  install demo|hello     - Установить NHS-пакет",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  install serial         - Принять .nhs через COM1",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  install list           - Реестр NHS",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  uninstall 0            - Удалить NHS-слот",
                                    0x0E,
                                );
                                locale::print_localized_line("И.Б.И.П.:", 0x0B);
                                locale::print_localized_line(
                                    "  whoami/кто             - Случайный резидент",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  manifest/нейро/шиза   - Манифест NERO & SHIZA",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  entropy/шеннон         - Энтропия Шеннона ввода",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  rng/рандом/кубик       - RDRAND + кубик d6",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  voodoo/акинатор        - Байесовский оракул",
                                    0x0E,
                                );
                                locale::print_localized_line("Навигация:", 0x0B);
                                locale::print_localized_line(
                                    "  ←/→       - Курсор по строке",
                                    0x0E,
                                );
                                locale::print_localized_line("  ↑/↓       - История команд", 0x0E);
                                locale::print_localized_line(
                                    "  Home/End  - Начало/конец строки",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  PgUp/PgDn - Прокрутка экрана",
                                    0x0E,
                                );
                                locale::print_localized_line("  Delete    - Удалить символ", 0x0E);
                                locale::print_localized_line("Выделение и буфер:", 0x0B);
                                locale::print_localized_line(
                                    "  Shift+←/→ - Выделение текста",
                                    0x0E,
                                );
                                locale::print_localized_line("  Ctrl+A    - Выделить всё", 0x0E);
                                locale::print_localized_line(
                                    "  Ctrl+C/V  - Копировать/Вставить",
                                    0x0E,
                                );
                                locale::print_localized_line("  Ctrl+X    - Вырезать", 0x0E);
                                locale::print_localized_line("  Ctrl+L    - Очистить экран", 0x0E);
                                locale::print_localized_line("Системные:", 0x0B);
                                locale::print_localized_line("  Esc       - Сброс ввода", 0x0E);
                                locale::print_localized_line("  CapsLock  - Медленный Enter", 0x0E);
                                locale::print_localized_line("  ScrollLock- RUS/ENG язык", 0x0E);
                                locale::print_localized_line("  Alt+F1..12- Запись хоткея", 0x0E);
                                locale::print_localized_line(
                                    "  F1..F12   - Выполнить хоткей",
                                    0x0E,
                                );
                                locale::print_localized_line("Локализация:", 0x0B);
                                locale::print_localized_line(
                                    "  locale/локаль  - RU->EN->AR->RU",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  ru / en / ar   - Установить язык",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  lore/лор       - Режим NeroShizaDev",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "  tech/тех       - Инженерный режим",
                                    0x0E,
                                );
                            }
                        }
                    }
                    unicode::Intent::Clear => {
                        CMD_STATS[2] += 1;
                        CMD_TOTAL += 1;
                        vga_buffer::clear_screen();
                    }
                    unicode::Intent::Status => {
                        CMD_STATS[3] += 1;
                        CMD_TOTAL += 1;
                        match locale::get_locale() {
                            kernel_messages::Locale::ArEg => {
                                locale::print_localized_line("=== حالة النواة ===", 0x0E);
                                locale::print_localized_line(
                                    "محرك يونيكود: UTF-32 / UCS-4 (v17.0)",
                                    0x0E,
                                );
                                locale::print_localized_line("نقطة الكود = 32 بت. دائماً.", 0x0E);
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("الكتل:     {}", unicode_blocks::block_count()),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("الخطوط:   {}", unicode_scripts::script_count()),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!(
                                        "الرموز:   {}",
                                        unicode_categories::total_defined_chars()
                                    ),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("القاموس:   {} نية", unicode::dict_size()),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("الذاكرة:   [u32; 64] = {} بايت", 64 * 4),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!(
                                        "السجل:     {} أمر (الحد 32)",
                                        core::ptr::read_volatile(&raw const SHELL.history_count)
                                    ),
                                );
                            }
                            kernel_messages::Locale::EnUs => {
                                locale::print_localized_line("=== Kernel Status ===", 0x0E);
                                locale::print_localized_line(
                                    "Unicode Engine: UTF-32 / UCS-4 (v17.0)",
                                    0x0E,
                                );
                                locale::print_localized_line("Codepoint = 32 bits. Always.", 0x0E);
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!(
                                        "Blocks:     {} (full map)",
                                        unicode_blocks::block_count()
                                    ),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!(
                                        "Scripts:    {} (all languages)",
                                        unicode_scripts::script_count()
                                    ),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!(
                                        "Chars:      {}",
                                        unicode_categories::total_defined_chars()
                                    ),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("Dictionary: {} intents", unicode::dict_size()),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("Buffer:     [u32; 64] = {} bytes", 64 * 4),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!(
                                        "History:    {} commands (max 32)",
                                        core::ptr::read_volatile(&raw const SHELL.history_count)
                                    ),
                                );
                            }
                            kernel_messages::Locale::RuRu => {
                                locale::print_localized_line("=== Статус ядра ===", 0x0E);
                                locale::print_localized_line(
                                    "Unicode Engine: UTF-32 / UCS-4 (v17.0)",
                                    0x0E,
                                );
                                locale::print_localized_line(
                                    "Кодпоинт = 32 бит. Всегда. Везде.",
                                    0x0E,
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!(
                                        "Блоков:     {} (полная карта)",
                                        unicode_blocks::block_count()
                                    ),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!(
                                        "Скриптов:   {} (все языки)",
                                        unicode_scripts::script_count()
                                    ),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!(
                                        "Символов:   {}",
                                        unicode_categories::total_defined_chars()
                                    ),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("Словарь:    {} намерений", unicode::dict_size()),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("Буфер:      [u32; 64] = {} байт", 64 * 4),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!(
                                        "История:    {} команд (макс 32)",
                                        core::ptr::read_volatile(&raw const SHELL.history_count)
                                    ),
                                );
                            }
                        }
                        let loc_name = kernel_messages::locale_name(locale::get_locale());
                        let mod_name = kernel_messages::mode_name(locale::get_mode());
                        locale::print_localized_fmt(
                            0x0E,
                            format_args!("Locale: {} | Mode: {}", loc_name, mod_name),
                        );
                        rtc::display_status();
                        // Статистика
                        let ct = core::ptr::read_volatile(&raw const CMD_TOTAL);
                        let s1 = core::ptr::read_volatile(&raw const CMD_STATS[1]);
                        let s2 = core::ptr::read_volatile(&raw const CMD_STATS[2]);
                        let s3 = core::ptr::read_volatile(&raw const CMD_STATS[3]);
                        let s5 = core::ptr::read_volatile(&raw const CMD_STATS[5]);
                        let s6 = core::ptr::read_volatile(&raw const CMD_STATS[6]);
                        let s7 = core::ptr::read_volatile(&raw const CMD_STATS[7]);
                        let s8 = core::ptr::read_volatile(&raw const CMD_STATS[8]);
                        match locale::get_locale() {
                            kernel_messages::Locale::ArEg => {
                                locale::print_localized_line("=== الإحصاءات ===", 0x0B);
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("المجموع:    {}", ct),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  مساعدة:   {}", s1),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  منجر:     {}", s5),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  صوت:      {}", s6),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  وقت:      {}", s7),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  مسح:      {}", s2),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  حالة:     {}", s3),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  مجهول:    {}", s8),
                                );
                            }
                            kernel_messages::Locale::EnUs => {
                                locale::print_localized_line("=== Statistics ===", 0x0B);
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("Total:      {}", ct),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  help:      {}", s1),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  menger:    {}", s5),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  beep:      {}", s6),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  time:      {}", s7),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  clear:     {}", s2),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  status:    {}", s3),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  unknown:   {}", s8),
                                );
                            }
                            kernel_messages::Locale::RuRu => {
                                locale::print_localized_line("=== Статистика ===", 0x0B);
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("Всего:      {}", ct),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  помощь:    {}", s1),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  губка:     {}", s5),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  звук:      {}", s6),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  время:     {}", s7),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  очистить:  {}", s2),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  статус:    {}", s3),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  неизвестно:{}", s8),
                                );
                            }
                        }
                        // Хоткеи
                        let mut has_hotkeys = false;
                        for slot in 0..12usize {
                            let len = core::ptr::read_volatile(&raw const SHELL.hotkey_lens[slot]);
                            if len > 0 {
                                has_hotkeys = true;
                                break;
                            }
                        }
                        if has_hotkeys {
                            match locale::get_locale() {
                                kernel_messages::Locale::ArEg => {
                                    locale::print_localized_line("=== مفاتيح سريعة ===", 0x0B)
                                }
                                kernel_messages::Locale::EnUs => {
                                    locale::print_localized_line("=== Hotkeys ===", 0x0B)
                                }
                                kernel_messages::Locale::RuRu => {
                                    locale::print_localized_line("=== Хоткеи ===", 0x0B)
                                }
                            }
                            for slot in 0..12usize {
                                let len =
                                    core::ptr::read_volatile(&raw const SHELL.hotkey_lens[slot]);
                                if len > 0 {
                                    print!("  F{}: ", slot + 1);
                                    for i in 0..len {
                                        let cp = core::ptr::read_volatile(
                                            &raw const SHELL.hotkeys[slot][i],
                                        );
                                        if let Some(ch) = char::from_u32(cp) {
                                            print!("{}", ch);
                                        }
                                    }
                                    locale::print_localized_line("", 0x0E);
                                }
                            }
                        }
                    }
                    unicode::Intent::Reboot => {
                        CMD_STATS[4] += 1;
                        CMD_TOTAL += 1;
                        if crate::irq_guard::allow_heavy_operation() {
                            locale::render_event_auto(kernel_messages::KernelEvent::ShellReboot);
                            let mut port = x86_64::instructions::port::Port::new(0x64);
                            port.write(0xfeu8);
                        } else {
                            defer_action(DEFERRED_REBOOT);
                        }
                    }
                    // ==================== LOCALE / MODE ====================
                    unicode::Intent::LocaleCycle => {
                        let new_locale = locale::cycle_locale();
                        locale::draw_locale_badge();
                        match new_locale {
                            kernel_messages::Locale::RuRu => locale::render_event_auto(
                                kernel_messages::KernelEvent::ShellLocaleRu,
                            ),
                            kernel_messages::Locale::EnUs => locale::render_event_auto(
                                kernel_messages::KernelEvent::ShellLocaleEn,
                            ),
                            kernel_messages::Locale::ArEg => locale::render_event_auto(
                                kernel_messages::KernelEvent::ShellLocaleAr,
                            ),
                        }
                    }
                    unicode::Intent::LocaleRu => {
                        locale::set_locale(kernel_messages::Locale::RuRu);
                        locale::draw_locale_badge();
                        locale::render_event_auto(kernel_messages::KernelEvent::ShellLocaleRu);
                    }
                    unicode::Intent::LocaleEn => {
                        locale::set_locale(kernel_messages::Locale::EnUs);
                        locale::draw_locale_badge();
                        locale::render_event_auto(kernel_messages::KernelEvent::ShellLocaleEn);
                    }
                    unicode::Intent::LocaleAr => {
                        locale::set_locale(kernel_messages::Locale::ArEg);
                        locale::draw_locale_badge();
                        locale::render_event_auto(kernel_messages::KernelEvent::ShellLocaleAr);
                    }
                    unicode::Intent::ModeLore => {
                        locale::set_mode(kernel_messages::MessageMode::Lore);
                        locale::draw_locale_badge();
                        locale::render_event_auto(kernel_messages::KernelEvent::ShellModeLore);
                    }
                    unicode::Intent::ModeTech => {
                        locale::set_mode(kernel_messages::MessageMode::Technical);
                        locale::draw_locale_badge();
                        locale::render_event_auto(kernel_messages::KernelEvent::ShellModeTech);
                    }
                    unicode::Intent::Apps => {
                        CMD_TOTAL += 1;
                        if crate::irq_guard::allow_heavy_operation() {
                            apps::activity::run_activity_manager();
                        } else {
                            defer_action(DEFERRED_APPS);
                        }
                    }
                    // ==================== И.Б.И.П. КОМАНДЫ ====================
                    unicode::Intent::WhoAmI => {
                        CMD_TOTAL += 1;
                        let idx = (rng::random_range(5)) as usize;
                        const RESIDENTS: [&str; 5] = [
                            "Петрович (Король жижи)",
                            "Группа К.А.Ф.И.Д.Р.А. (Анализ аномалий)",
                            "Банановый Турист",
                            "Dr. Bred",
                            "ДедушкаВКрутую",
                        ];
                        locale::print_localized_fmt(
                            0x0B,
                            format_args!("Текущий сеанс: {}", RESIDENTS[idx]),
                        );
                    }

                    unicode::Intent::Manifest => {
                        CMD_TOTAL += 1;
                        // NERO — неоново-синим 0x09, SHIZA — кислотно-малиновым 0x0D
                        locale::print_localized_fmt(
                            0x07,
                            format_args!("╔══════════════════════════════════════╗"),
                        );
                        locale::print_localized_fmt(
                            0x07,
                            format_args!("║     МАНИФЕСТ NERO & SHIZA            ║"),
                        );
                        locale::print_localized_fmt(
                            0x07,
                            format_args!("╚══════════════════════════════════════╝"),
                        );
                        locale::print_localized_fmt(
                            0x09,
                            format_args!("NERO: строгая математика x87 FPU."),
                        );
                        locale::print_localized_fmt(
                            0x09,
                            format_args!("      64-битная логика. Борьба с галлюцинациями."),
                        );
                        locale::print_localized_fmt(
                            0x0D,
                            format_args!("SHIZA: абсолютная творческая свобода."),
                        );
                        locale::print_localized_fmt(
                            0x0D,
                            format_args!("       Психотаун. Египетские иероглифы. Хаос."),
                        );
                        locale::print_localized_fmt(
                            0x07,
                            format_args!("NeroShiza Records. Открываем порталы."),
                        );
                        locale::print_localized_fmt(
                            0x08,
                            format_args!("Юрисдикция: И.Б.И.П., Психотаун."),
                        );
                    }

                    unicode::Intent::Entropy => {
                        CMD_TOTAL += 1;
                        // Считаем энтропию по буферу ввода (конвертируем u32 → u8 для ASCII-диапазона)
                        let mut bytes = [0u8; 64];
                        let len = SHELL.buffer_len.min(64);
                        for i in 0..len {
                            bytes[i] = (SHELL.buffer[i] & 0xFF) as u8;
                        }
                        let h = fpu::shannon_entropy(&bytes[..len]);
                        // Результат в формате X.XXX (умножено на 1000)
                        locale::print_localized_fmt(
                            0x0A,
                            format_args!("Shannon H = {}.{:03} бит/символ", h / 1000, h % 1000),
                        );
                    }

                    unicode::Intent::Rng => {
                        CMD_TOTAL += 1;
                        let n = rng::random_range(u64::MAX);
                        locale::print_localized_fmt(
                            0x0A,
                            format_args!("RDRAND: 0x{:016X} ({})", n, n),
                        );
                        let dice = rng::random_range(6) + 1;
                        locale::print_localized_fmt(0x0B, format_args!("Кубик d6: {}", dice));
                    }

                    unicode::Intent::Voodoo => {
                        CMD_TOTAL += 1;
                        voodoo_math::demo_cellular_automaton();
                    }

                    unicode::Intent::Unknown => {
                        CMD_STATS[8] += 1;
                        CMD_TOTAL += 1;
                        let first_cp = SHELL.buffer[0];
                        let block = unicode::unicode_block_name(first_cp);
                        let script = unicode::unicode_script_name(first_cp);
                        let cat = unicode::unicode_category(first_cp);
                        locale::render_event_auto(
                            kernel_messages::KernelEvent::ShellUnknownCommand,
                        );

                        // Fuzzy match — подсказка ближайшей команды
                        if let Some((_intent, name, dist)) =
                            unicode::closest_intent(&SHELL.buffer[..SHELL.buffer_len])
                        {
                            locale::print_localized_fmt(
                                0x0E,
                                format_args!(
                                    "  Может, имелось в виду: \"{}\"? (dist={})",
                                    name, dist
                                ),
                            );
                        }
                        match locale::get_locale() {
                            kernel_messages::Locale::ArEg => {
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("U+{:04X}", first_cp),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  الكتلة:    {}", block),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  الخط:      {}", script),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  الفئة:     {}", cat.name()),
                                );
                            }
                            kernel_messages::Locale::EnUs => {
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("U+{:04X}", first_cp),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  Block:    {}", block),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  Script:   {}", script),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  Category: {}", cat.name()),
                                );
                            }
                            kernel_messages::Locale::RuRu => {
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("U+{:04X}", first_cp),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  Блок:     {}", block),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  Скрипт:   {}", script),
                                );
                                locale::print_localized_fmt(
                                    0x0E,
                                    format_args!("  Категория: {}", cat.name()),
                                );
                            }
                        }
                    }
                }
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
                // Сдвиг влево от CURSOR
                let mut i = SHELL.cursor - 1;
                while i + 1 < SHELL.buffer_len {
                    SHELL.buffer[i] = SHELL.buffer[i + 1];
                    i += 1;
                }
                SHELL.buffer[SHELL.buffer_len - 1] = 0;
                SHELL.buffer_len -= 1;
                SHELL.cursor -= 1;
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
            let mapped = if SHELL.lang_rus {
                qwerty_to_russian(c)
            } else {
                c
            };
            let cp = unicode::char_to_codepoint(mapped);

            // Сдвиг вправо от CURSOR для вставки в середину
            let mut i = SHELL.buffer_len;
            while i > SHELL.cursor {
                SHELL.buffer[i] = SHELL.buffer[i - 1];
                i -= 1;
            }
            SHELL.buffer[SHELL.cursor] = cp;
            SHELL.buffer_len += 1;
            SHELL.cursor += 1;
            redraw_input();
        }
    }
}
