use super::internal::*;

pub fn unit_cube_faces(mat: MatID) -> [Face; 6] {
	[
		Face::rectangle(mat, (0, 1, 1), (0, 1, 0), (0, 0, 0), (0, 0, 1)), // left
		Face::rectangle(mat, (1, 0, 0), (1, 1, 0), (1, 1, 1), (1, 0, 1)), // right
		Face::rectangle(mat, (0, 0, 0), (1, 0, 0), (1, 0, 1), (0, 0, 1)), // bottom
		Face::rectangle(mat, (1, 1, 1), (1, 1, 0), (0, 1, 0), (0, 1, 1)), // top
		Face::rectangle(mat, (0, 1, 0), (1, 1, 0), (1, 0, 0), (0, 0, 0)), // back
		Face::rectangle(mat, (1, 0, 1), (1, 1, 1), (0, 1, 1), (0, 0, 1)), // front
	]
}
