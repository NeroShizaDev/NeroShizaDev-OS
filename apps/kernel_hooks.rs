use x86_64::instructions::port::Port;

pub fn init_for_kernel() {
    crate::apps::fpu::init();
    crate::apps::games::doom::stubs::init_heap();
}

pub fn display_boot_time_and_thermal() {
    crate::apps::rtc::display_status();
    crate::apps::rtc::display_thermal();
}

pub fn rng_supported() -> bool {
    crate::apps::rng::is_supported()
}

pub fn play_boot_beep() {
    unsafe {
        crate::apps::beeper::play(880);
        let mut port: Port<u8> = Port::new(0x80);
        for _ in 0u32..200_000 {
            core::hint::black_box(port.read());
        }
        crate::apps::beeper::stop();
    }
}
