use x86_64::instructions::interrupts;

const TRACE_DEPTH: usize = 8;

static mut TRACE_ENTRIES: [Option<&'static str>; TRACE_DEPTH] = [None; TRACE_DEPTH];
static mut TRACE_HEAD: usize = 0;
static mut TRACE_LEN: usize = 0;

pub fn record(action: &'static str) {
    interrupts::without_interrupts(|| unsafe {
        TRACE_ENTRIES[TRACE_HEAD] = Some(action);
        TRACE_HEAD = (TRACE_HEAD + 1) % TRACE_DEPTH;
        if TRACE_LEN < TRACE_DEPTH {
            TRACE_LEN += 1;
        }
    });
}

pub fn len() -> usize {
    interrupts::without_interrupts(|| unsafe { TRACE_LEN })
}

pub fn get_recent(index_from_oldest: usize) -> Option<&'static str> {
    interrupts::without_interrupts(|| unsafe {
        if index_from_oldest >= TRACE_LEN {
            return None;
        }

        let oldest = if TRACE_LEN == TRACE_DEPTH {
            TRACE_HEAD
        } else {
            0
        };
        let slot = (oldest + index_from_oldest) % TRACE_DEPTH;
        TRACE_ENTRIES[slot]
    })
}

pub fn contains_recent(action: &'static str) -> bool {
    interrupts::without_interrupts(|| unsafe {
        let oldest = if TRACE_LEN == TRACE_DEPTH {
            TRACE_HEAD
        } else {
            0
        };

        for i in 0..TRACE_LEN {
            let slot = (oldest + i) % TRACE_DEPTH;
            if TRACE_ENTRIES[slot] == Some(action) {
                return true;
            }
        }
        false
    })
}