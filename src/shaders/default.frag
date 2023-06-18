#version 450

struct PbrMaterial {
    vec3 baseColor;
    float metallic;
    float roughness;
    uint _; // texture id (hash)
    uint textureIndex;
    uint __; // padding to reach 32 bytes
};

layout(set = 2, binding = 0) readonly buffer MaterialProperties {
    PbrMaterial materials[];
} materialsProperties;

layout (location = 0) in flat int instanceIndex;
layout(location = 0) out vec4 outColor;

void main() {
    outColor = vec4(materialsProperties.materials[instanceIndex].baseColor, 1);
}