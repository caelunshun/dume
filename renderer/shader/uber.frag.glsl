#version 450

layout (location = 0) in vec2 f_TexCoord;
layout (location = 1) flat in ivec2 f_Paint;
layout (location = 2) in vec2 f_WorldPos;

// Shader constants stored in the `shader.x` vertex attribute.
#define PAINT_SOLID 0
#define PAINT_SPRITE 1
#define PAINT_ALPHA_TEXTURE 2 // used for fonts
#define PAINT_LINEAR_GRADIENT 3

layout (set = 0, binding = 1) uniform sampler u_Sampler;
layout (set = 0, binding = 2) uniform sampler u_NearestSampler;
layout (set = 0, binding = 3) uniform texture2D u_SpriteAtlas;
layout (set = 0, binding = 4) uniform texture2D u_FontAtlas;

layout (set = 0, binding = 5) buffer Colors {
    vec4 colors[];
};

layout (location = 0) out vec4 o_Color;

// Converts a color from sRGB gamma to linear light gamma
vec3 toLinear(vec3 sRGB) {
    bvec3 cutoff = lessThan(sRGB, vec3(0.04045));
    vec3 higher = pow((sRGB + vec3(0.055))/vec3(1.055), vec3(2.4));
    vec3 lower = sRGB/vec3(12.92);

    return mix(higher, lower, cutoff);
}

void main() {
    int paintType = f_Paint.x;
    int colorIndex = f_Paint.y;
    if (paintType == PAINT_SOLID) {
        o_Color = colors[colorIndex];
    } else if (paintType == PAINT_SPRITE) {
        o_Color = texture(sampler2D(u_SpriteAtlas, u_Sampler), f_TexCoord);
    } else if (paintType == PAINT_ALPHA_TEXTURE) {
        o_Color = vec4(colors[colorIndex].xyz, texture(sampler2D(u_FontAtlas, u_NearestSampler), f_TexCoord).x);
    } else if (paintType == PAINT_LINEAR_GRADIENT) {
        vec4 colorA = colors[colorIndex];
        vec4 colorB = colors[colorIndex + 1];

        vec4 point = colors[colorIndex + 2];
        vec2 pointA = point.xy;
        vec2 pointB = point.zw;

        // https://stackoverflow.com/questions/1459368/snap-point-to-a-line
        vec2 ap = f_WorldPos - pointA;
        vec2 ab = pointB - pointA;

        float ab2 = ab.x * ab.x + ab.y * ab.y;
        float apAB = ap.x * ab.x + ab.y * ap.y;
        float t = apAB / ab2;
        t = clamp(t, 0.0, 1.0);

        o_Color = colorA * (1 - t) + colorB * t;
    }
}
