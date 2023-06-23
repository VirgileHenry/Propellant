#version 450

layout(set = 1, binding = 0) uniform UniformCamera {
    mat4 proj;
    mat4 view;
} cam;

layout(set = 3, binding = 0) readonly buffer UniformModel {
    mat4 world_pos[];
} models;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inUv;

layout (location = 0) out int instanceIndex;
layout (location = 1) out vec3 outNormal;
layout (location = 2) out vec2 outUv;

void main() {
    instanceIndex = gl_InstanceIndex;
    outNormal = inNormal; // todo : transform normal to world space
    outUv = inUv;
    gl_Position = cam.proj * cam.view * models.world_pos[gl_InstanceIndex] * vec4(inPosition, 1.0);
}