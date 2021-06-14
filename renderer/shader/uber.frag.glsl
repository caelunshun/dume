#version 450

layout (location = 0) in vec2 f_TexCoord;
layout (location = 1) flat in ivec2 f_Paint;

// Shader constants stored in the `shader.x` vertex attribute.
#define PAINT_SOLID 0
#define PAINT_SPRITE 1
#define PAINT_ALPHA_TEXTURE 2 // used for fonts

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
    }
}
