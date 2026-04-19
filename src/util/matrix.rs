use crate::util::vec2::*;

#[derive(Debug, Clone)]
pub struct Matrix<T> {
	pub width: Coord,
	pub height: Coord,
	pub entries: Vec<T>,
}

impl<T> Matrix<T> {
	pub fn with_values(width: Coord, height: Coord, values: Vec<T>) -> Self {
		assert_eq!(values.len(), (width * height) as usize);

		Self {
			width,
			height,
			entries: values,
		}
	}

	pub fn in_bounds(&self, pos: &Vec2) -> bool {
		let Vec2 { x, y } = *pos;
		x >= 0 && y >= 0 && y < self.height as i32 && x < self.width as i32
	}

	#[inline]
	pub fn at(&self, pos: &Vec2u) -> Option<&T> {
		let Vec2u { x, y } = pos;
		if self.in_bounds(&pos.sign()) {
			Some(&self.entries[(y * self.width + x) as usize])
		} else {
			None
		}
	}

	#[inline]
	pub fn set_at(&mut self, pos: &Vec2u, value: T) {
		if self.in_bounds(&pos.sign()) {
			self.entries[(pos.y * self.width + pos.x) as usize] = value;
		} else {
			panic!("map index is out of range: {pos:?}")
		}
	}

	pub fn get_pos(&self, i: usize) -> Option<Vec2u> {
		self.entries.get(i)?;
		Some(Vec2u {
			x: (i % self.width as usize) as Coord,
			y: (i / self.width as usize) as Coord,
		})
	}

	pub fn dimensions(&self) -> Vec2u {
		Vec2u {
			x: self.width,
			y: self.height,
		}
	}

	pub fn get_row(&self, i: usize) -> Option<Vec<&T>> {
		let (width, height) = (self.width as usize, self.height as usize);

		if i >= height {
			None
		} else {
			let start = i * width;
			let end = (i + 1) * width;
			let row = self.entries[start..end].iter().collect::<Vec<&T>>();
			Some(row)
		}
	}

	pub fn get_col(&self, i: usize) -> Option<Vec<&T>> {
		let (width, height) = (self.width as usize, self.height as usize);

		if i >= width {
			None
		} else {
			let col = (0..height)
				.map(|row| &self.entries[row * width + i])
				.collect::<Vec<&T>>();
			Some(col)
		}
	}
}

impl<T> Matrix<T>
where
	T: Default,
{
	pub fn new(width: Coord, height: Coord) -> Self {
		Self {
			width,
			height,
			entries: (0..width * height).map(|_| T::default()).collect(),
		}
	}
}

impl<T> Matrix<T>
where
	T: Default + PartialEq,
{
	pub fn find_all(&self, target: T) -> Vec<Vec2u> {
		self.entries
			.iter()
			.enumerate()
			.filter(|(_, value)| **value == target)
			.map(|(i, _)| self.get_pos(i).unwrap())
			.collect()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn create_test_matrix() -> Matrix<i32> {
		Matrix {
			width: 3,
			height: 3,
			entries: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
		}
	}

	#[test]
	fn test_get_row() {
		let m = create_test_matrix();

		let row0 = m.get_row(0).unwrap();
		assert_eq!(row0, vec![&1, &2, &3]);

		let row1 = m.get_row(1).unwrap();
		assert_eq!(row1, vec![&4, &5, &6]);

		let row2 = m.get_row(2).unwrap();
		assert_eq!(row2, vec![&7, &8, &9]);

		assert!(m.get_row(3).is_none());
	}

	#[test]
	fn test_get_col() {
		let m = create_test_matrix();

		let col0 = m.get_col(0).unwrap();
		assert_eq!(col0, vec![&1, &4, &7]);

		let col1 = m.get_col(1).unwrap();
		assert_eq!(col1, vec![&2, &5, &8]);

		let col2 = m.get_col(2).unwrap();
		assert_eq!(col2, vec![&3, &6, &9]);

		assert!(m.get_col(3).is_none());
	}
}
