use super::internal::*;
//use crate::img::*;
use rand::prelude::*;

/// Bake: calculate lightmap snippets for a Map's Faces.
///
/// TODO: No need to use Block::inside for testing sample point validity.
/// Sample is invalid if first intersection has negative surface normal.
/// Use the same logic for testing "inside" for physics.
/// Remove Block::inside.
/// Enables arbitrary blender models to be used as models.
pub fn bake_raytraced(opts: &BakeOpts, materials: &Arc<MaterialPack>, md: &MapData, faces: &[Face], cancel: Cancel) -> Vec<Snippet2> {
	Scene::new(opts, materials, md, faces, cancel).bake()
}

// types to increase readability
pub(super) type Color = vec3;
pub(super) type ID = usize;
pub(super) type UV = vec2;
pub(super) type Normal = vec3;

pub struct Scene {
	pub opts: BakeOpts,
	pub faces: Vec<Face>,
	pub xfilter: XFilter,
	pub face_tree: Node<(Face, ID)>,
	pub block_tree: Node<Block>,
	pub sun_color: Color,
	pub sky_color: Color,
	pub from_sun: vec3, // light coming out of sun
	pub materials: Arc<MaterialPack>,
	pub palette: Palette,
	cancel: Cancel,
}

impl Scene {
	pub fn new(opts: &BakeOpts, materials: &Arc<MaterialPack>, md: &MapData, faces: &[Face], cancel: Cancel) -> Self {
		let xfilter = XFilter::new(2, faces.iter().map(|face| (face.clone(), lightmap_size_no_margin(&opts, face))));

		Self {
			opts: opts.clone(),
			faces: faces.to_owned(),
			xfilter,
			face_tree: Node::build_tree(faces.iter().enumerate().map(|(id, f)| (f.clone(), id)).collect()),
			block_tree: Node::build_tree(md.blocks().collect()),
			from_sun: md.meta.sun_dir,
			sun_color: md.meta.sun_color,
			sky_color: md.meta.sky_color,
			materials: materials.clone(),
			palette: md.palette.clone(),
			cancel,
		}
	}
}

impl Scene {
	fn bake(&self) -> Vec<Snippet2> {
		let opts = &self.opts;

		//let test = false;
		//if test {
		//	let vis = self.bake_faces("black", &Self::xsample_black, 1);
		//	let amb = self.bake_faces(
		//		"pattern",
		//		&|_, _, p, _| {
		//			let (x, y, z) = p.into();
		//			vec3(
		//				f32::sin(0.7 * x + 0.2 * y + 0.05 * z) + f32::cos(0.9 * x + 1.2 * y + 0.75 * z), //
		//				f32::cos(0.3 * x + 0.8 * y + 0.04 * z) + f32::sin(1.3 * x + 0.8 * y + 1.02 * z),
		//				f32::sin(0.3 * x + 0.4 * y + 0.92 * z) + f32::cos(1.1 * x + 0.2 * y + 0.49 * z),
		//			) / 2.0 + vec3(0.25, 0.25, 0.25)
		//		},
		//		1,
		//	);
		//	return self.make_snippets(&vis, &amb);
		//}

		const BEST: f32 = 1e-6;
		const WORST: f32 = 1e6;
		let black = || self.bake_faces("black", &Self::xsample_black, 1, BEST);

		let sun_visibility = self.bake_faces("sun", &Scene::xsample_sun_visibility, opts.lightmap_lamps_samples, BEST);
		let sun_visibility = self.stitch(sun_visibility);
		let sun_visibility = self.smooth(sun_visibility, opts.lightmap_blur_sun);
		let sun_light = self.visibility_to_light(&sun_visibility);
		if self.opts.lightmap_sun_only {
			return self.make_snippets(&black(), &sun_light);
			//return self.make_snippets(&sun_visibility, &black());
		}

		let sky_light = self.bake_faces("sky", &Scene::xsample_sky, opts.lightmap_sky_samples, opts.lightmap_error);
		if self.opts.lightmap_sky_only {
			let sky_light = self.stitch(sky_light);
			return self.make_snippets(&black(), &sky_light);
		}

		let emission_light = self.xbake_all_emission();
		let direct_light = BorderedImg::sum_3(&sun_light, &sky_light, &emission_light).collect::<Vec<_>>();

		let scattered_depth_1 = self.bake_faces("indirect 1", &|s, r, p, n| s.xsample_indirect(r, &direct_light, p, n), 10, WORST);
		//let scattered_depth_1 = //smudge(img)
		let with_ambient_depth_1 = BorderedImg::sum_2(&direct_light, &scattered_depth_1).collect::<Vec<_>>();

		let scattered_depth_2 = self.bake_faces("indirect 2", &|s, r, p, n| s.xsample_indirect(r, &with_ambient_depth_1, p, n), 20, WORST);
		let with_ambient_depth_2 = BorderedImg::sum_2(&direct_light, &scattered_depth_2).collect::<Vec<_>>();

		let scattered_depth_3 = self.bake_faces(
			"indirect 3",
			&|s, r, p, n| s.xsample_indirect(r, &with_ambient_depth_2, p, n),
			opts.lightmap_indirect_samples,
			opts.lightmap_error,
		);
		let with_ambient_depth_3 = BorderedImg::sum_2(&direct_light, &scattered_depth_3).collect::<Vec<_>>();

		let with_ambient_depth_3 = self.stitch(with_ambient_depth_3);
		let with_ambient_depth_3 = self.smooth(with_ambient_depth_3, opts.lightmap_blur_all);

		self.make_snippets(&sun_visibility, &with_ambient_depth_3)
	}

	fn smooth(&self, imgs: Vec<BorderedImg>, n: u32) -> Vec<BorderedImg> {
		let mut imgs = imgs;
		for _ in 0..n {
			imgs = self.xfilter.blur(&imgs);
		}
		imgs
	}

	fn stitch(&self, snips: Vec<BorderedImg>) -> Vec<BorderedImg> {
		if self.opts.lightmap_stitch {
			self.xfilter.stitch(&snips)
		} else {
			snips
		}
	}

	fn bake_faces<F>(&self, msg: &str, sample: &F, max_samples: u32, target_error: f32) -> Vec<BorderedImg>
	where
		F: Fn(&Scene, &HaltonSeq, vec3, Normal) -> Color + Send + Sync,
	{
		let msg = format!("baking {msg}");
		self.faces
			.par_iter()
			.map(|face| self.bake_face(face, sample, max_samples, target_error))
			.inspect(Self::log_progress(&msg, self.faces.len()))
			.collect::<Vec<_>>()
	}

	fn bake_face<F>(&self, face: &Face, sample: &F, max_samples: u32, target_error: f32) -> BorderedImg
	where
		F: Fn(&Scene, &HaltonSeq, vec3, Normal) -> Color + Send + Sync,
	{
		if self.cancel.is_canceled() {
			return BorderedImg::default(); // unused, for canceled return only
		}

		Integrator::new(face, &self.opts, max_samples, target_error).bake(&self, sample)
	}

	fn xbake_all_emission(&self) -> Vec<BorderedImg> {
		self //
			.faces
			.iter()
			.enumerate()
			.map(|(id, face)| self.xbake_emission(id, face))
			.collect()
	}

	fn xbake_emission(&self, face_id: ID, face: &Face) -> BorderedImg {
		let mut img = make_image_for_face(&self.opts, face);
		let inner_size = img.inner_size();

		let emissive = self.emission_for(face_id) * 8.0; // TODO
		for (ix, iy) in cross(0..inner_size.x(), 0..inner_size.y()) {
			let pix = uvec2(ix, iy);
			img.set_inner_idx(pix, emissive);
		}

		img
	}

	// Fraction of sun visible from `pos`.
	// Not a light intensity (not cos weighted).
	fn xsample_sun_visibility(&self, _rnd: &HaltonSeq, pos: vec3, normal: vec3) -> Color {
		let cos_theta = -self.from_sun.dot(normal);

		// Early return for backlit face.
		if cos_theta <= 0.0 {
			return vec3::ZERO;
		}

		// ray-trace shadow
		let ray = Ray32::new(pos, -self.from_sun);
		if self.face_tree.intersects(&ray) {
			vec3::ZERO
		} else {
			self.sun_color // Note: no cos_theta, done in shader taking into account normal map.
		}
	}

	fn xsample_sky(&self, rnd: &HaltonSeq, pos: vec3, normal: vec3) -> Color {
		let dir = cosine_sphere(rnd.halton23(), normal);
		let ray = Ray::new(pos, dir);

		if self.face_tree.intersects(&ray) {
			Color::ZERO
		} else {
			self.sky_color
		}
	}

	fn xsample_indirect(&self, rnd: &HaltonSeq, direct: &[BorderedImg], pos: vec3, normal: vec3) -> Color {
		let dir = cosine_sphere(rnd.halton23(), normal);
		let ray = Ray::new(pos, dir);

		let hr = self.face_tree.intersection(&ray);
		match hr.attrib {
			None => Color::ZERO, //sky already accounted for.
			Some((normal, uv, face_id)) => {
				if normal.dot(ray.dir) >= 0.0 {
					// Ray hits face from the back, which is not illuminated
					// (unless we were to add translucent faces).
					Color::ZERO
				} else {
					//let emission = self.emission_for(face_id);
					let reflectivity = self.reflectivity_for(face_id);
					self.opts.lightmap_reflectivity * (reflectivity * direct[face_id].at_uv_no_margin(uv))
					// + emission
				}
			}
		}
	}

	// Sun visibility to amount of light (weigh by cos theta).
	fn visibility_to_light(&self, vis: &[BorderedImg]) -> Vec<BorderedImg> {
		self //
			.faces
			.iter()
			.zip(vis.iter())
			.map(|(face, vis)| {
				let cos_th = f32::max(0.0, face.normalized_normal().dot(-self.from_sun));
				BorderedImg::from_img_with_margin(vis.img_with_margin().map_values(|color| cos_th * color))
			})
			.collect()
	}

	// DEBUG: a fake shader that colors points white if not occluded (valid sample point for lightmap)
	// or red if fully occluded (not valid to sample for lightmap, should extrapolate from neighbors instead
	// to avoid shadow bleeding).
	fn xsample_validity(&self, _rnd: &HaltonSeq, pos: vec3, _normal: vec3) -> Color {
		if is_valid_sampling_point(&self, pos) {
			Color::ONES
		} else {
			vec3(1.0, 0.0, 0.0)
		}
	}

	// DEBUG: used to disable lightmap contributions.
	fn xsample_black(&self, _rnd: &HaltonSeq, _pos: vec3, _normal: vec3) -> Color {
		vec3::ZERO
	}

	// ================================================================================ Face access

	fn emission_for(&self, face_id: ID) -> Color {
		let face = &self.faces[face_id];
		self.palette
			.material_name_for(face.mat) //
			.map(|name| self.materials.get_host(name))
			.map(|mat| mat.avg_emissive)
			.unwrap_or_default()
			* 10.0 // TODO remove
	}

	fn reflectivity_for(&self, face_id: ID) -> Color {
		let face = &self.faces[face_id];
		self.palette
			.material_name_for(face.mat) //
			.map(|name| self.materials.get_host(name))
			.map(|mat| mat.avg_diffuse)
			.unwrap_or_default()
	}

	pub(super) fn shading_normal_for(&self, face: &Face, face_uv: vec2) -> vec3 {
		match self.opts.lightmap_bake_normals {
			true => self._shading_normal_for(face, face_uv).unwrap_or_else(|| face.normalized_normal()),
			false => face.normalized_normal(),
		}
	}

	fn _shading_normal_for(&self, face: &Face, face_uv: vec2) -> Option<vec3> {
		let material = self.palette.host_material_for(&self.materials, face.mat)?;
		let texcoords = Self::texcoords_for(face, face_uv);
		//let face_uv = ();
		let normal_vecs = material.normal_vec.as_ref()?;
		let nm = normal_vecs.at_uv_nearest_wrap(texcoords);
		let uv_basis = uv_basis(face);
		let Matrix3([tangent_u, tangent_v, geom_normal]) = uv_basis;
		Some(nm.x() * tangent_u + nm.y() * tangent_v + nm.z() * geom_normal)
	}

	fn texcoords_for(face: &Face, face_uv: vec2) -> vec2 {
		let uv_basis = uv_basis(face);
		let vertex_uvs = face //
			.vertices()
			.into_iter()
			.map(|pos| uv_project(&uv_basis, pos.to_f32()))
			.collect::<SmallVec<[vec2; 4]>>();

		let (u, v) = face_uv.into();
		u * vertex_uvs[0] //in tangent direction
				+ v * vertex_uvs[2]  // in bitangent direction
				+ (1.0 -u -v) * vertex_uvs[1] // at origin
	}

	fn make_snippets(&self, direct_visibility: &[BorderedImg], ambient: &[BorderedImg]) -> Vec<Snippet2> {
		self.faces //
			.iter()
			.cloned()
			.zip(direct_visibility.iter().map(|img| img.img_with_margin().to_srgb()))
			.zip(ambient.iter().map(|img| img.img_with_margin().to_srgb()))
			.map(|((face, direct_visibility), ambient)| Snippet2 {
				face: face.clone(),
				direct_visibility,
				ambient,
			})
			.collect::<Vec<_>>()
	}

	// Increment counter and log progress towards total on each invocation.
	// To be used with Iterator::inspect.
	fn log_progress<T>(msg: &str, total: usize) -> impl Fn(&T) + Send + Sync {
		LOG.write(format!("{msg}..."));
		let count = Box::new(Counter::new());
		let msg = msg.to_owned();
		Box::new(move |_: &T| {
			let count = count.inc();
			let pct = (((100 * count) as f32) / ((total) as f32)).ceil() as u32;
			LOG.replace_last_line(format!("{msg} {count}/{total} {pct}%"));
		})
	}
}

pub struct HaltonSeq {
	pub scramble: vec2,
	pub halton_i: u32,
}

impl HaltonSeq {
	pub fn new(scramble_u: f32, scramble_v: f32) -> Self {
		Self {
			scramble: vec2(scramble_u, scramble_v),
			halton_i: 0,
		}
	}

	pub fn halton23(&self) -> vec2 {
		halton23_scrambled(self.halton_i, self.scramble.into()).into()
	}
}

// How big should the lightmap image of a Face be?
// Baking resolution * face physical size (orientation-independent) + 1.
//
// Note: the +1 is so that a 1x1 face (the smallest face), gets a 2x2
// lightmap image -- mapping each vertex to a distinct pixel. This is
// the minimum resolution needed to get smooth shading across the face.
//
// E.g.: 1x1 face, 2x2 lightmap:
//    . . . ... . . .
//    .      .      .
//    .  +-------+  .
//    .  |   .   |  .
//    . .|. ... .|. .
//    .  |   .   |  .
//    .  +-------+  .
//    .      .      .
//    . . . ... . . .
//
pub(super) fn lightmap_size_no_margin(opts: &BakeOpts, face: &Face) -> uvec2 {
	face //
		.sized_tangents()
		.map(|n| (n * opts.lightmap_resolution as i32).to_f32().len().round() as u32 + 1)
		.into()
}

#[cfg(test)]
mod test {
	use super::*;

	// in editor?

	//	fn test_x_filter(){
	//		let scene = Scene{};
	//		let faces = vec![];
	//		let imgs = faces.iter().map(|face|)
	//
	//
	//	}

	#[test]
	fn test_face_fragments() {
		//
		//         0       1/3    2/3     3/3
		//     0/2 +---+---+---+---+---+---+
		//         |   |       |       |   |
		//   0.5/2 +---+-------+-------+---+
		//         |   |       |       |   |
		//         |   |       |       |   |
		//   1.5/2 +---+-------+-------+---+
		//         |   |       |       |   |
		//     2/2 +---+-------+-------+---+

		let inner_size = uvec2(4, 3);
		let f = clamped_face_fragments(inner_size)
			.map(|(pix, range)| ((pix.x(), pix.y()), (range.min.tuple(), range.max.tuple())))
			.collect::<HashMap<_, _>>();
		assert_eq!(f[&(0, 0)], ((0.0 / 3.0, 0.0 / 2.0), (0.5 / 3.0, 0.5 / 2.0)));
		assert_eq!(f[&(3, 2)], ((2.5 / 3.0, 1.5 / 2.0), (3.0 / 3.0, 2.0 / 2.0)));
	}
}

// Image (including margin) for face.
// Margin is colored in (for debug);
pub(super) fn make_image_for_face(opts: &BakeOpts, face: &Face) -> BorderedImg {
	let inner_size = lightmap_size_no_margin(opts, face);
	let img = BorderedImg::new_with_inner_size(inner_size);
	let img = match opts.lightmap_outline {
		false => img,
		true => img.with(|img| img.draw_margin()),
	};
	img
}
