#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout(set = 2, binding = 0) uniform sampler2D all_textures[];

layout (location = 0) in flat int instanceIndex;
layout (location = 1) in smooth vec3 inPosition;
layout (location = 2) in smooth vec2 inUv;
layout (location = 3) in flat int textureId;
layout (location = 4) in smooth vec3 color;

layout (location = 0) out vec4 outColor;

void main() {
    vec4 texture = texture(all_textures[textureId], inUv);
    outColor = vec4(texture.rgb * color, texture.a);
}


