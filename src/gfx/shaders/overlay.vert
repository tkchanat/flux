#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec2 inTexcoord;

layout(location = 0) out vec2 outTexcoord;

void main() {
    gl_Position = vec4(inPosition, 1.0);
    outTexcoord = inTexcoord;
}
