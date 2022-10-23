#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inTexcoord;

layout(location = 0) out vec3 outNormal;

layout(set = 0, binding = 0) uniform CameraUniform {
    mat4 view;
    mat4 projection;
} camera;
layout(push_constant) uniform constants {
    mat4 model;
} pc;

void main() {
    gl_Position = camera.projection * camera.view * pc.model * vec4(inPosition, 1.0);
    outNormal = inNormal;
}