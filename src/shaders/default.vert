#version 450

layout(set = 0, binding = 0) uniform UniformCamera {
    mat4 proj_view;
} cam;

layout(set = 1, binding = 0) readonly buffer UniformModel {
    mat4 world_pos[];
} models;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inUv;

void main() {
    gl_Position = cam.proj_view * models.world_pos[gl_InstanceIndex] * vec4(inPosition, 1.0);
}