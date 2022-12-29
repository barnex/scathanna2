use super::internal::*;

pub const X: usize = 0;
pub const Y: usize = 1;
pub const Z: usize = 2;

/// Direction an entity wants to move in,
/// based on the currently pressed keys and look direction.
pub fn walk_dir(yaw: f32, input: &Inputs) -> vec3 {
	let mut dir = vec3::ZERO;
	if input.is_down(Button::LEFT) {
		dir[X] -= 1.0;
	}
	if input.is_down(Button::RIGHT) {
		dir[X] += 1.0;
	}
	if input.is_down(Button::FORWARD) {
		dir[Z] -= 1.0;
	}
	if input.is_down(Button::BACKWARD) {
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
pub fn fly_dir(yaw: f32, input: &Inputs) -> vec3 {
	let mut fly_dir = walk_dir(yaw, input);
	if input.is_down(Button::JUMP) {
		fly_dir[Y] += 1.0;
	}
	if input.is_down(Button::CROUCH) {
		fly_dir[Y] -= 1.0;
	}
	fly_dir.safe_normalized()
}
