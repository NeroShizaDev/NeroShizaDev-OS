pub mod fb_buffer;
pub mod vga_hw;
pub mod vga_unicode;

#[macro_export]
macro_rules! print {
	($($arg:tt)*) => ($crate::fb_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
	() => ($crate::print!("\n"));
	($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
