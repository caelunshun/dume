// Shader for rendering arbitrary paths, tesselated to vertices.
//
// The `paint` vertex attribute determines how to shade the path.
// The first component indicates whether to use a solid color, a
// a linear gradient, or a radial gradient; the second component is an
// index into the Colors storage buffer.

struct VertexOutput {
    [[location(0)]] world_pos: vec2<f32>;
    [[location(1)]] paint: vec2<u32>;
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
    [[location(0)]] world_pos: vec2<f32>,
    [[location(1)]] paint: vec2<u32>,
) -> VertexOutput {
    var out: VertexOutput;

    out.world_pos = world_pos;
    out.paint = paint;

    out.position = locals.projection_matrix * vec4<f32>(world_pos, 0.0, 1.0);

    return out;
}

var PAINT_TYPE_SOLID: u32 = 0;
var PAINT_TYPE_LINEAR_GRADIENT: u32 = 1;
var PAINT_TYPE_RADIAL_GRADIENT: u32 = 2;

[[block]]
struct Colors {
    buffer: [[stride(16)]] array<vec4<f32>>;
};
[[group(0), binding(1)]]
var<storage, read> colors: Colors;

fn linear_gradient(pos: vec2<f32>, point_a: vec2<f32>, point_b: vec2<f32>, color_a: vec4<f32>, color_b: vec4<f32>) -> vec4<f32> {
    // https://stackoverflow.com/questions/1459368/snap-point-to-a-line
    let ap = pos - point_a;
    let ab = point_b - point_a;

    let ab2 = ab.x * ab.x + ab.y * ab.y;
    let ap_ab = ap.x * ab.x + ab.y * ap.y;
    var t: f32 = ap_ab / ab2;
    t = clamp(t, 0.0, 1.0);

    return color_a * (1.0 - t) + color_b * t;
}

fn radial_gradient(pos: vec2<f32>, center: vec2<f32>, radius: f32, color_a: vec4<f32>, color_b: vec4<f32>) -> vec4<f32> {
    let t = distance(center, pos) / radius;
    return color_a * (1.0 - t) + color_b * t;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let paint_type = in.paint.x;
    let color_index = i32(in.paint.y);

    if (paint_type == PAINT_TYPE_SOLID) {
        return colors.buffer[color_index];
    } else {
        if (paint_type == PAINT_TYPE_LINEAR_GRADIENT) {
            let point = colors.buffer[color_index + 2];
            let point_a = point.xy;
            let point_b = point.zw;
            return linear_gradient(in.world_pos, point_a, point_b, colors.buffer[color_index], colors.buffer[color_index + 1]);
        } else {
            let point = colors.buffer[color_index + 2];
            let center = point.xy;
            let radius = point.z;
            return radial_gradient(in.world_pos, center, radius, colors.buffer[color_index], colors.buffer[color_index + 1]);
        }
    }
}
