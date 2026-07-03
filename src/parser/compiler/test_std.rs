#![cfg(test)]

use super::{SignatureSpec, compile_func, compile_world_simple, stdlib::STDLIB};

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
fn eq2() {
	let signature = SignatureSpec::new("eq", 2, 1);
	let entries = vec![1, 0, 0, 1];
	test_func(signature, entries);
}

#[test]
fn eq3() {
	let signature = SignatureSpec::new("eq", 3, 1);
	let entries = vec![1, 0, 0, 0, 0, 0, 0, 1];
	test_func(signature, entries);
}

#[test]
fn eq4() {
	let signature = SignatureSpec::new("eq", 4, 1);
	let entries = vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
	test_func(signature, entries);
}

#[test]
fn imply() {
	let signature = SignatureSpec::new("imply", 2, 1);
	let entries = vec![1, 1, 0, 1];
	test_func(signature, entries);
}

#[test]
fn mux3() {
	let signature = SignatureSpec::new("mux", 3, 1);
	let entries = vec![0, 0, 1, 1, 0, 1, 0, 1];
	test_func(signature, entries);
}

#[test]
fn mux6() {
	let signature = SignatureSpec::new("mux", 6, 1);
	let entries = vec![
		0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1,
		1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
		0, 1, 0, 1,
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
fn h_add2() {
	let signature = SignatureSpec::new("add", 4, 3);
	let entries = vec![0, 1, 2, 3, 1, 2, 3, 4, 2, 3, 4, 5, 3, 4, 5, 6];
	test_func(signature, entries);
}

#[test]
fn add2() {
	let signature = SignatureSpec::new("add", 5, 3);
	let entries = vec![
		0, 1, 1, 2, 2, 3, 3, 4, 1, 2, 2, 3, 3, 4, 4, 5, 2, 3, 3, 4, 4, 5, 5, 6, 3, 4, 4, 5, 5, 6,
		6, 7,
	];
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
fn buf3() {
	let signature = SignatureSpec::new("buf", 3, 3);
	let entries = vec![0, 1, 2, 3, 4, 5, 6, 7];
	test_func(signature, entries);
}

#[test]
fn p_eq4() {
	let signature = SignatureSpec::new("p_eq", 4, 1);
	let entries = (0..(2u32.pow(4)))
		.map(|x| (x & 0b11 == ((x & 0b1100) >> 2)) as u32)
		.rev()
		.collect::<Vec<_>>();
	test_func(signature, entries);
}

#[test]
fn p_eq6() {
	let signature = SignatureSpec::new("p_eq", 6, 1);
	let entries = (0..(2u32.pow(6)))
		.map(|x| (x & 0b111 == ((x & 0b111000) >> 3)) as u32)
		.rev()
		.collect::<Vec<_>>();
	test_func(signature, entries);
}

#[test]
fn p_eq8() {
	let signature = SignatureSpec::new("p_eq", 8, 1);
	let entries = (0..(2u32.pow(8)))
		.map(|x| (x & 0b1111 == ((x & 0b11110000) >> 4)) as u32)
		.collect();
	test_func(signature, entries);
}

// todo: test for enb()

#[test]
fn dec2() {
	let signature = SignatureSpec::new("dec", 2, 4);
	let entries = (0..(2u32.pow(2))).rev().map(|x| 1 << x).collect();
	test_func(signature, entries);
}

#[test]
fn dec3() {
	let signature = SignatureSpec::new("dec", 3, 8);
	let entries = (0..(2u32.pow(3))).rev().map(|x| 1 << x).collect();
	test_func(signature, entries);
}

#[test]
fn dec4() {
	let signature = SignatureSpec::new("dec", 4, 16);
	let entries = (0..(2u32.pow(4))).rev().map(|x| 1 << x).collect();
	test_func(signature, entries);
}

#[test]
fn enc2() {
	let signature = SignatureSpec::new("enc", 4, 2);
	let entries = vec![0, 3, 2, 2, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0];
	test_func(signature, entries);
}

#[test]
fn enc3() {
	let signature = SignatureSpec::new("enc", 8, 3);
	let entries = (0u32..0x100)
		.map(|x| match x {
			0 => 0,
			1 => 7,
			2..=3 => 6,
			4..=7 => 5,
			8..=15 => 4,
			16..=31 => 3,
			32..=63 => 2,
			64..=127 => 1,
			_ => 0,
		})
		.collect();
	test_func(signature, entries);
}

#[test]
fn hw4() {
	let signature = SignatureSpec::new("hw", 4, 3);
	let entries = (0u32..0x10).map(|x| x.count_ones()).collect();
	test_func(signature, entries);
}

#[test]
fn hw8() {
	let signature = SignatureSpec::new("hw", 8, 4);
	let entries = (0u32..0x100).map(|x| x.count_ones()).collect();
	test_func(signature, entries);
}

#[test]
fn one3() {
	let signature = SignatureSpec::new("one", 3, 1);
	let entries = (0u32..8).map(|x| (x.count_ones() == 1) as u32).collect();
	test_func(signature, entries);
}

#[test]
fn one4() {
	let signature = SignatureSpec::new("one", 4, 1);
	let entries = (0u32..16).map(|x| (x.count_ones() == 1) as u32).collect();
	test_func(signature, entries);
}
