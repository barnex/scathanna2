use super::internal::*;

/// Asset loader and cache.
pub struct ResourcePack {
	ctx: Arc<GraphicsCtx>,

	//pub textures: Arc<TexturePack>, // shared with materials
	//pub models: ModelPack,
	pub effects: EffectPack,
	pub animations: AnimationPack,
	pub materials: Arc<MaterialPack>, // shared with async baking. TODO: remove once maps load pre-baked lightmaps
}

impl ResourcePack {
	pub fn new(ctx: &Arc<GraphicsCtx>, assets: AssetsDir) -> Result<Self> {
		//let model_pack = ModelPack::new(ctx, &assets)?;
		let effect_pack = EffectPack::new(ctx, &assets)?;
		let animation_pack = AnimationPack::new(ctx, &assets)?;
		let material_pack = Arc::new(MaterialPack::new(ctx, assets.clone())?);

		Ok(Self {
			ctx: ctx.clone(),
			//models: model_pack,
			effects: effect_pack,
			animations: animation_pack,
			materials: material_pack,
		})
	}

	pub fn ctx(&self) -> &GraphicsCtx {
		&self.ctx
	}
}
