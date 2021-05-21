#version 450

layout(location=1) in vec2 v_pos;
layout(location=2) in vec2 uv;

layout(location=0) out vec4 f_color;

layout(set=0, binding=0) uniform RenderState {
    int width;
    int height;
    float t;
    float section_height;
};

layout(set = 1, binding = 0) uniform texture2D t_diffuse;
layout(set = 1, binding = 1) uniform sampler s_diffuse;

void main() {
    float amount = texture(sampler2D(t_diffuse, s_diffuse), uv).x * 10;
    vec4 col;
    if (uv.x < 0 || uv.x > 1 || uv.y < 0 || uv.y > 1) {
        col = vec4(1, 1, 1, 1);
    } else {
        col = vec4(min(amount * 3, 1), min(amount * 3 - 1, 1), min(amount * 3 - 2, 1), 1);
    }
    f_color = col;
}
