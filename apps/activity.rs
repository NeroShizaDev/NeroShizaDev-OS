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

// ── Activity flags — reserved for future use (not yet assigned on push) ──────
// ActivityFlags and IntentExtras removed: no implementation existed.
// AppKind::Shell removed: Shell exit is now handled via Pop from Launcher.

// ── App kind — every activity is one of these ────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum AppKind {
    Launcher = 0,
    Games = 1,
    Calculator = 2,
    Jackal = 3,
    Menger = 4,
    Voodoo = 5,
    Chronos = 6,
    Rtc = 7,
    Rng = 8,
    Beeper = 9,
    Fpu = 10,
    Locale = 11,
    /// NHS installed app — run NeroShizaScript from slot LAUNCH_NHS_SLOT.
    Nhs = 12,
}

// ── Stack slot ────────────────────────────────────────────────────────────────

struct Slot {
    kind: AppKind,
}

// ── ActivityStack — static, no heap, depth 16 ────────────────────────────────

const MAX_DEPTH: usize = 16;

struct ActivityStack {
    slots: [Option<Slot>; MAX_DEPTH],
    len: usize,
}

impl ActivityStack {
    const fn new() -> Self {
        // Option<Slot> is not Copy, so we can't use array repeat syntax.
        // Use MaybeUninit-free trick: init with None manually.
        Self {
            slots: [
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
            len: 0,
        }
    }

    fn push(&mut self, kind: AppKind) {
        if self.len >= MAX_DEPTH {
            return;
        }
        self.slots[self.len] = Some(Slot { kind });
        self.len += 1;
    }

    fn pop(&mut self) -> Option<Slot> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        self.slots[self.len].take()
    }

    fn top(&self) -> Option<&Slot> {
        if self.len == 0 {
            None
        } else {
            self.slots[self.len - 1].as_ref()
        }
    }

    fn top_kind(&self) -> Option<AppKind> {
        self.top().map(|s| s.kind)
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn depth(&self) -> usize {
        self.len
    }
}

// ── Lifecycle dispatch ────────────────────────────────────────────────────────

enum Lifecycle {
    Start,
    Pause,
    Resume,
    Destroy,
}

fn dispatch_lifecycle(kind: AppKind, ev: Lifecycle) {
    match (kind, ev) {
        (AppKind::Launcher, Lifecycle::Start) => crate::apps::launcher::on_start(),
        (AppKind::Launcher, Lifecycle::Resume) => crate::apps::launcher::on_resume(),
        (AppKind::Launcher, Lifecycle::Pause) => crate::apps::launcher::on_pause(),
        (AppKind::Launcher, Lifecycle::Destroy) => {}

        (AppKind::Games, Lifecycle::Start) => {}
        (AppKind::Games, Lifecycle::Resume) => {}
        (AppKind::Games, Lifecycle::Pause) => {}
        (AppKind::Games, Lifecycle::Destroy) => {}

        (AppKind::Calculator, Lifecycle::Start) => {}
        (AppKind::Calculator, Lifecycle::Resume) => {}
        (AppKind::Calculator, Lifecycle::Pause) => {}
        (AppKind::Calculator, Lifecycle::Destroy) => {}

        (AppKind::Jackal, Lifecycle::Start) => {}
        (AppKind::Jackal, Lifecycle::Resume) => {}
        (AppKind::Jackal, Lifecycle::Pause) => {}
        (AppKind::Jackal, Lifecycle::Destroy) => {}

        (AppKind::Menger, Lifecycle::Start) => {}
        (AppKind::Menger, Lifecycle::Resume) => {}
        (AppKind::Menger, Lifecycle::Pause) => {}
        (AppKind::Menger, Lifecycle::Destroy) => {}

        (AppKind::Voodoo, Lifecycle::Start) => {}
        (AppKind::Voodoo, Lifecycle::Resume) => {}
        (AppKind::Voodoo, Lifecycle::Pause) => {}
        (AppKind::Voodoo, Lifecycle::Destroy) => {}

        (AppKind::Chronos, Lifecycle::Start) => {}
        (AppKind::Chronos, Lifecycle::Resume) => {}
        (AppKind::Chronos, Lifecycle::Pause) => {}
        (AppKind::Chronos, Lifecycle::Destroy) => {}

        (AppKind::Rtc, Lifecycle::Start) => {}
        (AppKind::Rtc, Lifecycle::Resume) => {}
        (AppKind::Rtc, Lifecycle::Pause) => {}
        (AppKind::Rtc, Lifecycle::Destroy) => {}

        (AppKind::Rng, Lifecycle::Start) => {}
        (AppKind::Rng, Lifecycle::Resume) => {}
        (AppKind::Rng, Lifecycle::Pause) => {}
        (AppKind::Rng, Lifecycle::Destroy) => {}

        (AppKind::Beeper, Lifecycle::Start) => {}
        (AppKind::Beeper, Lifecycle::Resume) => {}
        (AppKind::Beeper, Lifecycle::Pause) => {}
        (AppKind::Beeper, Lifecycle::Destroy) => {}

        (AppKind::Fpu, Lifecycle::Start) => {}
        (AppKind::Fpu, Lifecycle::Resume) => {}
        (AppKind::Fpu, Lifecycle::Pause) => {}
        (AppKind::Fpu, Lifecycle::Destroy) => {}

        (AppKind::Locale, Lifecycle::Start) => crate::apps::locale_switcher::on_start(),
        (AppKind::Locale, Lifecycle::Resume) => crate::apps::locale_switcher::on_resume(),
        (AppKind::Locale, Lifecycle::Pause) => crate::apps::locale_switcher::on_pause(),
        (AppKind::Locale, Lifecycle::Destroy) => {}

        (AppKind::Nhs, Lifecycle::Start) => {}
        (AppKind::Nhs, Lifecycle::Resume) => {}
        (AppKind::Nhs, Lifecycle::Pause) => {}
        (AppKind::Nhs, Lifecycle::Destroy) => {}
    }
}

/// Run the activity until it returns a non-Continue intent.
fn dispatch_update(kind: AppKind, depth: usize) -> ActivityIntent {
    match kind {
        AppKind::Launcher => crate::apps::launcher::update(depth),
        AppKind::Games => {
            crate::apps::games::run();
            ActivityIntent::Pop
        }
        AppKind::Calculator => {
            crate::apps::calculator::run();
            ActivityIntent::Pop
        }
        AppKind::Jackal => {
            crate::apps::jackal::shell::run_demo();
            ActivityIntent::Pop
        }
        AppKind::Menger => {
            crate::apps::menger::run_demo();
            ActivityIntent::Pop
        }
        AppKind::Voodoo => {
            crate::fb_buffer::clear_screen();
            crate::voodoo_engine::demo_cellular_automaton();
            wait_key();
            ActivityIntent::Pop
        }
        AppKind::Chronos => {
            crate::fb_buffer::clear_screen();
            crate::apps::chronos::display_triple_time();
            wait_key();
            ActivityIntent::Pop
        }
        AppKind::Rtc => {
            crate::fb_buffer::clear_screen();
            crate::apps::rtc::display_status();
            wait_key();
            ActivityIntent::Pop
        }
        AppKind::Rng => {
            crate::fb_buffer::clear_screen();
            crate::apps::rng::demo();
            wait_key();
            ActivityIntent::Pop
        }
        AppKind::Beeper => {
            crate::fb_buffer::clear_screen();
            crate::apps::beeper::demo_hex_scale();
            wait_key();
            ActivityIntent::Pop
        }
        AppKind::Fpu => {
            crate::apps::fpu::demo();
            ActivityIntent::Pop
        }
        AppKind::Locale => crate::apps::locale_switcher::update(depth),
        // NHS: run the NeroShizaScript stored in LAUNCH_NHS_SLOT, then pop.
        AppKind::Nhs => {
            let slot = unsafe { crate::apps::launcher::LAUNCH_NHS_SLOT as usize };
            crate::apps::installer::runtime::run_slot(slot);
            ActivityIntent::Pop
        }
    }
}

// ── Main entry point (called by shell command `apps`) ────────────────────────

pub fn run_activity_manager() {
    let _guard = InputGuard::new();

    let mut stack = ActivityStack::new();
    stack.push(AppKind::Launcher);
    dispatch_lifecycle(AppKind::Launcher, Lifecycle::Start);

    loop {
        crate::ps2::flush_debug_serial(64);
        let depth = stack.depth();
        let kind = match stack.top_kind() {
            Some(k) => k,
            None => break,
        };

        let intent = dispatch_update(kind, depth);

        match intent {
            ActivityIntent::Continue => {
                hlt();
            }

            ActivityIntent::Pop => {
                crate::serial_println!("[APP] exit  {:?}", kind);
                dispatch_lifecycle(kind, Lifecycle::Destroy);
                stack.pop();
                if stack.is_empty() {
                    break;
                }

                // Resume whatever is now on top
                let resumed = stack.top_kind().unwrap();
                crate::serial_println!("[APP] resume {:?}", resumed);
                dispatch_lifecycle(resumed, Lifecycle::Resume);
            }

            ActivityIntent::Push(next) => {
                crate::serial_println!("[APP] launch {:?}", next);
                dispatch_lifecycle(kind, Lifecycle::Pause);
                stack.push(next);
                dispatch_lifecycle(next, Lifecycle::Start);
            }

            ActivityIntent::Replace(next) => {
                crate::serial_println!("[APP] replace {:?} -> {:?}", kind, next);
                dispatch_lifecycle(kind, Lifecycle::Destroy);
                stack.pop();
                stack.push(next);
                dispatch_lifecycle(next, Lifecycle::Start);
            }
        }
    }

    // Back in shell territory
    crate::fb_buffer::clear_screen();
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

pub struct ActivityDebugStats {
    pub current: AppKind,
    pub depth: usize,
    pub max_depth: usize,
    pub updates: u32,
    pub switches: u32,
    pub pushes: u32,
    pub replaces: u32,
    pub pops: u32,
}

pub fn debug_stats() -> ActivityDebugStats {
    ActivityDebugStats {
        current: AppKind::Launcher,
        depth: 0,
        max_depth: 0,
        updates: 0,
        switches: 0,
        pushes: 0,
        replaces: 0,
        pops: 0,
    }
}

pub fn app_kind_name(k: AppKind) -> &'static str {
    match k {
        AppKind::Launcher => "Launcher",
        AppKind::Games => "Games",
        AppKind::Calculator => "Calculator",
        AppKind::Jackal => "Jackal",
        AppKind::Menger => "Menger",
        AppKind::Voodoo => "Voodoo",
        AppKind::Chronos => "Chronos",
        AppKind::Rtc => "Rtc",
        AppKind::Rng => "Rng",
        AppKind::Beeper => "Beeper",
        AppKind::Fpu => "Fpu",
        AppKind::Locale => "Locale",
        AppKind::Nhs => "Nhs",
    }
}

pub fn wait_key() {
    unsafe {
        loop {
            if crate::ps2::has_scancode() {
                let sc = crate::ps2::read_scancode();
                if sc & 0x80 == 0 {
                    break;
                }
            }
            hlt();
        }
    }
}
