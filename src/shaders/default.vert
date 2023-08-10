#version 450

layout(set = 0, binding = 0) uniform UniformCamera {
    mat4 proj;
    mat4 view;
} cam;

layout(set = 2, binding = 0) readonly buffer UniformModel {
    mat4 world_pos[];
} models;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inUv;

layout (location = 0) out int instanceIndex;
layout (location = 1) out vec3 outPosition;
layout (location = 2) out vec3 outNormal;
layout (location = 3) out vec2 outUv;
layout (location = 4) out vec3 outCamPos;

void main() {
    instanceIndex = gl_InstanceIndex;
    outPosition = (models.world_pos[gl_InstanceIndex] * vec4(inPosition, 1.0)).xyz;
    outNormal = transpose(inverse(mat3(models.world_pos[gl_InstanceIndex]))) * inNormal;
    outUv = inUv;
    outCamPos = (inverse(cam.view) * vec4(0.0, 0.0, 0.0, 1.0)).xyz;
    gl_Position = cam.proj * cam.view * models.world_pos[gl_InstanceIndex] * vec4(inPosition, 1.0);
}