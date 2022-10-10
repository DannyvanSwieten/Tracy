#include "payload.glsl"
#include "random.glsl"
#include "constants.glsl"

float ggxNormalDistribution(float NdotH, float roughness)
{
	float a2 = roughness * roughness;
	float NdotH2 = NdotH * NdotH;
	float d = (NdotH2 * (a2 - 1) + 1);

	return a2 / (d * d * M_PI);
}

float schlickMaskingTerm(float NdotL, float NdotV, float roughness)
{
	// Karis notes they use alpha / 2 (or roughness^2 / 2)
	float k = (roughness * roughness) / 2;

	// Compute G(v) and G(l).  These equations directly from Schlick 1994
	//     (Though note, Schlick's notation is cryptic and confusing.)
	float g_v = NdotV / (NdotV * (1 - k) + k);
	float g_l = NdotL / (NdotL * (1 - k) + k);
	return g_v * g_l;
}

vec3 schlickFresnel(const vec3 f0, float lDotH)
{
	return f0 + (1 - f0) * pow(1.0f - lDotH, 5.0f);
}

vec3 ImportanceSampleGGX(vec2 Xi, vec3 N, float roughness)
{
	float a = roughness * roughness;

	float phi = 2.0 * M_PI * Xi.x;
	float cosTheta = sqrt((1.0 - Xi.y) / (1.0 + (a * a - 1.0) * Xi.y));
	float sinTheta = sqrt(1.0 - cosTheta * cosTheta);

	// from spherical coordinates to cartesian coordinates
	vec3 H;
	H.x = cos(phi) * sinTheta;
	H.y = sin(phi) * sinTheta;
	H.z = cosTheta;

	// from tangent-space vector to world-space sample vector
	vec3 up = abs(N.z) < 0.999 ? vec3(0.0, 0.0, 1.0) : vec3(1.0, 0.0, 0.0);
	vec3 tangent = normalize(cross(up, N));
	vec3 bitangent = cross(N, tangent);

	vec3 sampleVec = tangent * H.x + bitangent * H.y + N * H.z;
	return normalize(sampleVec);
}

vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
	return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

float saturate(const float v)
{
	return clamp(0, 1, v);
}

// Polynomial approximation by Christophe Schlick
float schlick(const float cosTheta, const float refractionIndex)
{
	float r0 = (1 - refractionIndex) / (1 + refractionIndex);
	r0 *= r0;
	return r0 + (1 - r0) * pow(1 - cosTheta, 5);
}

vec3 safe_normalize(const vec3 V, const vec3 fallback)
{
	const float l = length(V);
	return l > 0 ? V / l : fallback;
}

mat3 make_rotation_matrix_y(const vec3 y)
{
	const vec3 ref = abs(dot(y, vec3(0, 1, 0))) > 0.99f ? vec3(0, 0, 1) : vec3(0, 1, 0);

	const vec3 x = normalize(cross(y, ref));
	const vec3 z = cross(x, y);

	return mat3( x, y, z );
}

mat3 make_rotation_matrix_z(const vec3 z)
{
	const vec3 ref = abs(dot(z, vec3(0, 1, 0))) > 0.99f ? vec3(0, 0, 1) : vec3(0, 1, 0);

	const vec3 x = normalize(cross(ref, z));
	const vec3 y = cross(z, x);

	return mat3(x, y, z);
}

vec3 sample_lambert_brdf(const Material m, const vec3 N, inout vec3 L, const vec3 V, const vec2 Xi)
{
	const float phi = 2 * M_PI * Xi.y;
	const float sinTheta = sqrt(Xi.x);
	const float cosTheta = sqrt(1 - Xi.x);

	L = make_rotation_matrix_z(N)* vec3(sinTheta * cos(phi), sinTheta * sin(phi), cosTheta);

	return vec3(M_1_PI) * m.albedo.rgb;
}