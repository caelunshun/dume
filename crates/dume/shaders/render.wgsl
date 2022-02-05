// This file contains two kernels, one for
// subdividing elements into tiles and one for
// actually painting to the target texture.

let SHAPE_RECT: i32 = 0;
let SHAPE_CIRCLE: i32 = 1;
let SHAPE_STROKE: i32 = 2;
let SHAPE_FILL: i32 = 3;

let PAINT_TYPE_SOLID: i32 = 0;
let PAINT_TYPE_LINEAR_GRADIENT: i32 = 1;
let PAINT_TYPE_RADIAL_GRADIENT: i32 = 2;
let PAINT_TYPE_GLYPH: i32 = 3;

let STROKE_CAP_ROUND: i32 = 0;
let STROKE_CAP_SQUARE: i32 = 1;

struct PackedBoundingBox {
    pos: u32;
    size: u32;
};

struct BoundingBox {
    pos: vec2<f32>;
    size: vec2<f32>;
};

struct Globals {
    // Logical size of the target texture
    target_size: vec2<f32>;
    // Number of 16x16 tiles in each dimension
    tile_count: vec2<u32>;
    // Number of nodes in the input
    node_count: u32;
    // Scale factor from logical to physical pixel
    scale_factor: f32;
};

struct Node {
    shape: i32;
    pos_a: u32;
    pos_b: u32;
    extra: u32;

    paint_type: i32;
    
    // Some fields are unused depending on paint_type
    color_a: u32;
    color_b: u32;
    gradient_point_a: u32;
    gradient_point_b: u32;
};

// Stores input nodes and theiir bounding boxes;
struct NodeBoundingBoxes {
    bounding_boxes: array<PackedBoundingBox>;
};

struct Nodes {
    nodes: array<Node>;
};

// Stores the nodes that intersect each tile.
//
// Thus, the stride between tiles in the array is `64 * 4` (since
// one node index consumes 4 bytes). We assume no more than
// 64 elements will intersect one tile. (TODO handle this case.)
struct TileNodes {
    tile_nodes: array<u32>;
};

// Stores atomic counters for how many
// nodes are in each tile in `TileNodes`.
struct TileNodeCounters {
    counters: array<atomic<u32>>;
};

// Stores points used for filling and stroking paths.
struct Points {
    list: array<u32>;
};

[[group(0), binding(0)]] var<uniform> globals: Globals;
[[group(0), binding(1)]] var<storage, read> nodes: Nodes;
[[group(0), binding(2)]] var<storage, read> node_bounding_boxes: NodeBoundingBoxes;
[[group(0), binding(3)]] var<storage, read_write> tiles: TileNodes;
[[group(0), binding(4)]] var<storage, read_write> tile_counters: TileNodeCounters;
[[group(0), binding(5)]] var target_texture: texture_storage_2d<r32uint, read_write>;

[[group(0), binding(6)]] var samp_linear: sampler;
[[group(0), binding(7)]] var glyph_atlas: texture_2d<f32>;

[[group(0), binding(8)]] var<storage, read> points: Points;

fn unpack_pos(pos: u32) -> vec2<f32> {
    return unpack2x16unorm(pos) * globals.target_size * 2.0 - globals.target_size / 2.0;
}

fn unpack_upos(pos: u32) -> vec2<u32> {
    return vec2<u32>(
        pos & u32(0xFFFF),
        (pos >> u32(16)) & u32(0xFFFF),
    );
}

fn to_physical(pos: vec2<f32>) -> vec2<f32> {
    return pos * globals.scale_factor;
}

// Shader that assigns an array of nodes
// to each tile of 16x16 physical pixels.
//
// This kernel runs for each node and determines
// the list of tiles the node intersects. For each tile
// in the resulting list, it adds the node index to the tile's
// list of intersecting nodes.

fn unpack_bounding_box(bbox: PackedBoundingBox) -> BoundingBox {
    var result: BoundingBox;
    result.pos = unpack_pos(bbox.pos);
    result.size = unpack_pos(bbox.size);
    return result;
}

fn tile_stride() -> u32 {
    return u32(64);
}

fn tile_index(tile_pos: vec2<u32>) -> u32 {
    return (tile_pos.x + tile_pos.y * globals.tile_count.x) * tile_stride();
}

fn to_tile_pos(pos: vec2<f32>) -> vec2<u32> {
    let pos = to_physical(pos);
    let pos = clamp(pos, vec2<f32>(0.0), globals.target_size * globals.scale_factor);
    return vec2<u32>(pos / 16.0);
}

// Shader that assigns an array of nodes
// to each tile of 16x16 physical pixels.
//
// This kernel runs for each node (excluding fills) and determines
// the list of tiles the node intersects. For each tile
// in the resulting list, it adds the node index to the tile's
// list of intersecting nodes.

fn spread_tiles(node_index: u32, min: vec2<u32>, max: vec2<u32>) {
    var x = min.x;
    var y = min.y;
    loop {
        if (x > max.x || x >= globals.tile_count.x) {
            y = y + u32(1);
            x = min.x;
            if (y > max.y || y >= globals.tile_count.y) {
                break;
            }
        }

        let tile_index = tile_index(vec2<u32>(x, y));
        let ip = &tile_counters.counters[x + y * globals.tile_count.x];
        let i = atomicAdd(ip, u32(1));
        if (i >= u32(64)) {
            // The tile's buffer is full. For now, we'll
            // just skip the excess nodes - in the future we might
            // want some sort of overflow mechanism.
            x = x + u32(1);
            continue;
        }
        tiles.tile_nodes[tile_index + i] = node_index;

        x = x + u32(1);
    }
}

fn tile_normal_node(node: Node, node_index: u32) {
    let bbox = unpack_bounding_box(node_bounding_boxes.bounding_boxes[node_index]);
    if (bbox.pos.x + bbox.size.x < 0.0 || bbox.pos.y + bbox.size.y < 0.0) {
        return;
    }

    let min = to_tile_pos(bbox.pos);
    let max = to_tile_pos(bbox.pos + bbox.size);

    spread_tiles(node_index, min, max);
}

fn get_fill_bbox(offset: u32) -> BoundingBox {
    var packed: PackedBoundingBox;
    packed.pos = points.list[offset];
    packed.size = points.list[offset + u32(1)];
    return unpack_bounding_box(packed); 
}

fn tile_fill_node(node: Node, node_index: u32) {
    // Add the node to the tiles it intersects and all tiles to the right
    // for the remainder of the entire fill's bounding box.
    let bbox = unpack_bounding_box(node_bounding_boxes.bounding_boxes[node_index]);
    if (bbox.pos.x + bbox.size.x < 0.0 || bbox.pos.y + bbox.size.y < 0.0) {
        return;
    }

    let min = to_tile_pos(bbox.pos);
    let nodemax = to_tile_pos(bbox.pos + bbox.size);

    let offset = unpack_upos(node.pos_a).y;
    let fill_bbox = get_fill_bbox(offset);

    var max = to_tile_pos(fill_bbox.size + fill_bbox.pos);
    max.y = clamp(max.y, min.y, nodemax.y);

    spread_tiles(node_index, min, max);
}

[[stage(compute), workgroup_size(256)]]
fn tile_kernel(
    [[builtin(global_invocation_id)]] global_id: vec3<u32>,
) {
    let node_index = global_id.x;
    if (node_index >= globals.node_count) {
        return;
    }

    let node = nodes.nodes[node_index];
    if (node.shape == SHAPE_FILL) {
        tile_fill_node(node, node_index);
    } else {
        tile_normal_node(node, node_index);
    }
}


// Shader that sorts nodes in each tile to keep
// a stable draw order.
//
// We use insertion sort.

// The array of nodes loaded into private memory to reduce
// access times
var<private> local_nodes: array<u32, 64>;

[[stage(compute), workgroup_size(16, 16)]]
fn sort_kernel([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
    let tile_id = global_id.xy;

    if (tile_id.x >= globals.tile_count.x || tile_id.y >= globals.tile_count.y) {
        return;
    }

    // Copy the node list from global memory into private memory
    var num_nodes = i32(tile_counters.counters[tile_id.x + tile_id.y * globals.tile_count.x]);
    num_nodes = min(num_nodes, 64);
    let base_index = i32(tile_index(tile_id));
    var i = base_index;
    loop {
        if (i - base_index >= num_nodes) {
            break;
        }
        local_nodes[i - base_index] = tiles.tile_nodes[i];
        i = i + 1;
    }

    i = 0;
    loop {
        if (i >= num_nodes) {
            break;
        }

        let x = local_nodes[i];
        var j = i - 1;
        loop {
            if (!(j >= 0 && local_nodes[j] > x)) {
                break;
            }

            local_nodes[j + 1] = local_nodes[j];
            j = j - 1;
        }
        local_nodes[j + 1] = x;

        i = i + 1;
    }

    // Copy the node list back into global memory
    i = 0;
    loop {
        if (i >= num_nodes) {
            break;
        }
        tiles.tile_nodes[i + base_index] = local_nodes[i];
        i = i + 1;
    }
}

// Shader that runs on each pixel
// in each tile, executing draw commands.
//
// Drawing generally happens in three stages:
// 1) Determine the area coverage of the pixel from the shape being drawn
//    (thus also handling antialiasing)
// 2) Determine the color of the pixel based on the paint type
// 3) Composite the color onto the target texture, using the alpha value
//    from the coverage step

fn srgb_to_linear(srgb: vec3<f32>) -> vec3<f32> {
    let cutoff = srgb < vec3<f32>(0.04045);
    let higher = pow((srgb + vec3<f32>(0.055)) / vec3<f32>(1.055), vec3<f32>(2.4));
    let lower = srgb / vec3<f32>(12.92);

    return mix(higher, lower, vec3<f32>(cutoff));
}

fn linear_to_srgb(linear: vec3<f32>) -> vec3<f32> {
    let cutoff = linear < vec3<f32>(0.0031308);
    let higher = 1.055 * pow(linear, vec3<f32>(1.0 / 2.4)) - 0.055;
    let lower = linear * 12.92;
    return mix(higher, lower, vec3<f32>(cutoff));
}

fn linear_to_oklab(linear: vec3<f32>) -> vec3<f32> {
    let l = pow(0.4122214708 * linear.r + 0.5363325363 * linear.g + 0.0514459929 * linear.b, 0.33);
    let m = pow(0.2119034982 * linear.r + 0.6806995451 * linear.g + 0.1073969566 * linear.b, 0.33);
    let s = pow(0.0883024619 * linear.r + 0.2817188376 * linear.g + 0.6299787005 * linear.b, 0.33);
    return vec3<f32>(l * 0.2104542553 + m * 0.7936177850 + s * -0.0040720468,
        l * 1.9779984951 + m * -2.4285922050 + s * 0.4505937099,
        l * 0.0259040371 + m * 0.7827717662 + s * -0.8086757660);
}

fn oklab_to_linear(oklab: vec3<f32>) -> vec3<f32> {
    var l = oklab.x + oklab.y * 0.3963377774 + oklab.z * 0.2158037573;
    var m = oklab.x + oklab.y * -0.1055613458 + oklab.z * -0.0638541728;
    var s = oklab.x + oklab.y * -0.0894841775 + oklab.z * -1.2914855480;
    l = l * l * l; m = m * m * m; s = s * s * s;
    var r = l * 4.0767416621 + m * -3.3077115913 + s * 0.2309699292;
    var g = l * -1.2684380046 + m * 2.6097574011 + s * -0.3413193965;
    var b = l * -0.0041960863 + m * -0.7034186147 + s * 1.7076147010;
    return vec3<f32>(r, g, b);
}

fn oklab_to_lch(oklab: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(oklab.x, length(oklab.yz), atan2(oklab.z, oklab.y));
}

fn lch_to_oklab(lch: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(lch.x, lch.y * cos(lch.z), lch.y * sin(lch.z));
}

fn unpack_color(color: u32) -> vec4<f32> {
    let color = unpack4x8unorm(color);
    let srgb = srgb_to_linear(color.rgb);
    return vec4<f32>(srgb, color.a);
}

// Interpolates between two colors.
//
// The colors should be in linear RGB. Internally,
// this function uses the Oklab color space to generate
// smoother gradients.
fn interpolate_colors(color_a: vec4<f32>, color_b: vec4<f32>, t: f32) -> vec4<f32> {
    let ca = linear_to_oklab(color_a.rgb);
    let cb = linear_to_oklab(color_b.rgb);

    var result = ca * (1.0 - t) + cb * t;
    result = oklab_to_linear(result);
    return vec4<f32>(result, color_a.a * (1.0 - t) + color_b.a * t);
}

fn linear_gradient(pos: vec2<f32>, point_a: vec2<f32>, point_b: vec2<f32>, color_a: vec4<f32>, color_b: vec4<f32>) -> vec4<f32> {
    // https://stackoverflow.com/questions/1459368/snap-point-to-a-line
    let ap = pos - point_a;
    let ab = point_b - point_a;

    let ab2 = ab.x * ab.x + ab.y * ab.y;
    let ap_ab = ap.x * ab.x + ab.y * ap.y;
    var t: f32 = ap_ab / ab2;
    t = clamp(t, 0.0, 1.0);

    return interpolate_colors(color_a, color_b, t);
}

fn radial_gradient(pos: vec2<f32>, center: vec2<f32>, radius: f32, color_a: vec4<f32>, color_b: vec4<f32>) -> vec4<f32> {
    let t = distance(center, pos) / radius;
    let t = clamp(t, 0.0, 1.0);
    return interpolate_colors(color_a, color_b, t);
}

fn node_color(node: Node, pixel_pos: vec2<f32>) -> vec4<f32> {
    let paint = node.paint_type;
    if (paint == PAINT_TYPE_SOLID) {
        return unpack_color(node.color_a);
    } else if (paint == PAINT_TYPE_LINEAR_GRADIENT) {
        let point_a = to_physical(unpack_pos(node.gradient_point_a));
        let point_b = to_physical(unpack_pos(node.gradient_point_b));
        let color_a = unpack_color(node.color_a);
        let color_b = unpack_color(node.color_b);
        return linear_gradient(pixel_pos, point_a, point_b, color_a, color_b);
    } else if (paint == PAINT_TYPE_RADIAL_GRADIENT) {
        let center = to_physical(unpack_pos(node.gradient_point_a));
        let radius = to_physical(unpack_pos(node.gradient_point_b)).x;
        let color_a = unpack_color(node.color_a);
        let color_b = unpack_color(node.color_b);
        return radial_gradient(pixel_pos, center, radius, color_a, color_b); 
    } else if (paint == PAINT_TYPE_GLYPH) {
        let offset = unpack_upos(node.gradient_point_a);
        let origin = unpack_upos(node.gradient_point_b);
        let color = unpack_color(node.color_a);
        
        let texcoords = offset + (vec2<u32>(pixel_pos) - origin);
        let alpha = textureLoad(glyph_atlas, vec2<i32>(texcoords), 0).r;
        return vec4<f32>(color.rgb, alpha * color.a);
    } else {
        // Should never happen.
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }
}

var<workgroup> nodes_in_tile: array<Node, 64>;
var<private> node_index: i32;
var<private> num_nodes: i32;

fn has_next_node() -> bool {
    return node_index < num_nodes;
}

fn peek_next_node() -> Node {
    return nodes_in_tile[node_index];
}

fn take_next_node() -> Node {
    node_index = node_index + 1;
    return nodes_in_tile[node_index - 1];
}

fn rect_coverage(node: Node, pixel_pos: vec2<f32>) -> f32 {
    let pixel_min = pixel_pos;
    let pixel_max = pixel_min + 1.0;
    let pixel_mid = pixel_min + 0.5;
    let size = to_physical(unpack_pos(node.pos_b));
    let rect_min = to_physical(unpack_pos(node.pos_a));
    let rect_max = rect_min + size;
    // Compute intersection area
    // between the pixel and the rectangle.
    if (rect_max.x < pixel_min.x || rect_max.y < pixel_min.y 
        || rect_min.x > pixel_max.x || rect_min.y > pixel_max.y) {
        return 0.0;
    }

    let length_x = min(rect_max.x, pixel_max.x) - max(rect_min.x, pixel_min.x);
    let length_y = min(rect_max.y, pixel_max.y) - max(rect_min.y, pixel_min.y);
    let area = length_x * length_y;
    return clamp(area, 0.0, 1.0);
}

fn circle_coverage(node: Node, pixel_pos: vec2<f32>) -> f32 {
    let pixel_mid = pixel_pos + 0.5;
    let center = to_physical(unpack_pos(node.pos_a));
    let radius = to_physical(unpack_pos(node.pos_b)).x;

    let distance = length(pixel_mid - center);
    // Not the exact coverage, but close enough to look fine.
    let alpha = clamp(radius - distance, 0.0, 1.0);
    return alpha;
}

fn projection_on_line(a: vec2<f32>, b: vec2<f32>, pos: vec2<f32>) -> f32 {
    let l = distance(a, b);
    if (l == 0.0) {
        return 0.0;
    }
    let t = dot(pos - a, b - a) / (l * l);
    return t;
}

fn projection_on_line_segment(a: vec2<f32>, b: vec2<f32>, pos: vec2<f32>) -> vec2<f32> {
    let l = distance(a, b);
    if (l == 0.0) {
        return a;
    }
    let t = max(0.0, min(1.0, dot(pos - a, b - a) / (l * l)));
    let projection = a + t * (b - a);
    return projection;
}

fn distance_to_line_segment(a: vec2<f32>, b: vec2<f32>, pos: vec2<f32>) -> f32 {
    let projection = projection_on_line_segment(a, b, pos);
    return distance(pos, projection);
}

fn stroke_coverage(node: Node, pos: vec2<f32>) -> f32 {
    let index = unpack_upos(node.pos_a).x;
    let point_a = to_physical(unpack_pos(points.list[index]));
    let point_b = to_physical(unpack_pos(points.list[index + u32(1)]));

    let params2 = unpack_pos(node.pos_b);
    let stroke_width = params2.x * globals.scale_factor;
    let stroke_cap = i32(round(params2.y));

    var dist = 0.0;
    if (stroke_cap == STROKE_CAP_ROUND) {
        dist = distance_to_line_segment(point_a, point_b, pos);
    } else if (stroke_cap == STROKE_CAP_SQUARE) {
        let d = distance(point_a, point_b);
        if (d == 0.0) {
            dist = distance(point_a, pos);
        } else {
            let t = projection_on_line(point_a, point_b, pos);
            let max_t = stroke_width / d;
            let projection = point_a + t * (point_b - point_a);

            var end_factor = 0.0;
            if (t > 1.0) {
                end_factor = (t - 1.0) * d;
            } else if (t < 0.0) {
                end_factor = -t * d;
            }

            dist = max(distance(pos, projection), end_factor);
        }
    }

    let alpha = clamp(stroke_width - dist, 0.0, 1.0);
    return alpha;
}

fn fill_coverage(node: Node, pixel_pos: vec2<f32>) -> f32 {
    // Use even-odd fill rule on all nodes with the same fill ID.
    let fill_id = node.extra;

    var signed_area = 0.0;
    var node = node;

    loop {
        let offset = unpack_upos(node.pos_a).x;
        var point_a = to_physical(vec2<f32>(unpack_upos(points.list[offset])));
        var point_b = to_physical(vec2<f32>(unpack_upos(points.list[offset + u32(1)])));
        if (point_a.x > point_b.x) {
            let temp = point_b;
            point_b = point_a;
            point_a = temp;
        }

        // Determine the signed area contribution from this 
        // segment.
        let start = point_a - pixel_pos;
        let end = point_b - pixel_pos;
        let window = clamp(vec2<f32>(start.y, end.y), vec2<f32>(0.0), vec2<f32>(1.0));
        if (window.x != window.y) {
            let t = (window - start.y) / (end.y - start.y);
            let xs = mix(vec2<f32>(start.x), vec2<f32>(end.x), t);
            let xmin = min(min(xs.x, xs.y), 1.0);
            let xmax = max(xs.x, xs.y);
            if (abs(xmin - xmax) < 0.0001) {
                signed_area = signed_area + (window.x - window.y);
            } else {
                let b = min(xmax, 1.0);
                let c = max(b, 0.0);
                let d = max(xmin, 0.0);
                let area = (b + 0.5 * (d * d - c * c) - xmin) / (xmax - xmin);
                signed_area = signed_area + area * (window.x - window.y);
            }
        }

        if (!has_next_node()) {
            break;
        }
        let next_node = peek_next_node();
        if (next_node.extra != fill_id) {
            break;
        }
        node = take_next_node();
    }
    return clamp(abs(signed_area - 2.0 * round(0.5 * signed_area)), 0.0, 1.0);
}

fn node_coverage(node: Node, pixel_pos: vec2<f32>) -> f32 {
    if (node.shape == SHAPE_RECT) {
        return rect_coverage(node, pixel_pos);
    } else if (node.shape == SHAPE_CIRCLE) {
        return circle_coverage(node, pixel_pos);
    } else if (node.shape == SHAPE_STROKE) {
        return stroke_coverage(node, pixel_pos);
    } else if (node.shape == SHAPE_FILL) {
        return fill_coverage(node, pixel_pos);
    } else {
        // Should never happen
        return 1.0;
    }
}

[[stage(compute), workgroup_size(16, 16)]]
fn paint_kernel(
    [[builtin(local_invocation_id)]] local_id: vec3<u32>,
    [[builtin(workgroup_id)]] tile_id: vec3<u32>,
) {
    let pixel = vec2<i32>(tile_id.xy) * vec2<i32>(16) + vec2<i32>(local_id.xy);
    let pixel_pos = vec2<f32>(pixel);
    
    var color = unpack4x8unorm(textureLoad(target_texture, pixel).r).rgb;
    color = srgb_to_linear(color);

    let base_index = i32(tile_index(tile_id.xy));
    num_nodes = i32(tile_counters.counters[tile_id.x + tile_id.y * globals.tile_count.x]);
    num_nodes = min(num_nodes, 64);
    node_index = 0;

    // Copy nodes into workgroup memory
    if (local_id.x == u32(0)) {
        var i = 0;
        loop {
            if (i >= num_nodes) {
                break;
            }
            let node_index = tiles.tile_nodes[base_index + i];
            nodes_in_tile[i] = nodes.nodes[node_index];
            i = i + 1;
        }
    }

    workgroupBarrier();

    loop {
        if (!has_next_node()) {
            break;
        }
        let node: Node = take_next_node();

        var coverage = 0.0;
        if (node.shape == SHAPE_STROKE) {
            // Consume all segments in the same path (each is its own node)
            // then choose the segment with the highest coverage.
            let path_id = node.extra;
            var n = node;
            loop {
                coverage = max(coverage, node_coverage(n, pixel_pos));
                if (!has_next_node()) {
                    break;
                }
                let next_node = peek_next_node();
                if (next_node.extra != path_id || next_node.shape != SHAPE_STROKE) {
                    break;
                }
                n = take_next_node();
            }
        } else {
            coverage = node_coverage(node, pixel_pos);
        }

        let node_color = node_color(node, pixel_pos);
        color = mix(color, node_color.rgb, coverage * node_color.a);
    }

    // Blend onto the target texture. Note that we have
    // to do the linear => sRGB conversion ourselves.
    let color = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));
    let result = linear_to_srgb(color);
    textureStore(target_texture, pixel, vec4<u32>(pack4x8unorm(vec4<f32>(result, 1.0))));
}
