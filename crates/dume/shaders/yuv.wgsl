// Shader for rendering from YUV textures.
//
// The YUV image is stored in a separate texture for each plane,
// enabling subsampling modes like YUV420p.

struct VertexOutput {
    [[location(0)]] tex_coords: vec2<f32>;
    [[location(1)]] alpha: f32;
    [[builtin(position)]] position: vec4<f32>;
};

struct Locals {
    projection_matrix: mat4x4<f32>;
};
[[group(0), binding(0)]] 
var<uniform> locals: Locals;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] tex_coords: vec2<f32>,
    [[location(1)]] pos: vec2<f32>,
    [[location(2)]] alpha: f32,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = locals.projection_matrix * vec4<f32>(pos, 0.0, 1.0);
    out.alpha = alpha;
    out.tex_coords = tex_coords;
    return out;
}

[[group(0), binding(1)]] var samp: sampler;
[[group(0), binding(2)]] var texture_y: texture_2d<f32>;
[[group(0), binding(3)]] var texture_u: texture_2d<f32>;
[[group(0), binding(4)]] var texture_v: texture_2d<f32>;

fn srgb_to_linear(srgb: vec3<f32>) -> vec3<f32> {
    let cutoff = srgb < vec3<f32>(0.04045);
    let higher = pow((srgb + vec3<f32>(0.055)) / vec3<f32>(1.055), vec3<f32>(2.4));
    let lower = srgb / vec3<f32>(12.92);

    return mix(higher, lower, vec3<f32>(cutoff));
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var yuv: vec3<f32>;
    yuv.x = textureSample(texture_y, samp, in.tex_coords).r;
    yuv.y = textureSample(texture_u, samp, in.tex_coords).r;
    yuv.z = textureSample(texture_v, samp, in.tex_coords).r;
    
    yuv = yuv + vec3<f32>(-0.0627451017, -0.501960814, -0.501960814);
    

    // YUV => RGB conversion routine
    var color: vec4<f32>;
    color.r = dot(yuv, vec3<f32>(1.164, 0.000, 1.596));
    color.g = dot(yuv, vec3<f32>(1.164, -0.391, -0.813));
    color.b = dot(yuv, vec3<f32>(1.164,  2.018,  0.000));
    color.a = in.alpha;

    return vec4<f32>(srgb_to_linear(color.rgb), color.a);
}
