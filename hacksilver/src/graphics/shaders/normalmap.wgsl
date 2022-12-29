// Vertex shader

struct Globals {
    view_proj: mat4x4<f32>,
    cam_position: vec3<f32>,
    sun_dir: vec3<f32>,
    sun_color: vec3<f32>,
};

@group(1) @binding(0)
var<uniform> globals: Globals;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) lm_coords: vec2<f32>,
    @location(4) tangent_u: vec3<f32>,
    @location(5) tangent_v: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) lm_coords: vec2<f32>,
    @location(3) normal: vec3<f32>,
    @location(4) tangent_u: vec3<f32>,
    @location(5) tangent_v: vec3<f32>,
    @location(6) world_position: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.lm_coords = model.lm_coords;
    out.clip_position = globals.view_proj * vec4<f32>(model.position, 1.0);
    out.normal = model.normal;
    out.tangent_u = model.tangent_u;
    out.tangent_v = model.tangent_v;
    out.world_position = model.position;
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;

@group(0) @binding(2)
var t_lightmap: texture_2d<f32>;

@group(0) @binding(3)
var s_lightmap: sampler;

@group(0) @binding(4)
var t_normalmap: texture_2d<f32>;

@group(0) @binding(5)
var s_normalmap: sampler;

@group(0) @binding(6)
var t_direct: texture_2d<f32>;

@group(0) @binding(7)
var s_direct: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    let diffuse_tex = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let ambient_tex = textureSample(t_lightmap, s_lightmap, in.lm_coords);
    let nm = textureSample(t_normalmap, s_normalmap, in.tex_coords).xyz;
    let direct_tex = textureSample(t_direct, s_direct, in.lm_coords);

    let ambient_light = diffuse_tex * ambient_tex;
    // https://en.wikipedia.org/wiki/Normal_mapping#Calculation
    let nm = 2.0 * (nm - 0.5);
    let normal = normalize(nm.x * in.tangent_u + nm.y * in.tangent_v + nm.z * in.normal);

    //  //let debug = (in.tangent_u + in.tangent_v) / 2.0;
    //  //let debug = (debug + 1.0 / 2.0);
    //  //return vec4(debug, 0.0);


	// //  https://en.wikipedia.org/wiki/Phong_reflection_model
    let to_sun = normalize(vec3<f32>(-0.25916052, 0.8638684, -0.4319342));             // TODO !!!!!!!!!!!!!!
    //let to_sun = normalize(vec3<f32>(1.0, 1.0, 0.0));             // TODO !!!!!!!!!!!!!!
    let cos_theta = max(0.0, dot(to_sun, normal));

    //return vec4(cos_theta, cos_theta, cos_theta, 1.0);
    let sun_intens = direct_tex.x; // TODO loop over up to 4 lights
    let diffuse = (sun_intens * cos_theta) * diffuse_tex;

    //return diffuse + ambient_light;

    let view_dir = normalize(globals.cam_position - in.world_position);
    let refl_dir = reflect(-to_sun, normal);
    let spec = 0.4 * sun_intens * pow(max(0.0, dot(view_dir, refl_dir)), 16.0);

    return ambient_light + diffuse + vec4(spec, spec, spec, 1.0);
}