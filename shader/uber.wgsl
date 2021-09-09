struct VertexOutput {
    [[location(0)]] f_TexCoord: vec2<f32>;
    [[location(1)]] f_Paint: vec2<i32>;
    [[location(2)]] f_WorldPos: vec2<f32>;
    [[location(3)]] f_ScissorRect: vec2<i32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[block]]
struct Locals {
    ortho: mat4x4<f32>;
};
[[group(0), binding(0)]]
var u_Locals: Locals;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] a_Pos: vec2<f32>,
    [[location(1)]] a_TexCoord: vec2<f32>,
    [[location(2)]] a_Paint: vec2<i32>,
    [[location(3)]] a_ScissorRect: vec2<i32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = u_Locals.ortho * vec4<f32>(a_Pos, 0.0, 1.0);
    out.f_TexCoord = a_TexCoord;
    out.f_Paint = a_Paint;
    out.f_WorldPos = a_Pos;
    out.f_ScissorRect = a_ScissorRect;
    return out;
}

[[group(0), binding(1)]]
var u_SpriteSampler: sampler;
[[group(0), binding(2)]]
var u_FontSampler: sampler;
[[group(0), binding(3)]]
var u_SpriteAtlas: texture_2d<f32>;
[[group(0), binding(4)]]
var u_FontAtlas: texture_2d<f32>;

[[block]]
struct Colors {
    buffer: [[stride(16)]] array<vec4<f32>>;
};

[[group(0), binding(5)]]
var<storage, read> colors: Colors;

[[stage(fragment)]]
fn fs_main(
    in: VertexOutput
) -> [[location(0)]] vec4<f32> {
    // For uniformity validation, these must be sampled before
    // we branch.
    let spriteAtlasColor = textureSample(u_SpriteAtlas, u_SpriteSampler, in.f_TexCoord);
    let fontAtlasColor = textureSample(u_FontAtlas, u_FontSampler, in.f_TexCoord);

    let paintType = in.f_Paint.x;
    let colorIndex = in.f_Paint.y;

    let paintTypeSolid = 0;
    let paintTypeSprite = 1;
    let paintTypeAlphaTexture = 2;
    let paintTypeLinearGradient = 3;
    let paintTypeRadialGradient = 4;

    if (paintType == paintTypeSolid) {
        return colors.buffer[colorIndex];
    } else { if (paintType == paintTypeSprite) {
        return spriteAtlasColor;
    } else { if (paintType == paintTypeAlphaTexture) {
        let color = colors.buffer[colorIndex];
        var result: vec4<f32> = vec4<f32>(color.rgb, fontAtlasColor.x);
        result.a = result.a * color.a;
        return result;
    } else { if (paintType == paintTypeLinearGradient) {
        let colorA = colors.buffer[colorIndex];
        let colorB = colors.buffer[colorIndex + 1];

        let point = colors.buffer[colorIndex + 2];
        let pointA = point.xy;
        let pointB = point.zw;

        // https://stackoverflow.com/questions/1459368/snap-point-to-a-line
        let ap = in.f_WorldPos - pointA;
        let ab = pointB - pointA;

        let ab2 = ab.x * ab.x + ab.y * ab.y;
        let apAB = ap.x * ab.x + ab.y * ap.y;
        var t: f32 = apAB / ab2;
        t = clamp(t, 0.0, 1.0);

        return colorA * (1.0 - t) + colorB * t;
    } else {   
        if (paintType == paintTypeRadialGradient) {
            let colorA = colors.buffer[colorIndex];
            let colorB = colors.buffer[colorIndex + 1];

            let point = colors.buffer[colorIndex + 2];
            let center = point.xy;
            let radius = point.z;

            let t = distance(center, in.f_WorldPos) / radius;

            return colorA * (1.0 - t) + colorB * t;
        }
    }}}}

    return vec4<f32>(0.0);
}
