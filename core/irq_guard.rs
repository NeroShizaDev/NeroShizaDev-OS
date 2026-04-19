use core::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

static IRQ_DEPTH: AtomicUsize = AtomicUsize::new(0);
static BLOCK_HEAVY_IN_IRQ: AtomicBool = AtomicBool::new(true);
static VIOLATION_COUNT: AtomicU64 = AtomicU64::new(0);
static FIRST_HIT_SEEN: AtomicBool = AtomicBool::new(false);
static FIRST_HIT_ALERT_PENDING: AtomicBool = AtomicBool::new(false);

pub struct IrqScope;

#[must_use]
pub fn enter_irq() -> IrqScope {
    IRQ_DEPTH.fetch_add(1, Ordering::AcqRel);
    IrqScope
}

impl Drop for IrqScope {
    fn drop(&mut self) {
        let mut current = IRQ_DEPTH.load(Ordering::Acquire);
        while current != 0 {
            match IRQ_DEPTH.compare_exchange_weak(
                current,
                current - 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return,
                Err(actual) => current = actual,
            }
        }
    }
}

pub fn is_in_irq() -> bool {
    IRQ_DEPTH.load(Ordering::Acquire) != 0
}

pub fn allow_heavy_operation() -> bool {
    if !is_in_irq() {
        return true;
    }

    VIOLATION_COUNT.fetch_add(1, Ordering::AcqRel);
    if !FIRST_HIT_SEEN.swap(true, Ordering::AcqRel) {
        FIRST_HIT_ALERT_PENDING.store(true, Ordering::Release);
    }
    !BLOCK_HEAVY_IN_IRQ.load(Ordering::Acquire)
}

pub fn take_first_hit_alert() -> bool {
    FIRST_HIT_ALERT_PENDING.swap(false, Ordering::AcqRel)
}

pub fn guard_enabled() -> bool {
    BLOCK_HEAVY_IN_IRQ.load(Ordering::Acquire)
}

pub fn set_guard_enabled(enabled: bool) {
    BLOCK_HEAVY_IN_IRQ.store(enabled, Ordering::Release);
}

pub fn violation_count() -> u64 {
    VIOLATION_COUNT.load(Ordering::Acquire)
}

pub fn reset_counters() {
    VIOLATION_COUNT.store(0, Ordering::Release);
}


