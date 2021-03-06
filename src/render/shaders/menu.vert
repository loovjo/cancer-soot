#version 450

#include <colorspace.glsl>

layout(set=0, binding=0) uniform RenderState {
    int width;
    int height;
    float t;
    float section_height;
};

const int i2j[6] = {0, 1, 2, 2, 3, 0};

const vec2 positions[4] = vec2[4](
    vec2(-1, -1),
    vec2(1, -1),
    vec2(1, 0),
    vec2(-1, 0)
);

// LAB
vec3 colors[4] = vec3[4](
    vec3(125, 30, 227),
    vec3(54, 23, 230),
    vec3(247, 101, 204),
    vec3(219, 18, 34)
);

layout(location=1) out vec2 v_pos;
layout(location=2) out vec3 v_color;
layout(location=3) out float border_radius;
layout(location=4) out float outline_radius;

void main() {
    vec3 k = vec3(0, 0, 0);
    vec3 m = vec3(0, 0, 0);

    float h = section_height * 2 - 1;

    int j = i2j[gl_VertexIndex];

    if (j == 0) {
        k = vec3(0.478, 0.358, 0.399);
        m = vec3(0.559, 0.186, 0.672);

        v_pos = vec2(-1, h - 2);
    }
    if (j == 1) {
        k = vec3(0.678, 0.809, 0.079);
        m = vec3(0.589, 0.816, 0.269);

        v_pos = vec2(1, h - 2);
    }
    if (j == 2) {
        k = vec3(0.805, 0.972, 0.057);
        m = vec3(0.878, 0.057, 0.732);

        v_pos = vec2(1, h);
    }
    if (j == 3) {
        k = vec3(0.88,  0.742, 0.263);
        m = vec3(0.159, 0.311, 0.116);

        v_pos = vec2(-1, h);
    }

    vec3 delta = sin(k + m * t);

    v_color = rgb2lab(colors[j] / 256 + 0.2 * delta);

    border_radius = 30;
    outline_radius = 10;

    gl_Position = vec4(v_pos, 0.0, 1.0);
}
