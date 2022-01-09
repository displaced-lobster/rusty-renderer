// Vertex shader

struct Ambient {
    color: vec4<f32>;
};
[[group(0), binding(0)]]
var<uniform> ambient: Ambient;

struct Camera {
    view_pos: vec4<f32>;
    view_proj: mat4x4<f32>;
};
[[group(1), binding(0)]]
var<uniform> camera: Camera;

struct Light {
    position: vec3<f32>;
    color: vec3<f32>;
};
[[group(2), binding(0)]]
var<uniform> light: Light;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] color: vec4<f32>;
};
struct InstanceInput {
    [[location(3)]] model_matrix_0: vec4<f32>;
    [[location(4)]] model_matrix_1: vec4<f32>;
    [[location(5)]] model_matrix_2: vec4<f32>;
    [[location(6)]] model_matrix_3: vec4<f32>;
    [[location(7)]] normal_matrix_0: vec3<f32>;
    [[location(8)]] normal_matrix_1: vec3<f32>;
    [[location(9)]] normal_matrix_2: vec3<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] normal: vec3<f32>;
    [[location(1)]] position: vec3<f32>;
    [[location(2)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );

    let world_normal = normalize(normal_matrix * model.normal);
    let world_position = model_matrix * vec4<f32>(model.position, 1.0);

    var out: VertexOutput;
    out.clip_position = camera.view_proj * world_position;
    out.normal = world_normal;
    out.position = model.position;
    out.color = model.color;

    return out;
}

// Fragment shader

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let ambient_strength = 0.1;
    let ambient_color = ambient.color.xyz * ambient_strength;

    let light_dir = normalize(light.position - in.position);

    let diffuse_strength = dot(in.normal, light_dir);
    let diffuse_color = in.color.zyx * diffuse_strength;

    let color = ambient_color + diffuse_color;

    return vec4<f32>(color, in.color.a);
}
