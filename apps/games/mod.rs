// Games namespace — мини-ОС внутри ядра NeroShizaDev-OS
// Возвращает управление в apps::launcher при выходе (Q/Esc).

pub mod absurd_loading_screen;
pub mod byte_dodge;
pub mod clicker_game;
pub mod common_hw;
pub mod debil_card_game;
pub mod doom;
pub mod launcher;
pub mod match_grab_and_leave;
pub mod module_demo;
pub mod tribe;

/// Lifecycle hooks — вызываются ActivityManager'ом.
/// Не запускают run() — это делает dispatch_update().
pub fn on_start() {}
pub fn on_resume() {}

/// Точка входа: запустить Games-меню. Возвращается в вызывающий код.
pub fn run() {
    let mut l = launcher::Launcher::new();
    unsafe {
        l.run();
    }
}
