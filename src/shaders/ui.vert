#version 450

layout(set = 1, binding = 0) uniform UiResolution {
    float res;
    float screen_width;
    float screen_height;
} uiResolution;

layout(set = 2, binding = 0) readonly buffer UniformModel {
    // this matrix holds ui parameters in the following order:
    // x, rx, 0, 0, -> pos x and relative x
    // y, ry, 0, 0, -> pos y and relative y
    // w, rw, 0, 0, -> width and relative width
    // h, rh, 0, 0, -> height and relative height
    mat4 world_pos[];
} models;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inUv;

layout (location = 0) out int instanceIndex;
layout (location = 1) out vec3 outPosition;
layout (location = 2) out vec2 outUv;

void main() {
    instanceIndex = gl_InstanceIndex;
    // this shader will receive a [0, 1] x [0, 1] quad and need to resize it accoding to the model matrix !
    mat4 tf = models.world_pos[gl_InstanceIndex];
    float tx = tf[0][0] / uiResolution.screen_width + tf[1][0];
    float ty = tf[0][1] / uiResolution.screen_height + tf[1][1];
    float tw = tf[0][2] / uiResolution.screen_width + tf[1][2];
    float th = tf[0][3] / uiResolution.screen_height + tf[1][3];
    float ax = tf[3][0];
    float ay = tf[3][1];
    float px = tx + (inPosition.x - ax) * tw;
    float py = ty + (inPosition.y - ay) * th;
    // x2-1 allows to go from [0, 1] to [-1, 1] (vulkan render target is [-1, 1])
    vec3 pos = vec3(px * 2 - 1, py * 2 - 1, 0.0);

    outPosition = pos;
    outUv = inUv;
    gl_Position = vec4(pos, 1.0);
}