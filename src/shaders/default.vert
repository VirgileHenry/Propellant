#version 450

layout(binding = 0) uniform UniformCamera {
    mat4 proj_view;
} cam;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inUv;

void main() {
    gl_Position = cam.proj_view * vec4(inPosition, 1.0);
}