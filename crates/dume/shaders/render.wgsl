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
    
    // Some fields unused depending on paint_type
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
// In a pathological case, every node intersects each tile. So
// each tile in this buffer needs enough space to store
// `node_count` intersecting nodes.
//
// Thus, the stride between tiles in the array is `node_count * 4` (since
// one node index consumes 4 bytes).
struct TileNodes {
    tile_nodes: array<u32>;
};

[[group(0), binding(0)]] var<uniform> globals: Globals;
[[group(0), binding(1)]] var<storage, read> nodes: Nodes;
[[group(0), binding(2)]] var<storage, read> node_bounding_boxes: NodeBoundingBoxes;
[[group(0), binding(3)]] var<storage, read_write> tiles: TileNodes;
[[group(0), binding(4)]] var target_texture: texture_storage_2d<rgba8unorm, read_write>;

// Shader that assigns an array of nodes
// to each tile of 16x16 physical pixels.
//
// This kernel traverses the entire list of nodes
// for each tile. (Clearly there is room for optimization,
// e.g. by using a quadtree-like traversal.)

fn unpack_bounding_box(bbox: PackedBoundingBox) -> BoundingBox {
    var result: BoundingBox;
    result.pos = unpack2x16unorm(bbox.pos) * globals.target_size;
    result.size = unpack2x16unorm(bbox.size) * globals.target_size;
    return result;
}

fn tile_stride() -> u32 {
    return globals.node_count;
}

fn tile_index(tile_pos: vec2<u32>) -> u32 {
    return (tile_pos.x + tile_pos.y * globals.tile_count.x) * tile_stride();
}

[[stage(compute), workgroup_size(16, 16)]]
fn tile_kernel(
    [[builtin(global_invocation_id)]] global_id: vec3<u32>,
) {
    let tile_pos = global_id.xy;
    let tile_min = vec2<f32>(tile_pos) * 16.0 / globals.scale_factor;
    let tile_max = tile_min + vec2<f32>(16.0) / globals.scale_factor;

    if (tile_pos.x >= globals.tile_count.x || tile_pos.y >= globals.tile_count.y) {
        return;
    }

    var index: u32 = tile_index(tile_pos);
    var node_index: u32 = u32(0);
    loop {
        let node_bbox = unpack_bounding_box(node_bounding_boxes.bounding_boxes[node_index]);

        let node_min = node_bbox.pos;
        let node_max = node_min + node_bbox.size;

        // Do intersection test.
        if (node_max.x < tile_min.x || node_max.y < tile_min.y
            || node_min.x > tile_max.x || node_min.y > tile_max.y) {
            // No intersection; do nothing.
        } else {
            // NB a value of zero means the end of the list, so we add one to ensure the index is
            // always greater than zero. (The paint shader needs to subtract one to compensate.)
            tiles.tile_nodes[index] = node_index + u32(1);
            index = index + u32(1);
        }

        node_index = node_index + u32(1);
        if (node_index == globals.node_count) {
            break;
        }
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

fn unpack_pos(pos: u32) -> vec2<f32> {
    return unpack2x16unorm(pos) * globals.target_size * globals.scale_factor;
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
    loop {
        if (index - base_index == globals.node_count) {
            break;
        }

        let node_index = tiles.tile_nodes[index];
        if (node_index == u32(0)) {
            break;
        }

        let node: Node = nodes.nodes[node_index - u32(1)];

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
