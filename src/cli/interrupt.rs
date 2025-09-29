#![cfg(feature = "extras")]

#[inline]
pub fn enable_interrupt() {
	unsafe { std::env::set_var("ANTBYTE_INTERRUPT", "1") };
}

#[inline]
#[rustfmt::skip]
pub fn disable_interrupt() {
	unsafe { std::env::remove_var("ANTBYTE_INTERRUPT"); }
}

#[inline]
pub fn get_interrupt() -> bool {
	std::env::var("ANTBYTE_INTERRUPT").is_ok()
}
