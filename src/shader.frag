#version 450

layout(location=0) in vec3 v_color;
layout(location=1) in vec3 v_position;

layout(location=0) out vec4 f_color;

void main() {
    vec3 vRel = fract(v_position);
    if (any(lessThan(vec4(vRel, 1.0 - vRel), vec4(0.02)))) {
        f_color = vec4(0.0, 0.0, 0.0, 1.0);
    } else {
        f_color = vec4(v_color, 1.0);
    }
}