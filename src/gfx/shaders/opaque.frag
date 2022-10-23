#version 450

layout(location = 0) in vec3 inNormal;

layout(location = 0) out vec4 outColor;

void main() {
    vec3 lightDir = normalize(vec3(1.0));
    float intensity = max(dot(lightDir, inNormal), 0.0);
    outColor = vec4(vec3(intensity), 1.0);
}