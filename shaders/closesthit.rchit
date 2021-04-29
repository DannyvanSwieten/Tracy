#version 460
#extension GL_GOOGLE_include_directive : require
#extension GL_EXT_ray_tracing : require
#extension GL_EXT_nonuniform_qualifier : enable

#include "material.glsl"
#include "scatter.glsl"
#include "light.glsl"

layout(location = 0) rayPayloadInEXT RayPayload ray;

hitAttributeEXT vec3 attribs;

layout(binding = 0, set = 1) buffer Vertices { vec4 v[]; } vertices;
layout(binding = 1, set = 1) buffer Indices { uint i[]; } indices;
layout(binding = 3, set = 1) buffer Offsets { ivec2 o[]; } offsets;

struct Vertex
{
  vec3 pos;
  vec3 normal;
  vec2 uv;
};

Vertex unpack(uint index)
{
	const vec4 d0 = vertices.v[2 * index + 0];
	const vec4 d1 = vertices.v[2 * index + 1];

	Vertex v;
	v.pos = d0.xyz;
	v.normal = vec3(d0.w, d1.x, d1.y);
	v.uv = vec2(d1.z, d1.w);
	return v;
}

void main()
{
	const ivec2 o = offsets.o[gl_InstanceCustomIndexEXT];
	const ivec3 index = ivec3(indices.i[o.x + 3 * gl_PrimitiveID], indices.i[o.x + 3 * gl_PrimitiveID + 1], indices.i[o.x + 3 * gl_PrimitiveID + 2]) + o.y;

	const Vertex v0 = unpack(index.x);
	const Vertex v1 = unpack(index.y);
	const Vertex v2 = unpack(index.z);

	// Interpolate normal
	const vec3 barycentricCoords = vec3(1.0f - attribs.x - attribs.y, attribs.x, attribs.y);
	const vec3 N = normalize(gl_ObjectToWorldEXT * vec4(normalize(v0.normal * barycentricCoords.x + v1.normal * barycentricCoords.y + v2.normal * barycentricCoords.z), 0));
	const vec2 uv = normalize(v0.uv * barycentricCoords.x + v1.uv * barycentricCoords.y + v2.uv * barycentricCoords.z);

	ray.N = N;
	ray.uv = uv;
	ray.materialID = gl_InstanceCustomIndexEXT;
	ray.t = gl_HitTEXT;
}