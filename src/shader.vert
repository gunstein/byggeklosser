#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec2 a_tex_coords;

layout(location=0) out vec3 v_color;
layout(location=1) out vec3 v_position;

layout(set=1, binding=0) 
uniform Uniforms {
    vec3 u_view_position; 
    mat4 u_view_proj;
};

layout(location=5) in mat4 model_matrix;
layout(location=9) in vec3 a_color;

void main() {
    v_color = a_color;
    v_position = a_position;
    gl_Position = u_view_proj * model_matrix * vec4(a_position, 1.0);
}