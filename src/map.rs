use crate::vec::*;

#[derive(Debug, Clone)]
pub struct Map<T>
where
	T: Default + PartialEq + Clone,
{
	pub width: usize,
	pub height: usize,
	pub values: Vec<T>,
}

impl<T> Map<T>
where
	T: Default + PartialEq + Clone,
{
	pub fn new(width: usize, height: usize) -> Self {
		Self {
			width,
			height,
			values: vec![T::default(); width * height],
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
			panic!("map index is out of range: {:?}", pos)
		}
	}

	pub fn find_all(&self, target: T) -> Vec<Vec2u> {
		self.values
			.iter()
			.enumerate()
			.filter(|(_, value)| **value == target)
			.map(|(i, _)| self.get_pos(i).unwrap())
			.collect()
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
}

#[derive(Debug)]
pub struct ProxyMap {
	pub width: usize,
	pub height: usize,
	pub string: String,
}

impl ProxyMap {
	pub fn convert<T>(self, parser: fn(String) -> Vec<T>) -> Map<T>
	where
		T: Default + PartialEq + Clone,
	{
		Map {
			width: self.width,
			height: self.height,
			values: parser(self.string),
		}
	}
}

impl From<&str> for ProxyMap {
	fn from(value: &str) -> Self {
		let lines = value.trim().lines();

		Self {
			height: lines.clone().count(),
			width: lines.clone().next().unwrap().len(),
			string: lines.collect::<Vec<&str>>().join(""),
		}
	}
}
