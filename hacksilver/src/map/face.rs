use super::internal::*;

/// A Rectangular or Triangular shape + material ID.
#[derive(Clone, Default)]
pub struct Face {
	// Vertex ordering has special meaning:
	//
	//   v0: v1+tangent1
	//    ^        * (v3 not stored)
	//    |
	//    |
	//    +--------> v2: v1+tangent2
	//    v1:origin
	//
	vert: [ivec3; 3],
	pub shape: FaceShape,
	pub mat: MatID,
}

#[derive(Clone, Copy)]
pub enum FaceShape {
	Rect,
	Tri,
}

impl Face {
	/// Construct a rectangle with given vertices.
	/// Winding order:
	///
	/// v[0]      v[3]
	///     +-----+
	///     |     |
	///     +-----+
	/// v[1]      v[2]
	///
	pub fn rectangle<V: Into<ivec3>>(mat: MatID, v0: V, v1: V, v2: V, v3: V) -> Self {
		let v = [v0.into(), v1.into(), v2.into()];
		let v3 = v3.into();

		debug_assert!({
			let o = v[1];
			let a = v[0] - o;
			let b = v[2] - o;
			if v3 != o + a + b {
				dbg!(v[0], v[1], v[2], v3, o, a, b, o + a + b);
			}
			v3 == o + a + b
		});

		Self { mat, vert: v, shape: FaceShape::Rect }
	}

	/// Construct a triangle with given vertices.
	/// Winding order:
	///
	/// v[0]
	///     +
	///     | \
	///     +--+
	/// v[1]    v[2]
	///
	pub fn triangle<V: Into<ivec3>>(mat: MatID, v0: V, v1: V, v2: V) -> Self {
		let v = [v0.into(), v1.into(), v2.into()];
		Self { mat, vert: v, shape: FaceShape::Tri }
	}

	pub fn origin(&self) -> ivec3 {
		self.vert[1]
	}

	/// 3D position corresponding to a UV coordinate inside the face.
	#[inline]
	pub fn pos_for_uv(&self, uv: vec2) -> vec3 {
		let o = self.origin().to_f32();
		let [a, b] = self.sized_tangents().map(ivec3::to_f32);
		let (u, v) = uv.into();
		o + (u * a) + (v * b)
	}

	/// Normal vector, not scaled to unit length.
	pub fn sized_normal(&self) -> ivec3 {
		let a = self.vert[1] - self.vert[0];
		let b = self.vert[2] - self.vert[0];
		a.cross(b) // TODO: check handedness
	}

	/// Normal vector, scaled to unit length.
	pub fn normalized_normal(&self) -> vec3 {
		self.sized_normal().to_f32().normalized()
	}

	/// Tangent vectors, not scaled to unit length.
	pub fn sized_tangents(&self) -> [ivec3; 2] {
		[self.vert[0] - self.vert[1], self.vert[2] - self.vert[1]]
	}

	/// Tangent vectors, scaled to unit length.
	pub fn normalized_tangents(&self) -> [vec3; 2] {
		self.sized_tangents().map(|v| v.to_f32().normalized())
	}

	/// A copy with all vertices translated by `delta`.
	#[must_use = "does not alter self"]
	pub fn translated(&self, delta: ivec3) -> Self {
		self.map_positions(|p| p + delta)
	}

	pub fn translate(&mut self, delta: ivec3) {
		self.foreach_position(|p| p + delta)
	}

	pub fn scale(&mut self, scale: uvec3) {
		self.foreach_position(|p| p.mul3(scale.as_ivec()))
	}

	/// A copy of with function `f` applied to all vertex positions.
	#[must_use = "does not alter self"]
	pub fn map_positions<F>(&self, f: F) -> Self
	where
		F: Fn(ivec3) -> ivec3,
	{
		let mut v = self.vert.clone();
		v.iter_mut().for_each(|v| *v = f(*v));
		Self { vert: v, ..*self }
	}

	pub fn foreach_position<F>(&mut self, f: F)
	where
		F: Fn(ivec3) -> ivec3,
	{
		self.vert.iter_mut().for_each(|ptr| *ptr = f(*ptr))
	}

	pub fn vertices(&self) -> SmallVec<[ivec3; 4]> {
		let o = self.vert[1];
		let a = self.vert[0] - o;
		let b = self.vert[2] - o;
		let d = o + a + b;
		match self.shape {
			FaceShape::Rect => smallvec![self.vert[0], self.vert[1], self.vert[2], d],
			FaceShape::Tri => smallvec![self.vert[0], self.vert[1], self.vert[2]],
		}
	}
}

//-------------------------------------------------------------------------------- ray tracing

impl IBounded for Face {
	fn ibounds(&self) -> BoundingBox<i32> {
		BoundingBox::from_points(self.vertices().iter().copied()).expect("face has vertices")
	}
}

impl Intersect for Face {
	type Attrib = (vec3, vec2, MatID); // normal, UV, material

	#[inline]
	fn intersect(&self, r: &Ray32, hr: &mut HitRecord<f32, Self::Attrib>) -> bool {
		match self.shape {
			FaceShape::Tri => self.intersect_triangle(r, hr),
			FaceShape::Rect => self.intersect_rectangle(r, hr),
		}
	}
}

impl Face {
	#[inline]
	fn intersect_triangle(&self, r: &Ray32, hr: &mut HitRecord<f32, (vec3, vec2, MatID)>) -> bool {
		let o = self.origin().to_f32();
		let [a, b] = self.sized_tangents().map(ivec3::to_f32);

		let dir = r.dir;

		let n = a.cross(b);

		let s = r.start - o;
		let t = -n.dot(s) / n.dot(dir);
		//let n2 = n.dot(n);

		// handles NaN gracefully
		if !(t > 0.0 && t < hr.t) {
			return false;
		}

		let p = r.at(t) - o;
		// TODO: s + r.dir * t;

		// Barycentric coordinates for 3D triangle, after
		// Peter Shirley, Fundamentals of Computer Graphics, 2nd Edition.
		let nc = a.cross(p);
		let na = (b - a).cross(p - a);
		let n2 = n.dot(n);
		let l1 = n.dot(na) / n2;
		let l3 = n.dot(nc) / n2;
		let l2 = 1.0 - l1 - l3;

		let inside = f32::partial_min(f32::partial_min(l1, l2), l3) > 0.0;

		if inside {
			// TODO: check if l1, l2 are the correct barycentric coordinates!
			hr.record(t, &(self.normalized_normal(), vec2(l1, l2), self.mat))
		}

		inside
	}

	#[inline]
	fn intersect_rectangle(&self, r: &Ray32, hr: &mut HitRecord<f32, (vec3, vec2, MatID)>) -> bool {
		let o = self.origin().to_f32();
		let [a, b] = self.sized_tangents().map(ivec3::to_f32);

		let dir = r.dir;

		let n = a.cross(b);

		let s = r.start - o;
		let t = -n.dot(s) / n.dot(dir);
		//let n2 = n.dot(n);

		// handles NaN gracefully
		if !(t > 0.0 && t < hr.t) {
			return false;
		}

		let p = r.at(t) - o;
		// TODO: s + r.dir * t;

		let pa = p.dot(a);
		let pb = p.dot(b);
		let a2 = a.dot(a);
		let b2 = b.dot(b);

		let inside = pa >= 0.0 && pb >= 0.0 && pa <= a2 && pb <= b2;

		if inside {
			let u = pa / a2;
			let v = pb / b2;
			// TODO: don't record normal, get from ID
			hr.record(t, &(self.normalized_normal(), vec2(u, v), self.mat))
		}

		inside
	}
}

// Meshbuffer for a face, drawn as lines instead of triangles.
// Unused light/texture coordinates.
pub fn face_linebuffer(face: &Face) -> MeshBuffer {
	let normal = face.sized_normal().to_f32().normalized();

	let o = VertexLM {
		position: face.origin().to_f32(),
		normal,
		..default()
	};

	let [n1, n2] = face.sized_tangents();

	let a = VertexLM {
		position: o.position + n1.to_f32(),
		normal,
		..default()
	};

	let b = VertexLM {
		position: o.position + n2.to_f32(),
		normal,
		..default()
	};

	let c = VertexLM {
		position: o.position + n1.to_f32() + n2.to_f32(),
		normal,
		..default()
	};

	match face.shape {
		FaceShape::Rect => MeshBuffer {
			vertices: vec![a, o, b, c],
			indices: vec![0, 1, 1, 2, 2, 3, 3, 0],
		},
		FaceShape::Tri => MeshBuffer {
			vertices: vec![a, o, b],
			indices: vec![0, 1, 1, 2, 2, 0],
		},
	}
}

impl Default for FaceShape {
	fn default() -> Self {
		Self::Tri
	}
}

#[cfg(test)]
mod test {
	use super::*;

	/*

			 * (3,4)
			/|
		   / |
		  /  |
	(1,2)*---* (3,2)

	*/
	#[test]
	fn intersects() {
		let t = Face::triangle(MatID(0), (1, 2, -1), (3, 2, -1), (3, 4, -1));
		let ez = vec3::EZ;

		assert!(!t.intersects(&Ray::new(vec3(0., 0., 0.,), -ez)));
		assert!(!t.intersects(&Ray::new(vec3(0., 0., 0.,), ez)));
		assert!(t.intersects(&Ray::new(vec3(2., 3., 0.,), -ez)));
		assert!(!t.intersects(&Ray::new(vec3(2., 3., 0.,), ez)));
		assert!(!t.intersects(&Ray::new(vec3(4., 3., 0.,), -ez)));
		assert!(!t.intersects(&Ray::new(vec3(4., 3., 0.,), ez)));
		assert!(!t.intersects(&Ray::new(vec3(2., -3., 0.,), -ez)));
		assert!(!t.intersects(&Ray::new(vec3(2., -3., 0.,), ez)));
		assert!(!t.intersects(&Ray::new(vec3(0., 0., -2.,), ez)));
		assert!(!t.intersects(&Ray::new(vec3(0., 0., -2.,), -ez)));
		assert!(t.intersects(&Ray::new(vec3(2., 3., -2.,), ez)));
		assert!(!t.intersects(&Ray::new(vec3(2., 3., -2.,), -ez)));
	}
}
