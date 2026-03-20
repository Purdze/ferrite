#version 450

layout(push_constant) uniform PushConstants {
    mat4 mvp;
};

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex_coords;
layout(location = 2) in float light;
layout(location = 3) in vec3 tint;

layout(location = 0) out vec2 v_uv;
layout(location = 1) out float v_light;
layout(location = 2) out vec3 v_tint;

void main() {
    gl_Position = mvp * vec4(position, 1.0);
    v_uv = tex_coords;
    v_light = light;
    v_tint = tint;
}
