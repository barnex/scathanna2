use super::internal::*;

/// Node in a Bounding Volume Hierarchy (BHV).
/// https://en.wikipedia.org/wiki/Bounding_volume_hierarchy
pub enum Node<T> {
	Inner([(BoundingBox32, Box<Node<T>>); 2]),
	Leaf([T; 2]),
}

impl<T> Node<T>
where
	T: Clone + Default + Bounded,
{
	pub fn build_tree(nodes: Vec<T>) -> Self {
		if nodes.len() <= 2 {
			Self::build_leaf(nodes)
		} else {
			Self::build_inner(nodes)
		}
	}

	fn build_leaf(nodes: Vec<T>) -> Self {
		debug_assert!(nodes.len() <= 2 && nodes.len() >= 1);
		// A zero-sized (Default) Face is used in the BVH tree when a node only has 1 real child.
		// Instead of setting the other child to None, we use a zero-sized face
		// (which can never intersect a Ray). There is at most one such child because
		// we build the tree to be complete (https://en.wikipedia.org/wiki/Binary_tree#Types_of_binary_trees).
		Node::Leaf([nodes[0].clone(), nodes.get(1).cloned().unwrap_or_default()])
	}

	fn build_inner(nodes: Vec<T>) -> Self {
		debug_assert!(nodes.len() > 2);

		// split along the longest direction,
		// to end up with roughly cubical children.

		let hull = BoundingBox32::union(nodes.iter().map(T::bounds)).expect("nodes not empty");

		let split_dir = hull.size().argmax();

		let mut nodes = nodes;
		nodes.sort_by_key(|n| (n.bounds().center()[split_dir] * 1024.0) as i32 /*work around partial ord*/);

		let (left, right) = Self::split(nodes);
		let bb_left = BoundingBox32::union(left.iter().map(T::bounds)).unwrap();
		let bb_right = BoundingBox32::union(right.iter().map(T::bounds)).unwrap();

		let left = Box::new(Node::build_tree(left));
		let right = Box::new(Node::build_tree(right));

		Node::Inner([(bb_left, left), (bb_right, right)])
	}

	fn split(nodes: Vec<T>) -> (Vec<T>, Vec<T>) {
		let at = nodes.len() / 2;
		let mut nodes = nodes;
		let right = nodes.split_off(at);
		(nodes, right)
	}
}

//-------------------------------------------------------------------------------- Intersect

/// A Node of things that can intersect, can intersect itself.
/// E.g., we can intersect with a Node of Faces.
impl<T, A> Intersect for Node<T>
where
	T: Intersect<Attrib = A>,
	A: Clone,
{
	type Attrib = T::Attrib;
	fn intersect(&self, r: &Ray32, hr: &mut HitRecord<f32, A>) -> bool {
		match self {
			Node::Inner(ch) => Self::intersect_inner(ch, r, hr),
			Node::Leaf(ch) => Self::intersect_leaf(ch, r, hr),
		}
	}

	//fn intersects(&self, r: &Ray<f32>) -> bool {
	//
	//}
}

impl<T, A> Node<T>
where
	T: Intersect<Attrib = A>,
	A: Clone,
{
	fn intersect_inner(ch: &[(BoundingBox32, Box<Self>); 2], r: &Ray32, hr: &mut HitRecord<f32, A>) -> bool {
		let mut hit = false;
		if ch[0].0.intersects(r) {
			hit |= ch[0].1.intersect(r, hr);
		}
		if ch[1].0.intersects(r) {
			hit |= ch[1].1.intersect(r, hr);
		}
		hit
	}

	fn intersect_leaf(ch: &[T; 2], r: &Ray32, hr: &mut HitRecord<f32, A>) -> bool {
		ch[0].intersect(r, hr) | ch[1].intersect(r, hr)
	}
}

//-------------------------------------------------------------------------------- Volume

/// A node of `Volume`s is itself a `Volume`.
impl<T> Node<T>
where
	T: Volume,
{
	pub fn contains(&self, pos: vec3) -> bool {
		match self {
			Node::Inner(ch) => Self::contains_inner(ch, pos),
			Node::Leaf(ch) => Self::contains_leaf(ch, pos),
		}
	}

	fn contains_inner(ch: &[(BoundingBox32, Box<Self>); 2], pos: vec3) -> bool {
		if ch[0].0.contains(pos) {
			if ch[0].1.contains(pos) {
				// early return, no need to iterate second child
				return true;
			}
		}
		if ch[1].0.contains(pos) {
			ch[1].1.contains(pos)
		} else {
			false
		}
	}

	fn contains_leaf(ch: &[T; 2], pos: vec3) -> bool {
		ch[0].contains(pos) || ch[1].contains(pos)
	}
}
