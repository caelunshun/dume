#version 450

layout (location = 0) in vec2 f_TexCoord;
layout (location = 1) in vec4 f_ColorLinear;

layout (location = 0) out vec4 o_Color;

layout (set = 0, binding = 1) uniform sampler u_Sampler;
layout (set = 0, binding = 2) uniform texture2D u_FontAtlas;

void main() {
    float alpha = texture(sampler2D(u_FontAtlas, u_Sampler), f_TexCoord).r;
    o_Color = alpha * f_ColorLinear;
}
