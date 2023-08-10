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

layout(set = 1, binding = 0) uniform MainDirectionnalLight {
    vec3 direction;
    float _padd_0;
    vec3 ambiant_color;
    float _padd_1;
    vec3 direct_color;
    float _padd_2;
} mainLight;

layout(set = 3, binding = 0) readonly buffer MaterialProperties {
    PhongMaterial materials[];
} materialsProperties;

layout (location = 0) in flat int instanceIndex;
layout (location = 1) in smooth vec3 inPosition;
layout (location = 2) in smooth vec3 inNormal;
layout (location = 3) in smooth vec2 inUv;
layout (location = 4) in smooth vec3 inCamPos;

layout (location = 0) out vec4 outColor;


void main() {

    vec4 albedo_tex = vec4(1.0); //texture(all_textures[nonuniformEXT(materialsProperties.materials[instanceIndex].albedo.textureId)], inUv);
    vec3 albedo = albedo_tex.rgb * materialsProperties.materials[instanceIndex].albedo.color;
    vec4 metalic_tex = vec4(1.0); //texture(all_textures[nonuniformEXT(materialsProperties.materials[instanceIndex].metalic.textureId)], inUv);
    float metalic = metalic_tex.r * materialsProperties.materials[instanceIndex].metalic.color.r;

    vec3 ambiant = mainLight.ambiant_color * albedo;
    vec3 diffuse = mainLight.direct_color * albedo * max(0.0, dot(inNormal, -mainLight.direction));

    vec3 viewDir = normalize(inPosition - inCamPos);
    vec3 reflectDir = reflect(-mainLight.direction, inNormal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 128);

    vec3 specular = 0.5 * spec * mainLight.direct_color * metalic; 
      
    vec3 result = ambiant + diffuse + specular;
    outColor = vec4(result, 1.0);
}


