use core::arch::asm;

const TIMEOUT_TICKS: u64 = 30_000_000_000;

static mut START_TSC: u64 = 0;
static mut LAST_ACTIVITY_TSC: u64 = 0;

#[inline(always)]
fn rdtsc() -> u64 {
    let lo: u32;
    let hi: u32;
    unsafe {
        asm!(
            "rdtsc",
            out("eax") lo,
            out("edx") hi,
            options(nomem, nostack),
        );
    }
    ((hi as u64) << 32) | lo as u64
}

pub fn init() {
    unsafe {
        let now = rdtsc();
        START_TSC = now;
        LAST_ACTIVITY_TSC = now;
    }
}

pub fn pet() {
    unsafe {
        LAST_ACTIVITY_TSC = rdtsc();
    }
}

pub fn timed_out() -> bool {
    unsafe {
        let now = rdtsc();
        let elapsed = now.wrapping_sub(LAST_ACTIVITY_TSC);
        if elapsed > TIMEOUT_TICKS {
            crate::serial_println!(
                "[tribe/watchdog] timeout: elapsed={} total={}",
                elapsed,
                now.wrapping_sub(START_TSC)
            );
            return true;
        }
        false
    }
}
