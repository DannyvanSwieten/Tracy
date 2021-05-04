#version 460
#extension GL_EXT_ray_tracing : require
#extension GL_GOOGLE_include_directive : require

#include "ray_payload.glsl"

layout(location = 0) rayPayloadInEXT RayPayload ray;

void main()
{
    ray.hit = true;
}