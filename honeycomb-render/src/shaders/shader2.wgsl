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
    out.color = model.color % 7;
    return out;
}

// fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color: array<vec4<f32>, 7> = array(
        vec4<f32>(0.1, 0.1, 0.1, 1.0), // 0 -> black darts
        vec4<f32>(1.0, 0.1, 0.1, 0.1), // 1 ->
        vec4<f32>(0.8, 0.3, 0.3, 0.1), // 2 -> face
        vec4<f32>(0.1, 0.1, 1.0, 0.1), // 3 ->
        vec4<f32>(0.1, 1.0, 1.0, 0.1), // 4 ->
        vec4<f32>(1.0, 0.1, 1.0, 0.1), // 5 ->
        vec4<f32>(1.0, 1.0, 0.1, 0.1), // 6 ->
    );
    return color[in.color];
}