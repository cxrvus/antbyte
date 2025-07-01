pub struct BitVec(Vec<bool>);

impl BitVec {
	pub fn new(size: usize) -> Self {
		BitVec(vec![false; size])
	}

	pub fn bits(&self) -> &Vec<bool> {
		&self.0
	}

	fn binary_op<F>(&self, other: &BitVec, op: F) -> BitVec
	where
		F: Fn(bool, bool) -> bool,
	{
		let bits = self
			.0
			.iter()
			.zip(&other.0)
			.map(|(a, b)| op(*a, *b))
			.collect();
		BitVec(bits)
	}

	pub fn and(&self, other: &BitVec) -> BitVec {
		self.binary_op(other, |a, b| a & b)
	}

	pub fn or(&self, other: &BitVec) -> BitVec {
		self.binary_op(other, |a, b| a | b)
	}

	pub fn nand(&self, other: &BitVec) -> BitVec {
		self.binary_op(other, |a, b| !(a & b))
	}

	pub fn nor(&self, other: &BitVec) -> BitVec {
		self.binary_op(other, |a, b| !(a | b))
	}

	pub fn xor(&self, other: &BitVec) -> BitVec {
		self.binary_op(other, |a, b| a ^ b)
	}

	pub fn xnor(&self, other: &BitVec) -> BitVec {
		self.binary_op(other, |a, b| !(a ^ b))
	}

	pub fn invert(&self) -> BitVec {
		BitVec(self.0.iter().map(|a| !*a).collect())
	}
}

impl From<Vec<bool>> for BitVec {
	fn from(value: Vec<bool>) -> Self {
		BitVec(value)
	}
}
