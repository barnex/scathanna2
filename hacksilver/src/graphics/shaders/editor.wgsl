// Vertex shader

struct Globals {
    view_proj: mat4x4<f32>,
    cam_position: vec3<f32>,
    sun_dir: vec3<f32>,
    sun_color: vec3<f32>,
};

@group(1)  @binding(0)
var<uniform> globals: Globals;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    // unused:
    @location(3) lm_coords: vec2<f32>,
    @location(4) tangent_u: vec3<f32>,
    @location(5) tangent_v: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.normal = model.normal;
    out.world_pos = model.position;
    out.clip_position = globals.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;

// A function of world position that looks like an axis-aligned grid with spacing 1.
fn grid_intens(world_pos: vec3<f32>) -> f32 {
    var GRID_WIDTH: f32 = 0.06;
    var x = world_pos.x + 2048.0 + GRID_WIDTH / 2.0;
    var y = world_pos.y + 2048.0 + GRID_WIDTH / 2.0;
    var z = world_pos.z + 2048.0 + GRID_WIDTH / 2.0;
    // Note: offset 2048 to sidestep behavior of '%' around 0.
    var grid_intens: f32 = 1.1;
    if (x % 1.0 < GRID_WIDTH) {
        grid_intens = grid_intens - 0.2;
    }
    if (y % 1.0 < GRID_WIDTH) {
        grid_intens = grid_intens - 0.2;
    }
    if (z % 1.0 < GRID_WIDTH) {
        grid_intens = grid_intens - 0.2;
    }
    return clamp(grid_intens, 0.0, 1.0);
}

// A fake ambient lighthing function.
// Gives every face of a cube a distinct intensity
// so that they are all discernible.
fn ambient_intens(normal: vec3<f32>) -> f32 {
    var sun = normalize(vec3<f32>(0.2, 0.6, 0.4));
    return (0.5 + 0.5 * dot(sun, normalize(normal)));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex = textureSample(t_diffuse, s_diffuse, in.tex_coords).xyz;
    var light = ambient_intens(in.normal) * grid_intens(in.world_pos);
    return vec4<f32>(light * tex, 1.0);
}