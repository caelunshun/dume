// Shader to blit a layer onto another layer.

struct VertexOutput {
    [[location(0)]] tex_coords: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[block]]
struct Locals {
    projection_matrix: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> locals: Locals;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] tex_coords: vec2<f32>,
    [[location(1)]] position: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    out.tex_coords = tex_coords;
    out.position = locals.projection_matrix * vec4<f32>(position, 0.0, 1.0);

    return out;
}

[[group(0), binding(1)]] 
var layer: texture_2d<f32>;
[[group(0), binding(2)]] 
var sampler: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return textureSample(layer, sampler, in.tex_coords);
}
