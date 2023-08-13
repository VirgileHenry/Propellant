#version 450
#extension GL_EXT_nonuniform_qualifier : enable

struct TexturedColor {
    vec3 color;
    uint textureId;
};

struct UiMaterial {
    TexturedColor color;
    float cornerRadius;
};

// layout(set = 0, binding = 0) uniform sampler2D all_textures[];

layout(set = 1, binding = 0) readonly buffer MaterialProperties {
    UiMaterial materials[];
} materialsProperties;

layout (location = 0) in flat int instanceIndex;
layout (location = 1) in smooth vec3 inPosition;
layout (location = 2) in smooth vec2 inUv;

layout (location = 0) out vec4 outColor;


void main() {
    UiMaterial material = materialsProperties.materials[instanceIndex];
    vec3 uiColor = material.color.color; // * texture(all_textures[material.color.textureId], inUv).rgb;
    outColor = vec4(uiColor, 1.0);
}


