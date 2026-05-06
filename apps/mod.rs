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
pub mod calculator;
pub mod games;
pub mod installer;
pub mod jackal;
pub mod kernel_hooks;
pub mod launcher;
pub mod locale_switcher;

// Moved from moduls/
pub mod chronos;
pub mod menger;
pub mod rng;
pub mod rtc;

// Moved from demo/
pub mod beeper;
pub mod fpu;
