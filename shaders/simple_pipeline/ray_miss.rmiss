#version 460
#extension GL_GOOGLE_include_directive : require
#extension GL_EXT_ray_tracing : require

#include "ray_payload.glsl"

layout(location = 0) rayPayloadInEXT RayPayload ray;

void main()
{
	const vec3 d = normalize(gl_WorldRayDirectionEXT);
	const float t = 0.5 * (d.y + 1.0);
	ray.color = vec4((1.0 - t) * vec3(.75) + t * vec3(0.5, 0.7, 1.0), 1);
	ray.hit = false;
}
