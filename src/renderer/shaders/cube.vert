#version 450

layout(set = 0, binding = 0) uniform CameraUniform {
    mat4 view_proj;
};

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;

layout(location = 0) out vec3 v_color;

void main() {
    gl_Position = view_proj * vec4(position, 1.0);
    v_color = color;
}
