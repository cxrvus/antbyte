#![cfg(test)]

use crate::ant::world::parser::SignatureSpec;

use super::{compile_func, compile_world_simple, stdlib::STDLIB};

impl<'a> SignatureSpec<'a> {
	fn new(name: &'a str, param_count: usize, assignee_count: usize) -> Self {
		Self {
			name,
			assignee_count,
			param_count,
		}
	}
}

fn test_func(signature: SignatureSpec, entries: Vec<u32>) {
	let truth_table = compile_func(STDLIB, signature);
	assert_eq!(truth_table.entries(), &entries)
}

#[test]
fn comp_std() {
	_ = compile_world_simple(STDLIB).unwrap()
}

#[test]
fn and2() {
	let signature = SignatureSpec::new("and", 2, 1);
	let entries = vec![0, 0, 0, 1];
	test_func(signature, entries);
}

#[test]
fn and3() {
	let signature = SignatureSpec::new("and", 3, 1);
	let entries = vec![0, 0, 0, 0, 0, 0, 0, 1];
	test_func(signature, entries);
}

#[test]
fn xor() {
	let signature = SignatureSpec::new("xor", 2, 1);
	let entries = vec![0, 1, 1, 0];
	test_func(signature, entries);
}

#[test]
fn eq() {
	let signature = SignatureSpec::new("eq", 2, 1);
	let entries = vec![1, 0, 0, 1];
	test_func(signature, entries);
}

#[test]
fn mux3() {
	let signature = SignatureSpec::new("mux", 3, 1);
	let entries = vec![0, 0, 1, 0, 0, 1, 1, 1];
	test_func(signature, entries);
}

#[test]
fn mux6() {
	let signature = SignatureSpec::new("mux", 6, 1);
	let entries = vec![
		0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 1, 1, 0, 1, 1,
		1, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1, 0, 1, 1, 1, 0, 1, 0, 0, 1, 1, 1, 0, 1, 1, 0, 1, 1, 1,
		1, 1, 1, 1,
	];
	test_func(signature, entries);
}

#[test]
fn h_add() {
	let signature = SignatureSpec::new("add", 2, 2);
	let entries = vec![0, 1, 1, 2];
	test_func(signature, entries);
}

#[test]
fn add() {
	let signature = SignatureSpec::new("add", 3, 2);
	let entries = vec![0, 1, 1, 2, 1, 2, 2, 3];
	test_func(signature, entries);
}

#[test]
fn cpy2() {
	let signature = SignatureSpec::new("cpy", 1, 2);
	let entries = vec![0, 3];
	test_func(signature, entries);
}

#[test]
fn buf2() {
	let signature = SignatureSpec::new("buf", 2, 2);
	let entries = vec![0, 1, 2, 3];
	test_func(signature, entries);
}

#[test]
fn inv2() {
	let signature = SignatureSpec::new("inv", 2, 2);
	let entries = vec![3, 2, 1, 0];
	test_func(signature, entries);
}
