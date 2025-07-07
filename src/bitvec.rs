use std::ops::{Deref, DerefMut};

#[derive(Clone)]
pub struct BitVec(Vec<bool>);

impl BitVec {
	pub fn new() -> Self {
		BitVec(vec![])
	}

	pub fn repeat(value: bool, count: usize) -> Self {
		BitVec(vec![value; count])
	}

	pub fn unary(&self) -> Option<bool> {
		if self.len() == 1 { Some(self[0]) } else { None }
	}

	pub fn gated_buffer(&self, enable: bool) -> Self {
		match enable {
			true => self.clone(),
			false => Self::repeat(false, self.len()),
		}
	}

	pub fn or_sum(&self) -> bool {
		self.0.iter().any(|&b| b)
	}

	pub fn and_sum(&self) -> bool {
		self.0.iter().all(|&b| b)
	}

	pub fn sum(&self) -> usize {
		self.iter().map(|&b| b as usize).sum()
	}

	fn binary_op<F>(&self, other: &BitVec, op: F) -> BitVec
	where
		F: Fn(bool, bool) -> bool,
	{
		assert_eq!(self.len(), other.len());

		let bits = self
			.0
			.iter()
			.zip(&other.0)
			.map(|(a, b)| op(*a, *b))
			.collect();

		BitVec(bits)
	}

	pub fn and(&self, other: &Self) -> Self {
		self.binary_op(other, |a, b| a & b)
	}

	pub fn or(&self, other: &Self) -> Self {
		self.binary_op(other, |a, b| a | b)
	}

	pub fn xor(&self, other: &Self) -> Self {
		self.binary_op(other, |a, b| a ^ b)
	}

	pub fn invert(&self) -> Self {
		Self(self.iter().map(|a| !*a).collect())
	}
}

impl Default for BitVec {
	fn default() -> Self {
		Self::new()
	}
}

impl Deref for BitVec {
	type Target = Vec<bool>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for BitVec {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl From<Vec<bool>> for BitVec {
	fn from(value: Vec<bool>) -> Self {
		BitVec(value)
	}
}

impl From<bool> for BitVec {
	fn from(value: bool) -> Self {
		BitVec(vec![value])
	}
}
