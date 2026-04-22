/// Apps layer — fully separate from the kernel.
///
/// Entry point for the shell command `apps`:
///   crate::apps::activity::run_activity_manager()
///
/// Architecture: Android-style ActivityStack
///   Shell → run_activity_manager() → ActivityStack([Launcher])
///   User picks Games → Push(Games)  → ActivityStack([Launcher, Games])
///   User presses Esc → Exit         → ActivityStack([Launcher])
///   User presses Esc → Exit         → stack empty → back to shell

pub mod activity;
pub mod launcher;
pub mod games;
pub mod installer;
pub mod jackal;
pub mod locale_switcher;

// Moved from moduls/
pub mod menger;
pub mod chronos;
pub mod rng;
pub mod rtc;

// Moved from demo/
pub mod beeper;
pub mod fpu;

// Doom lives in apps/doom/ but its canonical crate path is still crate::doom
// (re-exported from lib.rs) so existing call sites don't break.
pub use crate::doom;
