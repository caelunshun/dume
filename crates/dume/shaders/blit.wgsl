// Since compute shaders can't write directly
// to the surface texture, we use this
// render pipeline to blit onto the framebuffer.

struct VertexOutput {
    [[location(0)]] texcoord: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

struct Globals {
    size: vec2<u32>;
};

[[group(0), binding(0)]] var tex: texture_2d<u32>;
[[group(0), binding(1)]] var samp: sampler;
[[group(0), binding(2)]] var<uniform> globals: Globals;

// Vertex shader to generate a fullscreen quad without vertex buffers. See:
// https://www.saschawillems.de/blog/2016/08/13/vulkan-tutorial-on-rendering-a-fullscreen-quad-without-buffers/.
[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    let texcoord = vec2<f32>(f32((i32(vertex_index) << u32(1)) & 2), f32(i32(vertex_index) & 2));
    out.texcoord = texcoord * vec2<f32>(globals.size);
    out.position = vec4<f32>(texcoord.x * 2.0 - 1.0, -(texcoord.y * 2.0 - 1.0), 0.0, 1.0);

    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return unpack4x8unorm(textureLoad(tex, vec2<i32>(in.texcoord), 0).r);
}
