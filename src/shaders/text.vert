#version 450

struct TextMaterial {
    vec2 minUv;
    vec2 maxUv;
    mat3 tfMat;
    vec3 color;
    int textureId;
};

struct UiTransform {
    mat3 tfMat;
};

layout(set = 0, binding = 0) readonly buffer MaterialProperties {
    TextMaterial materials[];
} materialsProperties;

layout(set = 1, binding = 0) readonly buffer UiTransforms {
    UiTransform uiTransforms[];
} uiTransforms;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inUv;

layout (location = 0) out int instanceIndex;
layout (location = 1) out vec3 outPosition;
layout (location = 2) out vec2 outUv;
layout (location = 3) flat out int textureId;
layout (location = 4) out vec3 outColor;

void main() {
    instanceIndex = gl_InstanceIndex;
    UiTransform parentTf = uiTransforms.uiTransforms[gl_InstanceIndex];
    TextMaterial material = materialsProperties.materials[gl_InstanceIndex];
    
    vec3 pos = parentTf.tfMat * material.tfMat * inPosition;
    // this allows to be renderer slightly over uimaterial
    pos.z -= 0.0000001;

    outPosition = pos;
    outUv = material.minUv + inUv * (material.maxUv - material.minUv);
    textureId = material.textureId;
    outColor = material.color;
    gl_Position = vec4(pos, 1.0);
}