#version 450

struct UiTransform {
    vec2 position;
    vec2 relative_position;
    vec2 size;
    vec2 relative_size;
    vec2 anchor;
    int layer;
};

layout(set = 0, binding = 0) uniform UiResolution {
    float res;
    float screen_width;
    float screen_height;
} uiResolution;

layout(set = 2, binding = 0) readonly buffer UiTransforms {
    UiTransform uiTransforms[];
} uiTransforms;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inUv;

layout (location = 0) out int instanceIndex;
layout (location = 1) out vec3 outPosition;
layout (location = 2) out vec2 outUv;

const float PI_INVERSE = 1.0 / 3.1415926535897932384626433832795;

void main() {
    instanceIndex = gl_InstanceIndex;
    // this shader will receive a [0, 1] x [0, 1] quad and need to resize it accoding to the model matrix !
    float r = uiResolution.res;
    UiTransform tf = uiTransforms.uiTransforms[gl_InstanceIndex];
    float tx = tf.position.x * r / uiResolution.screen_width + tf.relative_position.x;
    float ty = tf.position.y * r / uiResolution.screen_height + tf.relative_position.y;
    float tw = tf.size.x * r / uiResolution.screen_width + tf.relative_size.x;
    float th = tf.size.y * r / uiResolution.screen_height + tf.relative_size.y;
    float ax = tf.anchor.x;
    float ay = tf.anchor.y;
    float px = tx + (inPosition.x - ax) * tw;
    float py = ty + (inPosition.y - ay) * th;
    // x2-1 allows to go from [0, 1] to [-1, 1] (vulkan render target is [-1, 1])
    float depth = 0.5 - PI_INVERSE * atan(tf.layer);
    vec3 pos = vec3(px * 2 - 1, py * 2 - 1, depth);

    outPosition = pos;
    outUv = inUv;
    gl_Position = vec4(pos, 1.0);
}