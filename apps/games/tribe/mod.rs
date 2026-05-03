mod db;
mod db_ext;
mod game;
mod gurps;
mod input;
mod live_rules;
mod print;
mod tribe;
mod watchdog;
mod x87_rng;

pub fn on_start() {
    crate::serial_println!("[tribe] on_start");
    input::reset();
    watchdog::init();
}

pub fn on_resume() {
    crate::serial_println!("[tribe] on_resume");
    crate::ps2::clear_scancode_queue();
    input::reset();
    watchdog::init();
}

pub fn on_pause() {}

pub fn on_destroy() {
    crate::serial_println!("[tribe] on_destroy");
    crate::ps2::clear_scancode_queue();
    input::reset();
}

pub fn run() {
    if crate::fb_buffer::is_initialized() {
        crate::serial_println!("[tribe] UEFI/framebuffer mode: VGA text игра недоступна");
        return;
    }
    crate::serial_println!("[tribe] run enter");
    game::run();
    crate::serial_println!("[tribe] run exit");
}
