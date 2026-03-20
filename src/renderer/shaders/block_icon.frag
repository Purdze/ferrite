#version 450

layout(set = 0, binding = 0) uniform sampler2D atlas_tex;

layout(location = 0) in vec2 v_uv;
layout(location = 1) in float v_light;
layout(location = 2) in vec3 v_tint;

layout(location = 0) out vec4 out_color;

void main() {
    vec4 color = texture(atlas_tex, v_uv);
    if (color.a < 0.01) discard;
    vec3 lit = color.rgb * v_tint * v_light;
    out_color = vec4(lit * color.a, color.a);
}
