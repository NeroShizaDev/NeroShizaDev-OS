use super::state::{CMD_TOTAL, SHELL};
use core::sync::atomic::{AtomicU8, Ordering};

static mut SERIAL_MON_TICK: u32 = 0;
static mut SERIAL_MON_EVERY: u32 = 600;
static mut LAST_IRQ_VIOLATIONS: u64 = 0;
static mut LAST_OOM_COUNT: usize = 0;

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

fn show_irq_guard_first_hit_alert() {
    let msg = match crate::locale::get_locale() {
        crate::kernel_messages::Locale::RuRu => {
            "[IRQGUARD] Опасный вызов из IRQ заблокирован и отложен."
        }
        crate::kernel_messages::Locale::EnUs => {
            "[IRQGUARD] Heavy call from IRQ blocked and deferred."
        }
        crate::kernel_messages::Locale::ArEg => "[IRQGUARD] تم حظر نداء ثقيل من IRQ وتأجيله.",
    };
    let hits = crate::irq_guard::violation_count();

    crate::serial_println!(
        "[IRQGUARD][WARN] First hit captured: heavy operation from IRQ was blocked/deferred (violations={}).",
        hits
    );

    unsafe {
        crate::locale::write_str_at_vga(
            "                                                                                ",
            23,
            0,
            0x07,
        );
        crate::locale::write_str_at_vga(msg, 23, 0, 0x0E);
    }
}

fn emit_shell_serial_monitor() {
    unsafe {
        SERIAL_MON_TICK = SERIAL_MON_TICK.wrapping_add(1);
        if SERIAL_MON_TICK < SERIAL_MON_EVERY {
            return;
        }
        SERIAL_MON_TICK = 0;

        let act = crate::apps::activity::debug_stats();
        let app = crate::apps::activity::app_kind_name(act.current);
        let trace_depth = crate::trace::len();
        let irq_hits = crate::irq_guard::violation_count();

        let heap_used = crate::apps::games::doom::stubs::heap_used_bytes();
        let heap_total = crate::apps::games::doom::stubs::heap_total_bytes();
        let heap_peak = crate::apps::games::doom::stubs::heap_peak_used_bytes();
        let heap_oom = crate::apps::games::doom::stubs::heap_oom_count();
        let last_oom_req = crate::apps::games::doom::stubs::heap_last_oom_request();
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
            crate::kernel_messages::print_exit_phrase();
            unsafe {
                x86_64::instructions::port::Port::<u16>::new(0x604).write(0x2000);
            }
            x86_64::instructions::interrupts::disable();
            loop {
                x86_64::instructions::hlt();
            }
        }
        DEFERRED_REBOOT => {
            crate::locale::render_event_auto(crate::kernel_messages::KernelEvent::ShellReboot);
            let mut port = x86_64::instructions::port::Port::new(0x64);
            unsafe {
                port.write(0xfeu8);
            }
        }
        DEFERRED_APPS => {
            crate::apps::activity::run_activity_manager();
            crate::print!("> ");
        }
        _ => {}
    }
}
