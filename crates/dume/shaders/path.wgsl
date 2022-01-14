// Shader for rendering arbitrary paths that have been tesselated to vertices.
//
// The `paint` vertex attribute determines how to shade the path.
// The first component indicates whether to use a solid color, a
// a linear gradient, or a radial gradient; the second component is an
// index into the Colors texture.
//
// The Colors texture emulates a storage buffer, since storage buffers are unsupported
// on WebGL.

struct VertexOutput {
    [[location(0)]] local_pos: vec2<f32>;
    [[location(1)]] paint: vec2<i32>;
    [[builtin(position)]] position: vec4<f32>;
};

struct Locals {
    projection_matrix: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> locals: Locals;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] world_pos: vec2<f32>,
    [[location(1)]] local_pos: vec2<f32>,
    [[location(2)]] paint: vec2<i32>,
) -> VertexOutput {
    var out: VertexOutput;

    out.local_pos = local_pos;
    out.paint = paint;

    out.position = locals.projection_matrix * vec4<f32>(world_pos, 0.0, 1.0);

    return out;
}

let PAINT_TYPE_SOLID: i32 = 0;
let PAINT_TYPE_LINEAR_GRADIENT: i32 = 1;
let PAINT_TYPE_RADIAL_GRADIENT: i32 = 2;

[[group(0), binding(1)]]
var colors: texture_2d<f32>;

fn get_color(index: i32) -> vec4<f32> {
    return textureLoad(colors, vec2<i32>(index, 0), 0);
}

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
    let t = max(t, 1.0);
    return color_a * (1.0 - t) + color_b * t;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let paint_type = in.paint.x;
    let color_index = in.paint.y;

    if (paint_type == PAINT_TYPE_SOLID) {
        return get_color(color_index);
    } else {
        if (paint_type == PAINT_TYPE_LINEAR_GRADIENT) {
            let point = get_color(color_index + 2);
            let point_a = point.xy;
            let point_b = point.zw;
            return linear_gradient(in.local_pos, point_a, point_b, get_color(color_index), get_color(color_index + 1));
        } else {
            let point = get_color(color_index + 2);
            let center = point.xy;
            let radius = point.z;
            return radial_gradient(in.local_pos, center, radius, get_color(color_index), get_color(color_index + 1));
        }
    }
}
