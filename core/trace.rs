use x86_64::instructions::interrupts;
use core::sync::atomic::{AtomicBool, Ordering};

const TRACE_DEPTH: usize = 8;

static mut TRACE_ENTRIES: [Option<&'static str>; TRACE_DEPTH] = [None; TRACE_DEPTH];
static mut TRACE_HEAD: usize = 0;
static mut TRACE_LEN: usize = 0;
static TRACE_FROZEN: AtomicBool = AtomicBool::new(false);

#[inline]
fn oldest_index(len: usize, head: usize) -> usize {
    if len == TRACE_DEPTH { head } else { 0 }
}

unsafe fn push_entry(action: &'static str) {
    TRACE_ENTRIES[TRACE_HEAD] = Some(action);
    TRACE_HEAD = (TRACE_HEAD + 1) % TRACE_DEPTH;
    if TRACE_LEN < TRACE_DEPTH {
        TRACE_LEN += 1;
    }
}

pub fn record(action: &'static str) {
    if TRACE_FROZEN.load(Ordering::Acquire) {
        return;
    }
    interrupts::without_interrupts(|| unsafe {
        if TRACE_FROZEN.load(Ordering::Relaxed) {
            return;
        }
        push_entry(action);
    });
}

pub fn record_fatal(action: &'static str) {
    interrupts::without_interrupts(|| unsafe {
        // Первый fatal фиксирует последнюю метку и замораживает trace.
        if !TRACE_FROZEN.swap(true, Ordering::AcqRel) {
            push_entry(action);
        }
    });
}

pub fn freeze() {
    TRACE_FROZEN.store(true, Ordering::Release);
}

pub fn len() -> usize {
    interrupts::without_interrupts(|| unsafe { TRACE_LEN })
}

pub fn get_recent(index_from_oldest: usize) -> Option<&'static str> {
    interrupts::without_interrupts(|| unsafe {
        if index_from_oldest >= TRACE_LEN {
            return None;
        }

        let oldest = oldest_index(TRACE_LEN, TRACE_HEAD);
        let slot = (oldest + index_from_oldest) % TRACE_DEPTH;
        TRACE_ENTRIES[slot]
    })
}

pub fn contains_recent(action: &'static str) -> bool {
    interrupts::without_interrupts(|| unsafe {
        let oldest = oldest_index(TRACE_LEN, TRACE_HEAD);

        for i in 0..TRACE_LEN {
            let slot = (oldest + i) % TRACE_DEPTH;
            if TRACE_ENTRIES[slot] == Some(action) {
                return true;
            }
        }
        false
    })
}