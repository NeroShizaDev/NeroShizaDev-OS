// Games namespace — мини-ОС внутри ядра NeroShizaDev-OS
// Возвращает управление в apps::launcher при выходе (Q/Esc).

pub mod common_hw;
pub mod absurd_loading_screen;
pub mod clicker_game;
pub mod match_grab_and_leave;
pub mod debil_card_game;
pub mod module_demo;
pub mod byte_dodge;
pub mod launcher;

/// Lifecycle hooks — вызываются ActivityManager'ом.
pub fn on_start() {
    run();
}

pub fn on_resume() {
    run();
}

/// Точка входа: запустить Games-меню. Возвращается в вызывающий код.
pub fn run() {
    let mut l = launcher::Launcher::new();
    unsafe { l.run(); }
}
