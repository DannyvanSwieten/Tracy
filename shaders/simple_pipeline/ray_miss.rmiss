#version 460
#extension GL_GOOGLE_include_directive : require
#extension GL_EXT_ray_tracing : require

#include "ray_payload.glsl"
#include "constants.glsl"
layout(location = 0) rayPayloadInEXT RayPayload ray;
layout(set = 1, binding = 4) uniform sampler2D skybox_image;

vec2 direction_to_spherical(vec3 dir){
	float s = fract(1.0 / (2.0*M_PI) * atan(dir.y, -dir.x));
  	float t = 1.0 / (M_PI) * acos(-dir.z);
  	return vec2(s, t);
}

void main()
{
	const vec2 st = direction_to_spherical(gl_WorldRayDirectionEXT);
	const vec3 c = texture(skybox_image, st).rgb;
	ray.direct = c;
	ray.hit = false;
}
