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
    // See wgpu::VertexBufferLayout in vertex_kf.rs
    @location(0) tex_coords: vec2<f32>,
    @location(1) position1: vec3<f32>,
    @location(2) position2: vec3<f32>,
    @location(3) normal1: vec3<f32>,
    @location(4) normal2: vec3<f32>,
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
    @location(1) normal: vec3<f32>,
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
    let position = (1.0 - t) * model.position1 + t * model.position2;
    let normal = normalize((1.0 - t) * model.normal1 + t * model.normal2);

    var out: VertexOutput;
    out.clip_position = globals.view_proj * (model_matrix * vec4<f32>(position, 1.0));
    out.normal = normalize((model_matrix * vec4(normal, 0.0)).xyz); // TODO: remove normalize once model has scale 1.0;
    out.tex_coords = model.tex_coords;
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let diffuse = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let light = 0.8 * max(0.05, -dot(in.normal, globals.sun_dir));
    return vec4(light * diffuse.xyz, diffuse.w);
}