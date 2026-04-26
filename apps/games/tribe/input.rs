const KEYBUF_SIZE: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyEvent {
    Esc,
    Enter,
    Action(u8),
    History,
}

static mut KEYBUF: [KeyEvent; KEYBUF_SIZE] = [KeyEvent::Esc; KEYBUF_SIZE];
static mut HEAD: usize = 0;
static mut TAIL: usize = 0;

pub fn reset() {
    unsafe {
        HEAD = 0;
        TAIL = 0;
        crate::ps2::clear_scancode_queue();
    }
}

pub fn poll() {
    unsafe {
        while crate::ps2::has_scancode() {
            let sc = crate::ps2::read_scancode();
            if sc & 0x80 != 0 {
                continue;
            }

            if let Some(key) = map_scancode(sc) {
                push(key);
            }
        }
    }
}

pub fn pop_keydown() -> Option<KeyEvent> {
    unsafe {
        if HEAD == TAIL {
            return None;
        }

        let key = KEYBUF[TAIL];
        TAIL = (TAIL + 1) % KEYBUF_SIZE;
        Some(key)
    }
}

unsafe fn push(key: KeyEvent) {
    let next = (HEAD + 1) % KEYBUF_SIZE;
    if next == TAIL {
        TAIL = (TAIL + 1) % KEYBUF_SIZE;
    }
    KEYBUF[HEAD] = key;
    HEAD = next;
}

fn map_scancode(sc: u8) -> Option<KeyEvent> {
    match sc {
        0x01 => Some(KeyEvent::Esc),
        0x1C => Some(KeyEvent::Enter),
        0x23 => Some(KeyEvent::History),
        0x02 => Some(KeyEvent::Action(0)),
        0x03 => Some(KeyEvent::Action(1)),
        0x04 => Some(KeyEvent::Action(2)),
        0x05 => Some(KeyEvent::Action(3)),
        0x06 => Some(KeyEvent::Action(4)),
        _ => None,
    }
}
