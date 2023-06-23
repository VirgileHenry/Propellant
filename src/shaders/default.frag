#version 450
#extension GL_EXT_nonuniform_qualifier : enable

struct TexturedColor {
    vec3 color;
    uint textureId;
};

struct PhongMaterial {
    TexturedColor albedo;
    TexturedColor metalic;
};

layout(set = 0, binding = 0) uniform sampler2D all_textures[];

layout(set = 2, binding = 0) uniform MainDirectionnalLight {
    vec3 direction;
    vec3 ambiant_color;
    vec3 diffuse_color;
} mainLight;

layout(set = 4, binding = 0) readonly buffer MaterialProperties {
    PhongMaterial materials[];
} materialsProperties;

layout (location = 0) in flat int instanceIndex;
layout (location = 1) in vec3 inNormal;
layout (location = 2) in vec2 inUV;

layout (location = 0) out vec4 outColor;




void main() {
    vec4 albedo_tex = texture(all_textures[nonuniformEXT(materialsProperties.materials[instanceIndex].albedo.textureId)], inUV);
    vec3 albedo = albedo_tex.rgb * materialsProperties.materials[instanceIndex].albedo.color;
    outColor = vec4(albedo, 1);
}


