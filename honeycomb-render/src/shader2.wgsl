struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: u32,
}

// vertex shader

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 0.0, 1.0);
    out.color = model.color;
    return out;
}

// fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color: array<vec3<f32>, 7> = array(
        vec3<f32>(0.1, 0.1, 0.1),
        vec3<f32>(1.0, 0.1, 0.1),
        vec3<f32>(0.1, 1.0, 0.1),
        vec3<f32>(0.1, 0.1, 1.0),
        vec3<f32>(0.1, 1.0, 1.0),
        vec3<f32>(1.0, 0.1, 1.0),
        vec3<f32>(1.0, 1.0, 0.1),
    );
    return vec4<f32>(color[in.color], 1.0);
}