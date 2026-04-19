pub mod dir;
pub mod matrix;
pub mod vec2;

#[inline]
#[rustfmt::skip]
pub fn sleep(ms: u32) { std::thread::sleep(std::time::Duration::from_millis(ms as u64)); }

pub fn find_dupe<T: PartialEq + Clone>(vec: &[T]) -> Option<T> {
	for i in 0..vec.len() {
		for j in 0..i {
			if vec[i] == vec[j] {
				return Some(vec[i].clone());
			}
		}
	}

	None
}
