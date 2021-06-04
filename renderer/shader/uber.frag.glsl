#version 450

layout (location = 0) in vec2 f_TexCoord;
layout (location = 1) flat in ivec2 f_Paint;

// Shader constants stored in the `shader.x` vertex attribute.
#define PAINT_SOLID 0
#define PAINT_SPRITE 1
#define PAINT_ALPHA_TEXTURE 2 // used for fonts

layout (set = 0, binding = 1) uniform sampler u_Sampler;
layout (set = 0, binding = 2) uniform texture2D u_SpriteAtlas;
layout (set = 0, binding = 3) uniform texture2D u_FontAtlas;

layout (set = 0, binding = 4) buffer Colors {
    vec4 colors[];
};

layout (location = 0) out vec4 o_Color;

void main() {
    int paintType = f_Paint.x;
    int colorIndex = f_Paint.y;
    if (paintType == PAINT_SOLID) {
        o_Color = colors[colorIndex];
    } else if (paintType == PAINT_SPRITE) {
        o_Color = texture(sampler2D(u_SpriteAtlas, u_Sampler), f_TexCoord);
    } else if (paintType == PAINT_ALPHA_TEXTURE) {
        o_Color = colors[colorIndex]
            * texture(sampler2D(u_FontAtlas, u_Sampler), f_TexCoord).x;
    }
}
