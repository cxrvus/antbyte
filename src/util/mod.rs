pub mod matrix;
pub mod vec2;

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
