// Since compute shaders can't write directly
// to the surface texture, we use this
// render pipeline to blit onto the framebuffer.

struct VertexOutput {
    [[location(0)]] texcoord: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[group(0), binding(0)]] var tex: texture_2d<f32>;
[[group(0), binding(1)]] var samp: sampler;

// Vertex shader to generate a fullscreen quad without vertex buffers. See:
// https://www.saschawillems.de/blog/2016/08/13/vulkan-tutorial-on-rendering-a-fullscreen-quad-without-buffers/.
[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    out.texcoord = vec2<f32>(f32((i32(vertex_index) << u32(1)) & 2), f32(i32(vertex_index) & 2));
    out.position = vec4<f32>(out.texcoord.x * 2.0 - 1.0, -(out.texcoord.y * 2.0 - 1.0), 0.0, 1.0);

    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return textureSample(tex, samp, in.texcoord);
}
