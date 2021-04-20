#version 450

layout(location=0) in vec3 a_pos;
layout(location=1) in vec3 a_color;


layout(location=0) out vec3 v_color;

void main(){
    v_color = a_pos;
    gl_Position = vec4(a_pos, 1.0);
}