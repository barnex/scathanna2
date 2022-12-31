use super::internal::*;

pub const X: usize = 0;
pub const Y: usize = 1;
pub const Z: usize = 2;

/// Direction an entity wants to move in,
/// based on the currently pressed keys and look direction.
pub fn walk_dir(yaw: f32, inputs: &Inputs) -> vec3 {
	let mut dir = vec3::ZERO;
	if inputs.is_down(inputs.LEFT) {
		dir[X] -= 1.0;
	}
	if inputs.is_down(inputs.RIGHT) {
		dir[X] += 1.0;
	}
	if inputs.is_down(inputs.FORWARD) {
		dir[Z] -= 1.0;
	}
	if inputs.is_down(inputs.BACKWARD) {
		dir[Z] += 1.0;
	}
	if dir == vec3::ZERO {
		return vec3::ZERO;
	}
	let dir = -yaw_matrix(-yaw).transform_point_ignore_w(dir);
	dir.safe_normalized()
}

/// Direction an entity wants to fly in,
/// based on the currently pressed keys and look direction.
pub fn fly_dir(yaw: f32, inputs: &Inputs) -> vec3 {
	let mut fly_dir = walk_dir(yaw, inputs);
	if inputs.is_down(inputs.JUMP) {
		fly_dir[Y] += 1.0;
	}
	if inputs.is_down(inputs.CROUCH) {
		fly_dir[Y] -= 1.0;
	}
	fly_dir.safe_normalized()
}
