// This file contains two kernels, one for
// subdividing elements into tiles and one for
// actually painting to the target texture.

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

[[group(0), binding(0)]] var<uniform> globals: Globals;
[[group(0), binding(1)]] var<storage, read> nodes: Nodes;
[[group(0), binding(2)]] var<storage, read> node_bounding_boxes: NodeBoundingBoxes;
[[group(0), binding(3)]] var<storage, read_write> tiles: TileNodes;
[[group(0), binding(4)]] var<storage, read_write> tile_counters: TileNodeCounters;
[[group(0), binding(5)]] var target_texture: texture_storage_2d<rgba8unorm, read_write>;

fn unpack_pos(pos: u32) -> vec2<f32> {
    return unpack2x16unorm(pos) * globals.target_size * 2.0 * globals.scale_factor - globals.target_size / 2.0;
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
    let pos = clamp(pos, vec2<f32>(0.0), globals.target_size);
    return vec2<u32>(pos / 16.0);
}

[[stage(compute), workgroup_size(256, 1)]]
fn tile_kernel(
    [[builtin(global_invocation_id)]] global_id: vec3<u32>,
) {
    let node_index = global_id.x;
    if (node_index >= globals.node_count) {
        return;
    }

    let bbox = unpack_bounding_box(node_bounding_boxes.bounding_boxes[node_index]);
    if (bbox.pos.x + bbox.size.x < 0.0 || bbox.pos.y + bbox.size.y < 0.0) {
        return;
    }

    let min = to_tile_pos(bbox.pos);
    let max = to_tile_pos(bbox.pos + bbox.size);

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
        tiles.tile_nodes[tile_index + i] = node_index;

        x = x + u32(1);
    }
}


// Shader that sorts nodes in each tile to keep
// a stable draw order.
//
// We use insertion sort.

[[stage(compute), workgroup_size(16, 16)]]
fn sort_kernel([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
    let tile_id = global_id.xy;

    if (tile_id.x >= globals.tile_count.x || tile_id.y >= globals.tile_count.y) {
        return;
    }

    let num_nodes = i32(tile_counters.counters[tile_id.x + tile_id.y * globals.tile_count.x]);
    let base_index = i32(tile_index(tile_id));

    var i = base_index + 1;
    loop {
        if (i - base_index >= num_nodes) {
            break;
        }

        let x = tiles.tile_nodes[i];
        var j = i - 1;
        loop {
            if (!(j >= 0 && tiles.tile_nodes[j] > x)) {
                break;
            }

            tiles.tile_nodes[j + 1] = tiles.tile_nodes[j];
            j = j - 1;
        }
        tiles.tile_nodes[j + 1] = x;

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

let SHAPE_RECT: i32 = 0;
let SHAPE_CIRCLE: i32 = 1;

let PAINT_TYPE_SOLID: i32 = 0;

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

fn unpack_color(color: u32) -> vec4<f32> {
    let color = unpack4x8unorm(color);
    let srgb = srgb_to_linear(color.rgb);
    return vec4<f32>(srgb, color.a);
}

fn node_color(node: Node) -> vec4<f32> {
    if (node.paint_type == PAINT_TYPE_SOLID) {
        return unpack_color(node.color_a);
    } else {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }
}

fn node_coverage(node: Node, pixel_pos: vec2<f32>) -> f32 {
    let pixel_min = pixel_pos;
    let pixel_max = pixel_min + 1.0;
    let pixel_mid = pixel_min + 0.5;
    if (node.shape == SHAPE_RECT) {
        let size = unpack_pos(node.pos_b);
        let rect_min = unpack_pos(node.pos_a);
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
    } else if (node.shape == SHAPE_CIRCLE) {
        let center = unpack_pos(node.pos_a);
        let radius = unpack_pos(node.pos_b).x;

        let distance = length(pixel_mid - center);
        // Not the exact coverage, but close enough to look fine.
        let alpha = clamp(radius - distance, 0.0, 1.0);
        return alpha;
    } else {
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
    
    var color = textureLoad(target_texture, pixel).rgb;
    color = srgb_to_linear(color);

    var index = tile_index(tile_id.xy);
    let base_index = index;
    let num_nodes = tile_counters.counters[tile_id.x + tile_id.y * globals.tile_count.x];
    loop {
        if (index - base_index == num_nodes) {
            break;
        }

        let node_index = tiles.tile_nodes[index];

        let node: Node = nodes.nodes[node_index];

        let node_color = node_color(node);
        let node_coverage = node_coverage(node, pixel_pos);
        color = mix(color, node_color.rgb, node_coverage * node_color.a);

        index = index + u32(1);
    }

    // Blend onto the target texture. Note that we have
    // to do the linear => sRGB conversion ourselves.
    let color = clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));
    let result = linear_to_srgb(color);
    textureStore(target_texture, pixel, vec4<f32>(result, 1.0));
}
