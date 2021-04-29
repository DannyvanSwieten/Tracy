#version 460
#extension GL_GOOGLE_include_directive : require

#include "material.glsl"
#include "scatter.glsl"

layout(location = 0) in vec3 out_position;
layout(location = 1) in vec3 out_normal;
layout(location = 2) in vec4 out_instanceColor;
layout(location = 3) in vec4 out_instanceProperties;

layout(location = 0) out vec4 outputColor;

const vec3 light = vec3(100, 100, -100);

layout(std430, push_constant) uniform PushConstants
{
    layout(offset = 64) vec3 position;
} cameraSettings;

void main()
{
    vec3 N = normalize(out_normal);
    vec3 L = normalize(light);
    vec3 V = normalize(cameraSettings.position - out_position);
    vec3 H = normalize(V + L);
    
    vec4 m = out_instanceProperties;
    
    float NdotH = saturate(dot(N, H));
    float NdotL = saturate(dot(N, L));
    float NdotV = saturate(dot(N, V));
    float LdotH = saturate(dot(L, H));
    
    float D = ggxNormalDistribution(NdotH, m.r);
    float G = schlickMaskingTerm(NdotL, NdotV, m.r);
    vec3 F = schlickFresnel(vec3(0.04), LdotH);
    
    vec3 ggx = D * G * F / (4 * NdotV);
    vec3 diff = out_instanceColor.rgb * M_1_PI * NdotL;
    vec3 color = diff + ggx;
    
    outputColor = vec4(color, 1);
	outputColor = pow(outputColor, vec4(1) / vec4(2.2));
}
