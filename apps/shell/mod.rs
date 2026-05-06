mod commands;
mod runtime;
mod state;

#[path = "shell.rs"]
mod input;

pub use commands::{shell_dictionary_bytes, shell_dictionary_size};
pub use input::{handle_keyboard_input, handle_raw_key, show_shell_prompt};
pub use runtime::process_deferred_actions;
pub(crate) use runtime::{defer_shell_apps, defer_shell_reboot, defer_shell_shutdown};
pub use state::{
    ALT_HELD, CMD_STATS, CMD_TOTAL, CTRL_HELD, HIST_NAV, SCROLL_MODE, SCROLL_OFFSET, SHIFT_HELD,
};
pub(crate) use state::{
    bump_shell_command_stat, bump_shell_command_total, shell_command_stat, shell_command_total,
    shell_history_count, shell_hotkey_codepoint, shell_hotkey_len,
};
