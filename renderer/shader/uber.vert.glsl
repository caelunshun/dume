#version 450

layout (location = 0) in vec2 a_Pos;
layout (location = 1) in vec2 a_TexCoord;
layout (location = 2) in ivec2 a_Paint;

layout (location = 0) out vec2 f_TexCoord;
layout (location = 1) flat out ivec2 f_Paint;

layout (set = 0, binding = 0) uniform Locals {
    mat4 u_Ortho;
};

void main() {
    gl_Position = u_Ortho * vec4(a_Pos, 0.0, 1.0);
    f_TexCoord = a_TexCoord;
    f_Paint = a_Paint;
}
