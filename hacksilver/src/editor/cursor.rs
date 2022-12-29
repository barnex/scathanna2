use super::internal::*;

const MAX_LOG_CURSOR_SIZE: u8 = 6;
const MAX_CURSOR_SIZE: u8 = 1 << MAX_LOG_CURSOR_SIZE;

/// The Cursor plays a central role in the editor.
/// It provides a "template" that can be instantiated
/// with a translation (optionally aligned), rotation, re-scale.
pub struct Cursor {
	ctx: Arc<GraphicsCtx>,
	texture: Texture,

	// These block(s) will be copied into the scene on left-click.
	// The absolute position of the prototype is of no significance,
	// as the prototype is always placed at the cursor location.
	prototype: Vec<Block>,

	// Rotate the prototype before instantiating.
	rotate: Rotation,

	// Scale the prototype before instantiation.
	scale: Vector3<u8>,

	// Place the `min` corner of the prototype here before instantiating.
	// `None` means there is no current cursor position (e.g. crosshair pointing at the sky).
	abs_pos: Option<ivec3>,

	// Align cursor position to this power of two.
	log_cursor_align: u8,
}

impl Cursor {
	pub fn new(ctx: &Arc<GraphicsCtx>) -> Self {
		Self {
			ctx: ctx.clone(),
			texture: uniform_texture(ctx, vec4::ONES /*white*/),
			rotate: Rotation::UNIT,
			scale: Vector3::new(4, 4, 4),
			log_cursor_align: 2, // 2^2 == 4, a good initial cursor size/align
			abs_pos: None,
			prototype: vec![],
		}
		.with(Self::ensure_prototype)
		.with(Self::rescale)
	}

	/// Size of (an axis-aligned bounding box around) the cursor.
	pub fn size(&self) -> uvec3 {
		BoundingBox::union(self.abs_prototype().iter().map(|blk| blk.ibounds())) //
			.unwrap_or(BoundingBox::new(ivec3::ZERO, ivec3::ZERO))
			.size()
			.map(|v| v as u32)
	}

	/// Rotate the cursor shape.
	pub fn rotate(&mut self, r: Rotation) {
		// left-multiply so newly added rotation is applied after previous ones.
		self.rotate = r * self.rotate;

		// also rotate the overall cursor scaling (an anisotropic scaling gets rotated)
		let (new_scale, _) = r.rotate_bounds(self.scale);
		self.scale = new_scale;
	}

	/// Add `delta` to the overall cursor size,
	/// but never grow smaller or bigger than allowed.
	pub fn change_scale(&mut self, delta: ivec3) {
		// increment size in steps of `align` if it is already a multiple.
		let delta = if self.size().iter().all(|v| v % (self.align() as u32) == 0) {
			delta * (self.align() as i32)
		} else {
			delta
		};

		// make sure the new scale factor
		// 1) does not overflow in itself
		// 2) does not scale any of the cursor blocks beyond reasonable size
		let new_scale = (self.scale.map(|v| v as i32) + delta) //
			.map(|v| v.clamp(self.align() as i32, MAX_CURSOR_SIZE as i32))
			.map(|v| v as u32);

		let max_new_block_size = self
			.prototype //
			.iter()
			.map(|b| b.size.convert::<u32>() * new_scale)
			.map(|v| v.reduce(u32::max))
			.max()
			.unwrap_or_default();

		if new_scale.iter().all(|v| v <= MAX_CURSOR_SIZE as u32) && max_new_block_size <= MAX_CURSOR_SIZE as u32 {
			self.scale = new_scale.convert()
		}
	}

	/// Set the cursor position, or None if the cursor to be hidden.
	pub fn set_pos(&mut self, pos: Option<ivec3>) {
		self.abs_pos = pos;
	}

	/// Set the cursor to a single block of given type (shape).
	pub fn set_blocktyp(&mut self, bt: BlockTyp) {
		self.single_block().typ = bt;
	}

	/// Set the material (texture etc) for the entire cursor.
	pub fn set_material(&mut self, mat: MatID) {
		self.prototype.iter_mut().for_each(|b| b.mat = mat)
	}

	/// The cursor's current material.
	/// In the ambiguous case of a multi-material cursor,
	/// return the material of the first block.
	pub fn material(&self) -> MatID {
		self.prototype.get(0).map(|b| b.mat).unwrap_or_default()
	}

	pub fn grab1(&mut self, blk: Block) {
		*self.single_block() = blk;
		self.rescale();
	}

	pub fn add(&mut self, blk: Block) {
		self.scale = Vector3::ONES;
		self.rotate = Rotation::UNIT;
		self.prototype = self.abs_prototype();
		if !self.prototype.contains(&blk) {
			self.prototype.push(blk);
		} else {
			//self.prototype.remove(&blk);
		}
		self.rescale();
	}

	pub fn set_prototype(&mut self, blocks: impl Iterator<Item = Block>) {
		self.prototype = blocks.collect();
		self.rescale();
	}

	// After setting the cursor shape, we want to scale it down to the smallest size
	// that still has the same shape, and compensate by the scale factor.
	// This way, we can later scale the cursor down as much as possible,
	// or increment the size in the finest steps.
	//
	// E.g. if the cursor consists of 3x3x3 and 6x6x3 cuboids,
	// these will get scaled down 1x1x1 and 2x2x1 (perfectly keeping their shapes),
	// and the scale factor will be set to 3x3x3 (restoring the original size).
	// We can now resize the cursor to, e.g., 2x2x2 and 4x4x2 cuboids,
	// which would not have been possible by scaling the originals.
	fn rescale(&mut self) {
		self.scale = Vector3::ONES;
		self.rotate = Rotation::UNIT;
		// Shift prototype origin to 0,0,0
		let origin = BoundingBox::union(self.prototype.iter().map(|blk| blk.ibounds())) //
			.unwrap_or(BoundingBox::new(ivec3::ZERO, ivec3::ZERO))
			.min;
		self.prototype.iter_mut().for_each(|b| b.pos -= origin);

		// Determine how much we can scale down in each direction without changing shape:
		// In each direction, we can scale down by a factor that divides both the sizes
		// and positions of all blocks.
		let gcd_size = self
			.prototype
			.iter() //
			.map(|b| b.size)
			.reduce(|a, b| a.zip(b, gcd::binary_u8))
			.unwrap_or(Vector3::ONES)
			.map(|v| v as u32);

		let gcd_pos = self
			.prototype
			.iter() //
			.map(|b| b.pos.map(|v| v as u32))
			.reduce(|a, b| a.zip(b, gcd::binary_u32))
			.unwrap_or(Vector3::ONES);

		let gcd = gcd_size.zip(gcd_pos, gcd::binary_u32).convert();

		self.prototype.iter_mut().for_each(|b| {
			b.size = b.size / gcd;
			b.pos = b.pos / gcd.convert();
		});
		self.scale = gcd;
	}

	// Make sure the cursor is a single block, and return a reference to it.
	fn single_block(&mut self) -> &mut Block {
		self.ensure_prototype();
		self.prototype.truncate(1);
		self.scale = self.scale * self.prototype[0].size;
		self.prototype[0].size = Vector3::ONES;
		&mut self.prototype[0]
	}

	// ensure the prototype has at least 1 block.
	fn ensure_prototype(&mut self) {
		if self.prototype.len() == 0 {
			self.prototype = vec![Block {
				mat: MatID(1),
				size: Vector3::new(1, 1, 1),
				pos: ivec3::ZERO,
				rotation: Rotation::UNIT,
				typ: BlockTyp(0),
			}];
		}
	}

	// cursor should be aligned to this many voxels.
	pub fn align(&self) -> u8 {
		// clamp is defensive, should not be necessary.
		(1 << self.log_cursor_align).clamp(1, MAX_CURSOR_SIZE)
	}

	pub fn set_linear_align(&mut self, align: u8) {
		self.log_cursor_align = (align.trailing_zeros() as u8).clamp(0, MAX_LOG_CURSOR_SIZE);
	}

	pub fn draw(&self) -> SmallVec<[Object; 1]> {
		self.abs_prototype()
			.into_iter()
			.map(|blk| Object::new(&Arc::new(self.ctx.upload_meshbuffer(&block_linebuffer(&blk))), self.ctx.shader_pack.lines(&self.texture)))
			.collect()
	}

	// cursor prototype, but at absolute position
	pub fn abs_prototype(&self) -> Vec<Block> {
		let abs_pos = match self.abs_pos {
			Some(abs_pos) => abs_pos,
			None => return vec![],
		};

		let mut abs = self.prototype.iter().copied().map(move |b| self.transform_block(b)).collect::<Vec<_>>();
		let off = BoundingBox::union(abs.iter().map(|b| b.ibounds())).map(|bb| bb.min).unwrap_or_default();
		abs.iter_mut().for_each(|b| b.pos = b.pos + abs_pos - off);
		abs
	}

	fn transform_block(&self, b: Block) -> Block {
		//let r = self.rotate.matrix();

		let (new_size, pos_offset) = self.rotate.rotate_bounds(b.size);
		Block {
			rotation: self.rotate * b.rotation,
			size: new_size * self.scale,
			pos: (self.scale.map(|v| v as i32) * (self.rotate.rotate_pos(b.pos) + pos_offset)),
			..b
		}
	}
}

fn block_linebuffer(blk: &Block) -> MeshBuffer {
	let mut buf = MeshBuffer::new();
	for face in blk.faces() {
		buf.append(&face_linebuffer(&face));
	}
	buf
}
