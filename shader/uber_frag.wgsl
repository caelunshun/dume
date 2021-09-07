[[block]]
struct Colors {
    colors: [[stride(16)]] array<vec4<f32>>;
};

struct FragmentOutput {
    [[location(0)]] o_Color: vec4<f32>;
};

var<private> f_TexCoord1: vec2<f32>;
var<private> f_Paint1: vec2<i32>;
var<private> f_WorldPos1: vec2<f32>;
var<private> f_ScissorRect1: vec2<i32>;
[[group(0), binding(1)]]
var u_Sampler: sampler;
[[group(0), binding(2)]]
var u_NearestSampler: sampler;
[[group(0), binding(3)]]
var u_SpriteAtlas: texture_2d<f32>;
[[group(0), binding(4)]]
var u_FontAtlas: texture_2d<f32>;
[[group(0), binding(5)]]
var<storage, read_write> global: Colors;
var<private> o_Color: vec4<f32>;

fn main1() {
    var spriteAtlasColor: vec4<f32>;
    var fontAtlasColor: vec4<f32>;
    var paintType: i32;
    var colorIndex: i32;
    var color: vec4<f32>;
    var colorA: vec4<f32>;
    var colorB: vec4<f32>;
    var point: vec4<f32>;
    var pointA: vec2<f32>;
    var pointB: vec2<f32>;
    var ap: vec2<f32>;
    var ab: vec2<f32>;
    var ab2_: f32;
    var apAB: f32;
    var t: f32;
    var encodedRect: vec4<f32>;
    var rectPos: vec2<f32>;
    var rectSize: vec2<f32>;

    let _e11: vec2<f32> = f_TexCoord1;
    let _e12: vec4<f32> = textureSample(u_SpriteAtlas, u_Sampler, _e11);
    spriteAtlasColor = _e12;
    let _e14: vec2<f32> = f_TexCoord1;
    let _e15: vec4<f32> = textureSample(u_FontAtlas, u_NearestSampler, _e14);
    fontAtlasColor = _e15;
    let _e17: vec2<i32> = f_Paint1;
    paintType = _e17.x;
    let _e20: vec2<i32> = f_Paint1;
    colorIndex = _e20.y;
    let _e23: i32 = paintType;
    if ((_e23 == 0)) {
        {
            let _e26: i32 = colorIndex;
            let _e28: vec4<f32> = global.colors[_e26];
            o_Color = _e28;
        }
    } else {
        let _e29: i32 = paintType;
        if ((_e29 == 1)) {
            {
                let _e32: vec4<f32> = spriteAtlasColor;
                o_Color = _e32;
            }
        } else {
            let _e33: i32 = paintType;
            if ((_e33 == 2)) {
                {
                    let _e36: i32 = colorIndex;
                    let _e38: vec4<f32> = global.colors[_e36];
                    color = _e38;
                    let _e40: vec4<f32> = color;
                    let _e42: vec4<f32> = fontAtlasColor;
                    o_Color = vec4<f32>(_e40.xyz, _e42.x);
                    let _e46: vec4<f32> = o_Color;
                    let _e48: vec4<f32> = color;
                    o_Color.w = (_e46.w * _e48.w);
                }
            } else {
                let _e51: i32 = paintType;
                if ((_e51 == 3)) {
                    {
                        let _e54: i32 = colorIndex;
                        let _e56: vec4<f32> = global.colors[_e54];
                        colorA = _e56;
                        let _e58: i32 = colorIndex;
                        let _e62: vec4<f32> = global.colors[(_e58 + 1)];
                        colorB = _e62;
                        let _e64: i32 = colorIndex;
                        let _e68: vec4<f32> = global.colors[(_e64 + 2)];
                        point = _e68;
                        let _e70: vec4<f32> = point;
                        pointA = _e70.xy;
                        let _e73: vec4<f32> = point;
                        pointB = _e73.zw;
                        let _e76: vec2<f32> = f_WorldPos1;
                        let _e77: vec2<f32> = pointA;
                        ap = (_e76 - _e77);
                        let _e80: vec2<f32> = pointB;
                        let _e81: vec2<f32> = pointA;
                        ab = (_e80 - _e81);
                        let _e84: vec2<f32> = ab;
                        let _e86: vec2<f32> = ab;
                        let _e89: vec2<f32> = ab;
                        let _e91: vec2<f32> = ab;
                        ab2_ = ((_e84.x * _e86.x) + (_e89.y * _e91.y));
                        let _e96: vec2<f32> = ap;
                        let _e98: vec2<f32> = ab;
                        let _e101: vec2<f32> = ab;
                        let _e103: vec2<f32> = ap;
                        apAB = ((_e96.x * _e98.x) + (_e101.y * _e103.y));
                        let _e108: f32 = apAB;
                        let _e109: f32 = ab2_;
                        t = (_e108 / _e109);
                        let _e112: f32 = t;
                        t = clamp(_e112, 0.0, 1.0);
                        let _e116: vec4<f32> = colorA;
                        let _e118: f32 = t;
                        let _e122: vec4<f32> = colorB;
                        let _e123: f32 = t;
                        o_Color = ((_e116 * (f32(1) - _e118)) + (_e122 * _e123));
                    }
                }
            }
        }
    }
    let _e126: vec2<i32> = f_ScissorRect1;
    if ((_e126.x == 1)) {
        {
            let _e130: vec2<i32> = f_ScissorRect1;
            let _e133: vec4<f32> = global.colors[_e130.y];
            encodedRect = _e133;
            let _e135: vec4<f32> = encodedRect;
            rectPos = _e135.xy;
            let _e138: vec4<f32> = encodedRect;
            rectSize = _e138.zw;
            let _e141: vec2<f32> = f_WorldPos1;
            let _e143: vec2<f32> = rectPos;
            let _e146: vec2<f32> = f_WorldPos1;
            let _e148: vec2<f32> = rectPos;
            let _e152: vec2<f32> = f_WorldPos1;
            let _e154: vec2<f32> = rectPos;
            let _e156: vec2<f32> = rectSize;
            let _e161: vec2<f32> = f_WorldPos1;
            let _e163: vec2<f32> = rectPos;
            let _e165: vec2<f32> = rectSize;
            if (((((_e141.x < _e143.x) || (_e146.y < _e148.y)) || (_e152.x > (_e154.x + _e156.x))) || (_e161.y > (_e163.y + _e165.y)))) {
                {
                    o_Color.w = f32(0);
                    return;
                }
            } else {
                return;
            }
        }
    } else {
        return;
    }
}

[[stage(fragment)]]
fn main([[location(0)]] f_TexCoord: vec2<f32>, [[location(1)]] f_Paint: vec2<i32>, [[location(2)]] f_WorldPos: vec2<f32>, [[location(3)]] f_ScissorRect: vec2<i32>) -> FragmentOutput {
    f_TexCoord1 = f_TexCoord;
    f_Paint1 = f_Paint;
    f_WorldPos1 = f_WorldPos;
    f_ScissorRect1 = f_ScissorRect;
    main1();
    let _e29: vec4<f32> = o_Color;
    return FragmentOutput(_e29);
}
