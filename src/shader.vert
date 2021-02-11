#version 450

layout(location = 0) in vec3 a_Pos;
layout(location = 1) in vec2 a_TexCoord;
layout(location = 2) in vec3 a_Normal;
layout(location = 0) out vec2 v_TexCoord;
layout(location = 1) out vec3 v_Normal;

layout(set = 0, binding = 0) uniform Locals {
    mat4 u_Transform;
};

void main() {
    v_TexCoord = a_TexCoord;
    v_Normal = a_Normal;
    gl_Position = u_Transform * vec4(a_Pos, 1.0);
}
