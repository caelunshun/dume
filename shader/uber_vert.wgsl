[[block]]
struct Locals {
    u_Ortho: mat4x4<f32>;
};

struct VertexOutput {
    [[location(0)]] f_TexCoord: vec2<f32>;
    [[location(1)]] f_Paint: vec2<i32>;
    [[location(2)]] f_WorldPos: vec2<f32>;
    [[location(3)]] f_ScissorRect: vec2<i32>;
    [[builtin(position)]] member: vec4<f32>;
};

var<private> a_Pos1: vec2<f32>;
var<private> a_TexCoord1: vec2<f32>;
var<private> a_Paint1: vec2<i32>;
var<private> a_ScissorRect1: vec2<i32>;
var<private> f_TexCoord: vec2<f32>;
var<private> f_Paint: vec2<i32>;
var<private> f_WorldPos: vec2<f32>;
var<private> f_ScissorRect: vec2<i32>;
[[group(0), binding(0)]]
var<uniform> global: Locals;
var<private> gl_Position: vec4<f32>;

fn main1() {
    let _e11: mat4x4<f32> = global.u_Ortho;
    let _e12: vec2<f32> = a_Pos1;
    gl_Position = (_e11 * vec4<f32>(_e12, 0.0, 1.0));
    let _e17: vec2<f32> = a_TexCoord1;
    f_TexCoord = _e17;
    let _e18: vec2<i32> = a_Paint1;
    f_Paint = _e18;
    let _e19: vec2<f32> = a_Pos1;
    f_WorldPos = _e19;
    let _e20: vec2<i32> = a_ScissorRect1;
    f_ScissorRect = _e20;
    return;
}

[[stage(vertex)]]
fn main([[location(0)]] a_Pos: vec2<f32>, [[location(1)]] a_TexCoord: vec2<f32>, [[location(2)]] a_Paint: vec2<i32>, [[location(3)]] a_ScissorRect: vec2<i32>) -> VertexOutput {
    a_Pos1 = a_Pos;
    a_TexCoord1 = a_TexCoord;
    a_Paint1 = a_Paint;
    a_ScissorRect1 = a_ScissorRect;
    main1();
    let _e27: vec2<f32> = f_TexCoord;
    let _e29: vec2<i32> = f_Paint;
    let _e31: vec2<f32> = f_WorldPos;
    let _e33: vec2<i32> = f_ScissorRect;
    let _e35: vec4<f32> = gl_Position;
    return VertexOutput(_e27, _e29, _e31, _e33, _e35);
}
