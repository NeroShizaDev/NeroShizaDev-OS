use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::{Uart16550Tty, backend::PioBackend, Config};

lazy_static! {
    pub static ref SERIAL1: Mutex<Uart16550Tty<PioBackend>> = {
        // SAFETY: 0x3F8 — стандартный базовый адрес COM1.
        // Вызывается единственный раз при первом обращении к SERIAL1.
        let serial = unsafe {
            Uart16550Tty::new_port(0x3F8, Config::default())
                .expect("serial: не могу инициализировать COM1")
        };
        Mutex::new(serial)
    };
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });
}

/// Выводит в хост через serial (без переноса строки).
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// Выводит в хост через serial (с переносом строки).
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
