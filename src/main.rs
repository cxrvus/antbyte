use crate::{map::Map, vec::Vec2};

pub mod ant;
pub mod bitvec;
pub mod map;
pub mod vec;
pub mod world;

fn main() {
	let mut board = Map::<u8>::new(8, 8);

	board.set_at(&Vec2 { x: 4, y: 6 }, 1);

	let board_str = board
		.values
		.iter()
		.map(|x| match x {
			0 => "..".to_string(),
			_ => "##".to_string(),
		})
		.collect::<Vec<_>>()
		.chunks(board.width)
		.map(|chunk| chunk.join("") + "\n")
		.collect::<String>();

	println!("\n{board_str}\n");
}
