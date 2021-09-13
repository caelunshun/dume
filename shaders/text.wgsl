// Shader for renderirng text. Uses grayscale alpha.

struct VertexOutput {
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] tex_coords: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[block]]
struct Locals {
    projection_matrix: mat4x4<f32>;
};
[[group(0), binding(0)]] 
var locals: Locals;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] color: vec4<f32>,
    [[location(1)]] tex_coords: vec2<f32>,
    [[location(2)]] pos: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    out.color = color;
    out.tex_coords = tex_coords;
    out.position = locals.projection_matrix * vec4<f32>(pos, 0.0, 1.0);

    return out;
}

[[group(0), binding(1)]]
var font_atlas: texture_2d<f32>;
[[group(0), binding(2)]]
var sampler: sampler;

[[stage(fragment)]]
fn fs_main(
    in: VertexOutput
) -> [[location(0)]] vec4<f32> {
    let alpha = textureSample(font_atlas, sampler, in.tex_coords).r;
    let color = vec4<f32>(in.color.rgb, alpha * in.color.a);
    return color;
}
