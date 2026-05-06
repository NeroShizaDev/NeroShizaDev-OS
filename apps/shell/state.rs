use crate::fb_buffer;
use core::sync::atomic::AtomicBool;

pub struct ShellState {
    pub buffer: [u32; 64],
    pub buffer_len: usize,
    pub cursor: usize,

    pub clipboard: [u32; 64],
    pub clipboard_len: usize,

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

pub(super) static mut SHELL: ShellState = ShellState {
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

pub static ALT_HELD: AtomicBool = AtomicBool::new(false);
pub static CTRL_HELD: AtomicBool = AtomicBool::new(false);
pub static SHIFT_HELD: AtomicBool = AtomicBool::new(false);
pub static mut SCROLL_MODE: bool = false;
pub static mut SCROLL_OFFSET: usize = 0;
pub static mut HIST_NAV: isize = -1;
pub static mut CMD_TOTAL: u32 = 0;
pub static mut CMD_STATS: [u32; 9] = [0; 9];

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

pub(super) fn shell_buffer_eq_ascii(expected: &str) -> bool {
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

pub(super) fn shell_buffer_to_ascii_lower(out: &mut [u8; 64]) -> Option<usize> {
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

pub(super) fn shell_buffer_snapshot(out: &mut [u32; 64]) -> usize {
    unsafe {
        let len = SHELL.buffer_len;
        for i in 0..len {
            out[i] = SHELL.buffer[i];
        }
        len
    }
}

pub(super) fn redraw_input() {
    unsafe {
        let input_len = SHELL.buffer_len;
        let cursor = SHELL.cursor;
        debug_assert!(input_len <= 63, "INPUT INDEX overflow: {}", input_len);
        debug_assert!(
            cursor <= input_len,
            "CURSOR past INDEX: cursor={} index={}",
            cursor,
            input_len
        );
        fb_buffer::draw_shell_input_line(
            &SHELL.buffer[..SHELL.buffer_len],
            SHELL.cursor,
            SHELL.sel_active,
            SHELL.sel_start,
            SHELL.sel_end,
            SHELL.lang_rus,
        );
    }
}

pub(super) fn reset_buffer() {
    unsafe {
        SHELL.buffer_len = 0;
        SHELL.cursor = 0;
        SHELL.sel_active = false;
        for i in 0..64 {
            SHELL.buffer[i] = 0;
        }
    }
}
