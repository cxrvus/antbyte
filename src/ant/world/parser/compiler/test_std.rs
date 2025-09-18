#![cfg(test)]

use super::{FuncCall, compile_func, compile_world_simple, stdlib::STDLIB};

fn test_func(call: FuncCall, entries: Vec<u32>) {
	let truth_table = compile_func(STDLIB, call);
	assert_eq!(truth_table.entries(), &entries)
}

#[test]
fn comp_std() {
	_ = compile_world_simple(STDLIB).unwrap()
}

#[test]
fn and2() {
	let call = FuncCall::from_spec("and", 2, 1);
	let entries = vec![0, 0, 0, 1];
	test_func(call, entries);
}

#[test]
fn and3() {
	let call = FuncCall::from_spec("and", 3, 1);
	let entries = vec![0, 0, 0, 0, 0, 0, 0, 1];
	test_func(call, entries);
}

#[test]
fn xor() {
	let call = FuncCall::from_spec("xor", 2, 1);
	let entries = vec![0, 1, 1, 0];
	test_func(call, entries);
}

#[test]
fn eq() {
	let call = FuncCall::from_spec("eq", 2, 1);
	let entries = vec![1, 0, 0, 1];
	test_func(call, entries);
}

#[test]
fn mux3() {
	let call = FuncCall::from_spec("mux", 3, 1);
	let entries = vec![0, 0, 1, 0, 0, 1, 1, 1];
	test_func(call, entries);
}

#[test]
fn mux6() {
	let call = FuncCall::from_spec("mux", 6, 1);
	let entries = vec![
		0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 1, 1, 0, 1, 1,
		1, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1, 0, 1, 1, 1, 0, 1, 0, 0, 1, 1, 1, 0, 1, 1, 0, 1, 1, 1,
		1, 1, 1, 1,
	];
	test_func(call, entries);
}

#[test]
fn h_add() {
	let call = FuncCall::from_spec("add", 2, 2);
	let entries = vec![0, 1, 1, 2];
	test_func(call, entries);
}

#[test]
fn add() {
	let call = FuncCall::from_spec("add", 3, 2);
	let entries = vec![0, 1, 1, 2, 1, 2, 2, 3];
	test_func(call, entries);
}

#[test]
fn cpy2() {
	let call = FuncCall::from_spec("cpy", 1, 2);
	let entries = vec![0, 3];
	test_func(call, entries);
}

#[test]
fn buf2() {
	let call = FuncCall::from_spec("buf", 2, 2);
	let entries = vec![0, 1, 2, 3];
	test_func(call, entries);
}

#[test]
fn inv2() {
	let call = FuncCall::from_spec("inv", 2, 2);
	let entries = vec![3, 2, 1, 0];
	test_func(call, entries);
}
