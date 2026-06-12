use crate::util::vec2::*;

#[derive(Debug, Clone)]
pub struct Grid<T> {
	pub width: Coord,
	pub height: Coord,
	pub entries: Vec<T>,
}

impl<T> Default for Grid<T> {
	fn default() -> Self {
		Self {
			width: 2,
			height: 2,
			entries: Default::default(),
		}
	}
}

impl<T> Grid<T> {
	pub fn with_entries(width: Coord, height: Coord, entries: Vec<T>) -> Self {
		assert_eq!(entries.len(), (width * height) as usize);

		Self {
			width,
			height,
			entries,
		}
	}

	pub fn in_bounds(&self, pos: &Vec2) -> bool {
		let Vec2 { x, y } = *pos;
		x >= 0 && y >= 0 && y < self.height as i32 && x < self.width as i32
	}

	#[inline]
	pub fn get(&self, pos: Position) -> Option<&T> {
		let Position { x, y } = pos;
		if self.in_bounds(&pos.sign()) {
			Some(&self.entries[(y * self.width + x) as usize])
		} else {
			None
		}
	}

	#[inline]
	pub fn set(&mut self, pos: Position, value: T) {
		if self.in_bounds(&pos.sign()) {
			self.entries[(pos.y * self.width + pos.x) as usize] = value;
		} else {
			panic!("map index is out of range: {pos:?}")
		}
	}

	pub fn dimensions(&self) -> Position {
		Position {
			x: self.width,
			y: self.height,
		}
	}
}

impl<T> Grid<T>
where
	T: Default,
{
	pub fn new(width: Coord, height: Coord) -> Self {
		Self {
			width,
			height,
			entries: (0..width as usize * height as usize)
				.map(|_| T::default())
				.collect(),
		}
	}
}
