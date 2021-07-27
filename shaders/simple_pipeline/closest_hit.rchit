#version 460
#extension GL_EXT_ray_tracing : require
#extension GL_EXT_nonuniform_qualifier : enable
#extension GL_EXT_scalar_block_layout : enable
#extension GL_GOOGLE_include_directive : enable

#extension GL_EXT_shader_explicit_arithmetic_types_int32 : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require
#extension GL_EXT_buffer_reference2 : require

#include "ray_payload.glsl"

struct Material
{
    vec4 albedo;
    vec4 emission;
    ivec4 maps;
};

hitAttributeEXT vec2 attribs;
layout(location = 0) rayPayloadInEXT RayPayload ray;

layout(buffer_reference, scalar) readonly buffer Vertices { vec3 data[]; };
layout(buffer_reference, scalar) readonly buffer Indices { int32_t data[]; };
layout(buffer_reference, scalar) readonly buffer Offsets { ivec4 data[]; };
layout(buffer_reference, scalar) readonly buffer Materials { Material data[]; };

layout(binding = 1, set = 1) uniform BufferAddresses {
    uint64_t vertex_address;
    uint64_t index_address;
    uint64_t offset_address;
    uint64_t material_address;
};

void main()
{
    Materials materials = Materials(material_address);
    Offsets offsets = Offsets(offset_address);
    Indices indices = Indices(index_address);
    Vertices vertices = Vertices(vertex_address);

    const vec3 barycentric = vec3(1 - attribs.x - attribs.y, attribs.x, attribs.y);

    const int32_t start_index = 3 * gl_PrimitiveID + offsets.data[gl_InstanceCustomIndexEXT].y;
    const int32_t i0 = indices.data[start_index];
    const int32_t i1 = indices.data[start_index + 1];
    const int32_t i2 = indices.data[start_index + 2];

    const vec3 v0 = vertices.data[i0];
    const vec3 v1 = vertices.data[i1];
    const vec3 v2 = vertices.data[i2];

    const vec3 pv0 = barycentric.x * v0;
    const vec3 pv1 = barycentric.y * v1;
    const vec3 pv2 = barycentric.z * v2;

    const vec3 e10 = v1 - v0;
    const vec3 e20 = v2 - v0;
    const vec3 N = normalize(cross(e10, e20));

    const vec3 L = normalize(vec3(1));
    const float I = max(0, dot(L, N));

    ray.hit = true;
    ray.color = materials.data[gl_InstanceCustomIndexEXT].albedo * .5;
    ray.normal = N;
    ray.point = gl_WorldRayOriginEXT + gl_WorldRayDirectionEXT * gl_HitTEXT;
}