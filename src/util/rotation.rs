use std::sync::LazyLock;

use crate::util::dir::Direction;

const ACC_SCALE: i8 = 64;

static DELTA_TABLE: LazyLock<[(i8, i8); 0x100]> = LazyLock::new(|| {
	std::array::from_fn(|dir| {
		let angle = (dir as f64) * (2.0 * std::f64::consts::PI / 256.0);

		let raw_dx = angle.cos();
		let raw_dy = angle.sin();

		// normalize so the larger axis == 1.0, then scale to SCALE
		let largest = raw_dx.abs().max(raw_dy.abs());

		let dx = ((raw_dx / largest) * ACC_SCALE as f64).round() as i8;
		let dy = ((raw_dy / largest) * ACC_SCALE as f64).round() as i8;

		(dx, dy)
	})
});

pub struct Rotation {
	angle: u8,
	acc_x: i8,
	acc_y: i8,
}

impl Rotation {
	pub fn tick(&mut self, delta_angle: u8) -> Direction {
		self.angle = self.angle.wrapping_add(delta_angle);

		let (dx, dy) = DELTA_TABLE[self.angle as usize];

		add_delta(&mut self.acc_x, dx);
		add_delta(&mut self.acc_y, dy);

		let mut step_x: i8 = 0;
		let mut step_y: i8 = 0;

		if self.acc_x >= ACC_SCALE {
			step_x = 1;
			self.acc_x -= ACC_SCALE;
		} else if self.acc_x <= -ACC_SCALE {
			step_x = -1;
			self.acc_x += ACC_SCALE;
		}

		if self.acc_y >= ACC_SCALE {
			step_y = 1;
			self.acc_y -= ACC_SCALE;
		} else if self.acc_y <= -ACC_SCALE {
			step_y = -1;
			self.acc_y += ACC_SCALE;
		}

		to_dir((step_x, step_y))
	}
}

fn add_delta(acc: &mut i8, delta: i8) {
	if delta > 0 && *acc < 0 || delta < 0 && *acc > 0 {
		*acc = 0;
	}

	*acc += delta;
}

fn to_dir(vec: (i8, i8)) -> Direction {
	Direction::from(match vec {
		(1, 0) => 0,
		(1, 1) => 1,
		(0, 1) => 2,
		(-1, 1) => 3,
		(-1, 0) => 4,
		(-1, -1) => 5,
		(0, -1) => 6,
		(1, -1) => 7,
		_ => panic!(
			"rotation returned an invalid direction vector: ({}, {})",
			vec.0, vec.1
		),
	})
}
