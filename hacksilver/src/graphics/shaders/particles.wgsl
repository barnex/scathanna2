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
    // See wgpu::VertexBufferLayout in vertex.rs
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) velocity: vec3<f32>, // stored as normal
    // unused:
    @location(3) lm_coords: vec2<f32>,
    @location(4) tangent_u: vec3<f32>,
    @location(5) tangent_v: vec3<f32>,
};

struct InstanceInput {
    // See wgpu::VertexBufferLayout in instance_raw.rs.
    // colums of the model transfrom matrix
    @location(6) model_matrix_0: vec4<f32>,
    @location(7) model_matrix_1: vec4<f32>,
    @location(8) model_matrix_2: vec4<f32>,
    @location(9) model_matrix_3: vec4<f32>,
    // extra general purpose data.
    @location(10) extra: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) alpha: f32,
};

// arguments from canvas.rs: render_pass.set_vertex_buffer(0, ...), set_vertex_buffer(1, ...)
@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    let t = instance.extra.x;
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = globals.view_proj * (model_matrix * vec4<f32>(model.position + t * model.velocity, 1.0));
    out.alpha = 1.0 - t;
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let extra_bright = 6.0; // pump up the brightness
    color.w *= extra_bright * in.alpha;
    //if color.w == 0.0 {
    //    discard;
    //}
    return color;
}