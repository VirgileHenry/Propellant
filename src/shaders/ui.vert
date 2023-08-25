#version 450

struct UiTransform {
    mat3 tfMat;
};

layout(set = 1, binding = 0) readonly buffer UiTransforms {
    UiTransform uiTransforms[];
} uiTransforms;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inUv;

layout (location = 0) out int instanceIndex;
layout (location = 1) out vec3 outPosition;
layout (location = 2) out vec2 outUv;

void main() {
    instanceIndex = gl_InstanceIndex;
    UiTransform tf = uiTransforms.uiTransforms[gl_InstanceIndex];
    
    vec3 pos = tf.tfMat * inPosition;

    outPosition = pos;
    outUv = inUv;
    gl_Position = vec4(pos, 1.0);
}