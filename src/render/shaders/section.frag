#version 450

#include <colorspace.glsl>

layout(location=1) in vec2 v_pos;
layout(location=2) in vec3 v_color;
layout(location=3) in float border_radius;
layout(location=4) in float outline_radius;

layout(location=0) out vec4 f_color;

layout(set=0, binding=0) uniform RenderState {
    int width;
    int height;
    float t;
    float section_height;
};


float aa_bias = 1;

void main() {
    float alpha = 1;
    float outline = 0;

    vec2 pxy = v_pos * vec2(width, height);

    vec2 n = vec2(abs(pxy.x), pxy.y);
    float h = height * (section_height * 2 - 1);

    if (n.x >= width - border_radius && n.y >= h - border_radius) {
        float dist = distance(n, vec2(width - border_radius, h - border_radius));
        float border = border_radius - dist;
        float px_border = border * aa_bias;
        if (px_border < 0) {
            alpha = 0;
        } else if (px_border < 1) {
            alpha = px_border;
        }
    }

    float outline_x = 0;
    float px_dx = aa_bias * (n.x - (width - outline_radius));
    if (px_dx < 0) {
        outline_x = 0;
    } else if (px_dx < 1) {
        outline_x = px_dx;
    } else {
        outline_x = 1;
    }

    float outline_y = 0;
    float px_dy = aa_bias * (n.y - (h - outline_radius));
    if (px_dy < 0) {
        outline_y = 0;
    } else if (px_dy < 1) {
        outline_y = px_dy;
    } else {
        outline_y = 1;
    }

    float outline_r = 0;
    if (n.x >= width - border_radius && n.y >= h - border_radius) {
        float dist = distance(n, vec2(width - border_radius, h - border_radius));
        float px_outline = (dist - border_radius + outline_radius) * aa_bias;
        if (px_outline < 0) {
            outline_r = 0;
        } else if (px_outline < 1) {
            outline_r = px_outline;
        } else {
            outline_r = 1;
        }
    }

    outline = max(max(outline_x, outline_y), outline_r);

    vec4 c_outline = vec4(lab2rgb(0.1 * v_color), alpha);
    vec4 c_inline = vec4(lab2rgb(v_color), alpha);

    f_color = outline * c_outline + (1 - outline) * c_inline;
}

