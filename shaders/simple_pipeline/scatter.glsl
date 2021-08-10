#include "constants.glsl"

mat3 make_rotation_matrix_z(const vec3 z)
{
	const vec3 ref = abs(dot(z, vec3(0, 1, 0))) > 0.99f ? vec3(0, 0, 1) : vec3(0, 1, 0);

	const vec3 x = normalize(cross(ref, z));
	const vec3 y = cross(z, x);

	return mat3(x, y, z);
}

vec3 sample_lambert_brdf(const vec3 N, inout vec3 L, const vec3 V, const vec2 Xi)
{
	const float phi = 2 * M_PI * Xi.y;
	const float sinTheta = sqrt(Xi.x);
	const float cosTheta = sqrt(1 - Xi.x);

	L = make_rotation_matrix_z(N)* vec3(sinTheta * cos(phi), sinTheta * sin(phi), cosTheta);

	return vec3(M_1_PI);
}