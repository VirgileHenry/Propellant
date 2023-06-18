#version 450

struct TexturedColor {
    vec3 color;
    uint textureId;
};

struct PbrMaterial {
    TexturedColor albedo;
    TexturedColor metalic;
};

layout(set = 1, binding = 0) uniform MainDirectionnalLight {
    vec3 direction;
    vec3 ambiant_color;
    vec3 diffuse_color;
} mainLight;

layout(set = 3, binding = 0) readonly buffer MaterialProperties {
    PbrMaterial materials[];
} materialsProperties;

layout (location = 0) in flat int instanceIndex;
layout(location = 0) out vec4 outColor;




void main() {
    
    outColor = vec4(materialsProperties.materials[instanceIndex].albedo.color, 1);
}