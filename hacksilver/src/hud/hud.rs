use super::internal::*;

pub struct HUD {
	ctx: Arc<GraphicsCtx>,
	slots: [Slot; 7],
	pub crosshair: bool,
	cache: Cache<Object>,
}

#[derive(Default)]
struct Slot {
	text: String,
	ttl_secs: f32,
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
#[repr(u8)]
pub enum HUDPos {
	TopLeft = 0,
	TopRight = 1,
	BottomLeft = 2,
	BottomRight = 3,
	Center = 4,
	TopCenter = 5,
	TopCenter2 = 6,
}

use HUDPos::*;

impl HUD {
	pub fn new(ctx: &Arc<GraphicsCtx>) -> Self {
		Self {
			ctx: ctx.clone(),
			slots: default(),
			crosshair: true,
			cache: default(),
		}
	}
	pub fn apply(&mut self, upd: HUDUpdate) -> Option<()> {
		*self.slots.get_mut(upd.pos as usize)? = Slot {
			text: upd.text,
			ttl_secs: upd.ttl_sec,
		};
		self.cache.clear();
		Some(())
	}

	pub fn show_info(&mut self, text: impl Into<String>) {
		self.set_text(HUDPos::TopLeft, text, 5.0)
	}

	pub fn set_text(&mut self, pos: HUDPos, text: impl Into<String>, ttl_secs: f32) {
		let text = text.into();
		self.slots[pos as usize] = Slot { text, ttl_secs };
		self.cache.clear();
	}

	pub fn tick(&mut self, dt: f32) {
		for slot in &mut self.slots {
			if slot.ttl_secs > 0.0 {
				slot.ttl_secs -= dt;
				if slot.ttl_secs < 0.0 {
					slot.text.clear();
					self.cache.clear();
				}
			}
		}
	}

	pub fn draw_on(&self, sg: &mut SceneGraph) {
		if self.crosshair {
			self.draw_crosshair(sg);
		}

		sg.push(self.cache.clone_or(|| self.render(sg.viewport)));
	}

	fn render(&self, viewport: uvec2) -> Object {
		let mut buf = MeshBuffer::new();
		let text = |pos| &self.slots[pos as usize].text;

		buf.append(&layout_text(viewport, uvec2(0, 0), text(TopLeft)));
		buf.append(&layout_text_right(viewport, text(TopRight)));
		buf.append(&layout_text_bottom(viewport, text(BottomLeft)));

		{
			let text = text(Center);
			// some fixed-point arithmetic to get the text about 20% above the crosshairs
			let pos = (viewport * 1024) / uvec2(2 * 1024, 2 * 1024 + 512) - text_size_pix(text) / 2;
			buf.append(&layout_text(viewport, pos, text));
		}

		{
			let text = text(TopCenter);
			let pos = viewport / uvec2(2, 4) - text_size_pix(text) / 2;
			buf.append(&layout_text(viewport, pos, text));
		}

		{
			let text = text(TopCenter2);
			let pos = viewport / uvec2(2, 4) - text_size_pix(text) / 2 + uvec2(0, 2 * EMBEDDED_CHAR_SIZE.y());
			buf.append(&layout_text(viewport, pos, text));
		}

		{
			let text = text(BottomRight);
			let pos = viewport - text_size_pix(text);
			buf.append(&layout_text(viewport, pos, text));
		}

		let vao = Arc::new(self.ctx.upload_meshbuffer(&buf));
		let shader = self.ctx.shader_pack.text();

		Object::new(&vao, shader)
	}

	fn draw_crosshair(&self, sg: &mut SceneGraph) {
		let center = sg.viewport / 2 - EMBEDDED_CHAR_SIZE / 2;
		sg.push(Object::new(
			&Arc::new(self.ctx.upload_meshbuffer(&layout_text(sg.viewport, center, EMBEDDED_CROSSHAIR))),
			self.ctx.shader_pack.text(),
		));
	}
}
