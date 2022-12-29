use super::internal::*;

pub struct GameCtx {
	pub assets: AssetsDir,
	pub graphics: Arc<GraphicsCtx>,
	pub resources: ResourcePack,
	pub sound_pack: SoundPack,
}

impl GameCtx {
	pub fn new(ctx: &Arc<GraphicsCtx>, settings: Settings) -> Result<Self> {
		let assets = AssetsDir::find()?;
		let resources = ResourcePack::new(ctx, assets.clone())?;
		let sound_pack = SoundPack::new(&settings.sound, assets.clone())?;

		Ok(Self {
			assets,
			resources,
			graphics: ctx.clone(),
			sound_pack,
		})
	}
}
