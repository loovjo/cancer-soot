#version 450

layout(set=0, binding=0) uniform RenderState {
    int width;
    int height;
    float t;

    float section_height;
};

const int i2j[6] = {0, 1, 2, 2, 3, 0};

const vec2 positions[4] = vec2[4](
    vec2(-0.7, -0.6),
    vec2(0.7, -0.6),
    vec2(0.7, 0.8),
    vec2(-0.7, 0.8)
);

const vec2 uvs[4] = vec2[4](
    vec2(0, 0),
    vec2(1, 0),
    vec2(1, 1),
    vec2(0, 1)
);

layout(location=1) out vec2 v_pos;
layout(location=2) out vec2 uv;

void main() {
    int j = i2j[gl_VertexIndex];

    gl_Position = vec4(positions[j], 0.0, 1.0);
    uv = uvs[j];
}

