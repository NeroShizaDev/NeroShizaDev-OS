/// NeroShizaDev-OS — Activity Manager
///
/// Architecture: Nintendo Switch Horizon Applet Manager × Android Back-Stack
/// on bare-metal x86 Rust, zero heap, zero compositor overhead.
///
/// Lifecycle per activity:
///
///   Push  → on_start()  → update() loop
///                              ↓ Push(new)
///                         on_pause()  →  new.on_start() → new.update()
///                                              ↓ Pop
///                         on_resume() ← (redraws full screen)
///                              ↓ Pop / Replace
///                         on_destroy()
///
/// The Activity Manager IS the main loop. Shell is the bottom of the stack.

use x86_64::instructions::hlt;

// ── Activity intent ───────────────────────────────────────────────────────────

pub enum ActivityIntent {
    /// Stay alive, nothing to do this tick.
    Continue,
    /// Esc / Back — pop me off the stack.
    Pop,
    /// Open a new activity on top; I go to on_pause().
    Push(AppKind),
    /// Close me and open another without leaving a back-entry.
    Replace(AppKind),
}

// ── Activity flags (bitmask, stored in stack slot) ────────────────────────────

pub struct ActivityFlags(pub u8);

impl ActivityFlags {
    /// Default: activity is saved in back-stack.
    pub const NORMAL:       ActivityFlags = ActivityFlags(0x00);
    /// Like Android FLAG_ACTIVITY_NO_HISTORY: Pop after first Push on top.
    pub const NO_HISTORY:   ActivityFlags = ActivityFlags(0x01);
    /// Like SINGLE_TOP: if already on top, don't push duplicate.
    pub const SINGLE_TOP:   ActivityFlags = ActivityFlags(0x02);
    /// Clear entire stack before pushing (like FLAG_ACTIVITY_CLEAR_TOP).
    pub const CLEAR_TOP:    ActivityFlags = ActivityFlags(0x04);

    pub fn has(&self, f: &ActivityFlags) -> bool { self.0 & f.0 != 0 }
}

// ── Intent extras — tiny data passed between activities ───────────────────────

/// Fixed 32-byte payload, no heap. Like Android Intent extras but honest.
#[derive(Clone, Copy)]
pub struct IntentExtras {
    pub tag:  [u8; 4],      // 4-char type tag, e.g. b"GAME"
    pub data: [u8; 28],     // opaque payload
}

impl IntentExtras {
    pub const EMPTY: Self = Self { tag: *b"NONE", data: [0u8; 28] };
}

// ── App kind — every activity is one of these ────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum AppKind {
    Launcher = 0,
    Games    = 1,
    Doom     = 2,
    Jackal   = 3,
    Menger   = 4,
    Voodoo   = 5,
    Chronos  = 6,
    Rtc      = 7,
    Rng      = 8,
    Beeper   = 9,
    Fpu      = 10,
    Locale   = 11,
    /// Back to the interrupt-driven shell (hlt_loop).
    Shell    = 12,
}

// ── Stack slot ────────────────────────────────────────────────────────────────

struct Slot {
    kind:   AppKind,
    flags:  ActivityFlags,
    extras: IntentExtras,
}

// ── ActivityStack — static, no heap, depth 16 ────────────────────────────────

const MAX_DEPTH: usize = 16;

struct ActivityStack {
    slots: [Option<Slot>; MAX_DEPTH],
    len:   usize,
}

impl ActivityStack {
    const fn new() -> Self {
        // Option<Slot> is not Copy, so we can't use array repeat syntax.
        // Use MaybeUninit-free trick: init with None manually.
        Self {
            slots: [
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
            ],
            len: 0,
        }
    }

    fn push(&mut self, kind: AppKind, flags: ActivityFlags, extras: IntentExtras) {
        if self.len >= MAX_DEPTH { return; }
        self.slots[self.len] = Some(Slot { kind, flags, extras });
        self.len += 1;
    }

    fn pop(&mut self) -> Option<Slot> {
        if self.len == 0 { return None; }
        self.len -= 1;
        self.slots[self.len].take()
    }

    fn top(&self) -> Option<&Slot> {
        if self.len == 0 { None } else { self.slots[self.len - 1].as_ref() }
    }

    fn top_kind(&self) -> Option<AppKind> {
        self.top().map(|s| s.kind)
    }

    fn is_empty(&self) -> bool { self.len == 0 }

    fn depth(&self) -> usize { self.len }

    /// CLEAR_TOP: drain down to (not including) the bottom launcher.
    fn clear_all_but_bottom(&mut self) {
        while self.len > 1 {
            if let Some(slot) = self.pop() {
                dispatch_lifecycle(slot.kind, Lifecycle::Destroy);
            }
        }
    }
}

// ── Lifecycle dispatch ────────────────────────────────────────────────────────

enum Lifecycle { Start, Pause, Resume, Destroy }

fn dispatch_lifecycle(kind: AppKind, ev: Lifecycle) {
    match (kind, ev) {
        (AppKind::Launcher, Lifecycle::Start)   => crate::apps::launcher::on_start(),
        (AppKind::Launcher, Lifecycle::Resume)  => crate::apps::launcher::on_resume(),
        (AppKind::Launcher, Lifecycle::Pause)   => crate::apps::launcher::on_pause(),
        (AppKind::Launcher, Lifecycle::Destroy) => {}

        (AppKind::Games,  Lifecycle::Start)   => crate::apps::games::on_start(),
        (AppKind::Games,  Lifecycle::Resume)  => crate::apps::games::on_resume(),
        (AppKind::Games,  Lifecycle::Pause)   => {}
        (AppKind::Games,  Lifecycle::Destroy) => {}

        (AppKind::Doom,   Lifecycle::Start)   => { crate::doom::init(&[]); }
        (AppKind::Doom,   Lifecycle::Resume)  => {}   // Doom redraws on first update()
        (AppKind::Doom,   Lifecycle::Pause)   => {}
        (AppKind::Doom,   Lifecycle::Destroy) => crate::doom::on_destroy(),

        (AppKind::Jackal, Lifecycle::Start)   => {}
        (AppKind::Jackal, Lifecycle::Resume)  => {}
        (AppKind::Jackal, Lifecycle::Pause)   => {}
        (AppKind::Jackal, Lifecycle::Destroy) => {}

        (AppKind::Menger,  Lifecycle::Start)   => {}
        (AppKind::Menger,  Lifecycle::Resume)  => {}
        (AppKind::Menger,  Lifecycle::Pause)   => {}
        (AppKind::Menger,  Lifecycle::Destroy) => {}

        (AppKind::Voodoo,  Lifecycle::Start)   => {}
        (AppKind::Voodoo,  Lifecycle::Resume)  => {}
        (AppKind::Voodoo,  Lifecycle::Pause)   => {}
        (AppKind::Voodoo,  Lifecycle::Destroy) => {}

        (AppKind::Chronos, Lifecycle::Start)   => {}
        (AppKind::Chronos, Lifecycle::Resume)  => {}
        (AppKind::Chronos, Lifecycle::Pause)   => {}
        (AppKind::Chronos, Lifecycle::Destroy) => {}

        (AppKind::Rtc,     Lifecycle::Start)   => {}
        (AppKind::Rtc,     Lifecycle::Resume)  => {}
        (AppKind::Rtc,     Lifecycle::Pause)   => {}
        (AppKind::Rtc,     Lifecycle::Destroy) => {}

        (AppKind::Rng,     Lifecycle::Start)   => {}
        (AppKind::Rng,     Lifecycle::Resume)  => {}
        (AppKind::Rng,     Lifecycle::Pause)   => {}
        (AppKind::Rng,     Lifecycle::Destroy) => {}

        (AppKind::Beeper,  Lifecycle::Start)   => {}
        (AppKind::Beeper,  Lifecycle::Resume)  => {}
        (AppKind::Beeper,  Lifecycle::Pause)   => {}
        (AppKind::Beeper,  Lifecycle::Destroy) => {}

        (AppKind::Fpu,     Lifecycle::Start)   => {}
        (AppKind::Fpu,     Lifecycle::Resume)  => {}
        (AppKind::Fpu,     Lifecycle::Pause)   => {}
        (AppKind::Fpu,     Lifecycle::Destroy) => {}

        (AppKind::Locale,  Lifecycle::Start)   => crate::apps::locale_switcher::on_start(),
        (AppKind::Locale,  Lifecycle::Resume)  => crate::apps::locale_switcher::on_resume(),
        (AppKind::Locale,  Lifecycle::Pause)   => crate::apps::locale_switcher::on_pause(),
        (AppKind::Locale,  Lifecycle::Destroy) => {}

        // Shell: stateless from lifecycle perspective.
        // on_start re-draws the shell prompt so the screen looks right.
        (AppKind::Shell, Lifecycle::Start) => {
            crate::vga_buffer::clear_screen();
            crate::print!("> ");
        }
        (AppKind::Shell, Lifecycle::Resume)  => {
            crate::vga_buffer::clear_screen();
            crate::print!("> ");
        }
        (AppKind::Shell, Lifecycle::Pause)   => {}
        (AppKind::Shell, Lifecycle::Destroy) => {}
    }
}

/// Run the activity until it returns a non-Continue intent.
fn dispatch_update(kind: AppKind, depth: usize) -> ActivityIntent {
    match kind {
        AppKind::Launcher => crate::apps::launcher::update(depth),
        AppKind::Games    => { crate::apps::games::run();                            ActivityIntent::Pop }
        AppKind::Doom     => { crate::doom::run();                                   ActivityIntent::Pop }
        AppKind::Jackal   => { crate::apps::jackal::shell::run_demo();               ActivityIntent::Pop }
        AppKind::Menger   => { crate::menger::run_demo();                            ActivityIntent::Pop }
        AppKind::Voodoo   => { crate::voodoo_math::demo_cellular_automaton();        ActivityIntent::Pop }
        AppKind::Chronos  => { crate::chronos::display_triple_time();               ActivityIntent::Pop }
        AppKind::Rtc      => { crate::rtc::display_status();                         ActivityIntent::Pop }
        AppKind::Rng      => { crate::rng::demo();                                   ActivityIntent::Pop }
        AppKind::Beeper   => { crate::beeper::demo_hex_scale();                      ActivityIntent::Pop }
        AppKind::Fpu      => { crate::fpu::demo();                                   ActivityIntent::Pop }
        AppKind::Locale   => crate::apps::locale_switcher::update(depth),
        // Shell: pop immediately — control returns to hlt_loop which drives the shell via ISR.
        AppKind::Shell    => ActivityIntent::Pop,
    }
}

// ── Main entry point (called by shell command `apps`) ────────────────────────

pub fn run_activity_manager() {
    let _guard = InputGuard::new();

    let mut stack = ActivityStack::new();
    stack.push(AppKind::Launcher, ActivityFlags::NORMAL, IntentExtras::EMPTY);
    dispatch_lifecycle(AppKind::Launcher, Lifecycle::Start);

    loop {
        let depth = stack.depth();
        let kind  = match stack.top_kind() { Some(k) => k, None => break };

        let intent = dispatch_update(kind, depth);

        match intent {
            ActivityIntent::Continue => { hlt(); }

            ActivityIntent::Pop => {
                dispatch_lifecycle(kind, Lifecycle::Destroy);
                stack.pop();
                if stack.is_empty() { break; }

                // Resume whatever is now on top
                let resumed = stack.top_kind().unwrap();
                dispatch_lifecycle(resumed, Lifecycle::Resume);
            }

            ActivityIntent::Push(next) => {
                // SINGLE_TOP: if next == top, do nothing
                if next == kind {
                    if let Some(s) = stack.top() {
                        if s.flags.has(&ActivityFlags::SINGLE_TOP) { continue; }
                    }
                }
                dispatch_lifecycle(kind, Lifecycle::Pause);
                stack.push(next, ActivityFlags::NORMAL, IntentExtras::EMPTY);
                dispatch_lifecycle(next, Lifecycle::Start);
            }

            ActivityIntent::Replace(next) => {
                dispatch_lifecycle(kind, Lifecycle::Destroy);
                stack.pop();
                stack.push(next, ActivityFlags::NORMAL, IntentExtras::EMPTY);
                dispatch_lifecycle(next, Lifecycle::Start);
            }
        }
    }

    // Back in shell territory
    crate::vga_buffer::clear_screen();
    crate::locale::draw_locale_badge();
}

// ── Input capture guard ───────────────────────────────────────────────────────
// Always routes PS/2 to the current top-of-stack activity.
// On drop → returns input ownership to shell.

struct InputGuard;

impl InputGuard {
    fn new() -> Self {
        crate::ps2::set_input_owner(crate::ps2::InputOwner::Apps);
        crate::ps2::clear_scancode_queue();
        Self
    }
}

impl Drop for InputGuard {
    fn drop(&mut self) {
        crate::ps2::set_input_owner(crate::ps2::InputOwner::Shell);
        crate::ps2::clear_scancode_queue();
    }
}

// ── Public utility ────────────────────────────────────────────────────────────

pub fn wait_key() {
    unsafe {
        loop {
            if crate::ps2::has_scancode() {
                let sc = crate::ps2::read_scancode();
                if sc & 0x80 == 0 { break; }
            }
            hlt();
        }
    }
}
