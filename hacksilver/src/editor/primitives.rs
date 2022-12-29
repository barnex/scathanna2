use super::internal::*;

pub fn unit_cube_faces(mat: MatID) -> SmallVec<[Face; 6]> {
	smallvec![
		Face::rectangle(mat, (0, 1, 1), (0, 1, 0), (0, 0, 0), (0, 0, 1)), // left
		Face::rectangle(mat, (1, 0, 0), (1, 1, 0), (1, 1, 1), (1, 0, 1)), // right
		Face::rectangle(mat, (0, 0, 0), (1, 0, 0), (1, 0, 1), (0, 0, 1)), // bottom
		Face::rectangle(mat, (1, 1, 1), (1, 1, 0), (0, 1, 0), (0, 1, 1)), // top
		Face::rectangle(mat, (0, 1, 0), (1, 1, 0), (1, 0, 0), (0, 0, 0)), // back
		Face::rectangle(mat, (1, 0, 1), (1, 1, 1), (0, 1, 1), (0, 0, 1)), // front
	]
}

//   +
//  / \
// +   +
// |\ /
// +-+
pub fn unit_wedge_faces(mat: MatID) -> SmallVec<[Face; 6]> {
	smallvec![
		Face::rectangle(mat, (0, 1, 1), (0, 1, 0), (0, 0, 0), (0, 0, 1)), // left
		Face::rectangle(mat, (0, 0, 0), (1, 0, 0), (1, 0, 1), (0, 0, 1)), // bottom
		Face::rectangle(mat, (1, 0, 1), (1, 0, 0), (0, 1, 0), (0, 1, 1)), // top
		Face::triangle(mat, (1, 0, 0), (0, 0, 0), (0, 1, 0),),            // back
		Face::triangle(mat, (0, 1, 1), (0, 0, 1), (1, 0, 1)),             // front
	]
}

pub fn unit_tetra_faces(mat: MatID) -> SmallVec<[Face; 6]> {
	smallvec![
		Face::triangle(mat, (0, 1, 0), (0, 0, 0), (0, 0, 1)),  // left
		Face::triangle(mat, (0, 0, 1), (0, 0, 0), (1, 0, 0)),  // bottom
		Face::triangle(mat, (1, 0, 0), (0, 0, 0), (0, 1, 0),), // back
		Face::triangle(mat, (0, 1, 0), (0, 0, 1), (1, 0, 0)),  // top
	]
}

pub fn unit_itetra_faces(mat: MatID) -> SmallVec<[Face; 6]> {
	smallvec![
		Face::rectangle(mat, (1, 0, 0), (1, 1, 0), (1, 1, 1), (1, 0, 1)), // right
		Face::rectangle(mat, (1, 0, 1), (1, 1, 1), (0, 1, 1), (0, 0, 1)), // front
		Face::rectangle(mat, (1, 1, 1), (1, 1, 0), (0, 1, 0), (0, 1, 1)), // top
		Face::triangle(mat, (0, 0, 1), (0, 1, 1), (0, 1, 0)),             // left
		Face::triangle(mat, (1, 0, 0), (1, 0, 1), (0, 0, 1)),             // bottom
		Face::triangle(mat, (0, 1, 0), (1, 1, 0), (1, 0, 0)),             // back
		Face::triangle(mat, (1, 0, 0), (0, 0, 1), (0, 1, 0)),             // diagonal
	]
}
