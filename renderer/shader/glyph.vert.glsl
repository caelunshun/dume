#version 450

layout (location = 0) in vec2 a_Pos;
layout (location = 1) in vec2 a_TexCoord;
layout (location = 2) in vec4 a_ColorLinear;

layout (set = 0, binding = 0) uniform Uniforms {
    mat4 u_Ortho;
};

layout (location = 0) out vec2 f_TexCoord;
layout (location = 1) out vec4 f_ColorLinear;

void main() {
    gl_Position = u_Ortho * vec4(a_Pos, 0.0, 1.0);
    f_TexCoord = a_TexCoord;
    f_ColorLinear = a_ColorLinear;
}
