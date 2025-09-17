#![cfg(test)]

use super::compile_func;

fn test_func(code: &str, entries: Vec<u32>) {
	let truth_table = compile_func(code.to_owned(), "main");
	assert_eq!(truth_table.entries(), &entries)
}

const XOR: &str = "(a, b) => c { c = or(and(-a, b), and(a, -b)); }";

fn with_std(code: &str) -> String {
	format!(
		r"
		fn xor = {XOR}
		
		{code}"
	)
}

mod gates {
	use super::*;

	#[test]
	fn empty() {
		let code = r"fn main = () => () { }";
		let entries = vec![0];
		test_func(code, entries);
	}

	#[test]
	fn buffer_gate() {
		let code = r"fn main = p => q { q = p; }";
		let entries = vec![0, 1];
		test_func(code, entries);
	}

	#[test]
	fn not_gate() {
		let code = r"fn main = p => q { q = -p; }";
		let entries = vec![1, 0];
		test_func(code, entries);
	}

	#[test]
	fn or_gate() {
		let code = r"fn main = (a, b) => c { c = or(a, b); }";
		let entries = vec![0, 1, 1, 1];
		test_func(code, entries);
	}

	#[test]
	fn or_gate3() {
		let code = r"fn main = (a, b, c) => r { r = or(a, b, c); }";
		let entries = vec![0, 1, 1, 1, 1, 1, 1, 1];
		test_func(code, entries);
	}

	#[test]
	fn and_gate() {
		let code = r"fn main = (a, b) => c { c = and(a, b); }";
		let entries = vec![0, 0, 0, 1];
		test_func(code, entries);
	}

	#[test]
	fn and_gate3() {
		let code = r"fn main = (a, b, c) => r { r = and(a, b, c); }";
		let entries = vec![0, 0, 0, 0, 0, 0, 0, 1];
		test_func(code, entries);
	}

	#[test]
	fn xor_gate() {
		let code = &with_std("fn main = (a, b) => c { c = xor(a, b); }");
		let entries = vec![0, 1, 1, 0];
		test_func(code, entries);
	}
}

mod math {
	use super::*;

	#[test]
	fn h_add() {
		let code = &with_std(
			r"
		fn main = (a, b) => (sum, c_out) {
			sum = xor(a, b);
			c_out = and(a, b);
		}",
		);

		let entries = vec![0, 1, 1, 2];
		test_func(code, entries);
	}
}
