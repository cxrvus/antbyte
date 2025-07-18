use crate::util::vec2::*;

#[derive(Debug, Clone)]
pub struct Matrix<T> {
	pub width: usize,
	pub height: usize,
	pub values: Vec<T>,
}

impl<T> Matrix<T> {
	pub fn with_values(width: usize, height: usize, values: Vec<T>) -> Self {
		assert_eq!(values.len(), width * height);

		Self {
			width,
			height,
			values,
		}
	}

	pub fn in_bounds(&self, pos: &Vec2) -> bool {
		let Vec2 { x, y } = *pos;
		x >= 0 && y >= 0 && y < self.height as i32 && x < self.width as i32
	}

	pub fn at(&self, pos: &Vec2) -> Option<&T> {
		let Vec2u { x, y } = pos.unsign()?;
		if self.in_bounds(pos) {
			Some(&self.values[y * self.width + x])
		} else {
			None
		}
	}

	pub fn set_at(&mut self, pos: &Vec2, value: T) {
		if self.in_bounds(pos) {
			let Vec2u { x, y } = pos.unsign().unwrap();
			self.values[y * self.width + x] = value;
		} else {
			panic!("map index is out of range: {pos:?}")
		}
	}

	pub fn get_pos(&self, i: usize) -> Option<Vec2u> {
		self.values.get(i)?;
		Some(Vec2u {
			x: (i % self.width),
			y: (i / self.width),
		})
	}

	pub fn dimensions(&self) -> Vec2u {
		Vec2u {
			x: self.width,
			y: self.height,
		}
	}

	pub fn get_row(&self, i: usize) -> Option<Vec<&T>> {
		if i >= self.height {
			None
		} else {
			let start = i * self.width;
			let end = (i + 1) * self.width;
			let row = self.values[start..end].iter().collect::<Vec<&T>>();
			Some(row)
		}
	}

	pub fn get_col(&self, i: usize) -> Option<Vec<&T>> {
		if i >= self.width {
			None
		} else {
			let col = (0..self.height)
				.map(|row| &self.values[row * self.width + i])
				.collect::<Vec<&T>>();
			Some(col)
		}
	}
}

impl<T> Matrix<T>
where
	T: Default,
{
	pub fn new(width: usize, height: usize) -> Self {
		Self {
			width,
			height,
			values: (0..width * height).map(|_| T::default()).collect(),
		}
	}
}

impl<T> Matrix<T>
where
	T: Default + PartialEq,
{
	pub fn find_all(&self, target: T) -> Vec<Vec2u> {
		self.values
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
			values: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
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

#[derive(Debug)]
pub struct ProxyMatrix {
	pub width: usize,
	pub height: usize,
	pub string: String,
}

impl ProxyMatrix {
	pub fn convert<T>(self, parser: fn(String) -> Vec<T>) -> Matrix<T> {
		Matrix {
			width: self.width,
			height: self.height,
			values: parser(self.string),
		}
	}
}

impl From<&str> for ProxyMatrix {
	fn from(value: &str) -> Self {
		let lines = value.trim().lines();

		Self {
			height: lines.clone().count(),
			width: lines.clone().next().unwrap().len(),
			string: lines.collect::<Vec<&str>>().join(""),
		}
	}
}
