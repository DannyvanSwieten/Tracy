#version 460
#extension GL_EXT_ray_tracing : require
#extension GL_EXT_nonuniform_qualifier : enable
#extension GL_EXT_scalar_block_layout : enable
#extension GL_GOOGLE_include_directive : enable

#extension GL_EXT_shader_explicit_arithmetic_types_int32 : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require
#extension GL_EXT_buffer_reference2 : require

#include "ray_payload.glsl"
#include "material.glsl"
#include "scatter.glsl"
#include "random.glsl"
//#include "object.glsl"

hitAttributeEXT vec2 attribs;
layout(location = 0) rayPayloadInEXT RayPayload ray;

layout(buffer_reference, scalar) readonly buffer Vertices { vec3 data[]; };
layout(buffer_reference, scalar) readonly buffer Indices { int32_t data[]; };
layout(buffer_reference, scalar) readonly buffer TextureCoordinates { vec2 data[]; };
layout(buffer_reference, scalar) readonly buffer Offsets { ivec2 data[]; };
layout(buffer_reference, scalar) readonly buffer Materials { Material data[]; };

layout(binding = 1, set = 1) uniform BufferAddresses {
    uint64_t vertex_address;
    uint64_t index_address;
    uint64_t texcoord_address;
    uint64_t offset_address;
    uint64_t material_address;
};

layout(binding = 2, set = 1) uniform sampler2D images[1024];

void direction_of_anisotropicity(vec3 N, out vec3 tangent, out vec3 binormal){
    tangent = cross(N, vec3(1.,0.,1.));
    binormal = normalize(cross(N, tangent));
    tangent = normalize(cross(N, binormal));
}

void main()
{
    Materials materials = Materials(material_address);
    Offsets offsets = Offsets(offset_address);
    Indices indices = Indices(index_address);
    Vertices vertices = Vertices(vertex_address);
    TextureCoordinates tex_coords = TextureCoordinates(texcoord_address);

    const vec3 barycentric = vec3(1 - attribs.x - attribs.y, attribs.x, attribs.y);

    const int32_t start_index = 3 * gl_PrimitiveID + offsets.data[gl_InstanceCustomIndexEXT].x;
    const int32_t i0 = indices.data[start_index];
    const int32_t i1 = indices.data[start_index + 1];
    const int32_t i2 = indices.data[start_index + 2];

    const int32_t start_vertex = offsets.data[gl_InstanceCustomIndexEXT].y;
    const vec3 v0 = gl_ObjectToWorldEXT * vec4(vertices.data[start_vertex + i0], 1);
    const vec3 v1 = gl_ObjectToWorldEXT * vec4(vertices.data[start_vertex + i1], 1);
    const vec3 v2 = gl_ObjectToWorldEXT * vec4(vertices.data[start_vertex + i2], 1);

    const vec3 pv0 = barycentric.x * v0;
    const vec3 pv1 = barycentric.y * v1;
    const vec3 pv2 = barycentric.z * v2;

    const vec2 uv0 = barycentric.x * tex_coords.data[start_vertex + i0];
    const vec2 uv1 = barycentric.y * tex_coords.data[start_vertex + i1];
    const vec2 uv2 = barycentric.z * tex_coords.data[start_vertex + i2];

    const vec2 uv = uv0 + uv1 + uv2;

    const vec3 e10 = v1 - v0;
    const vec3 e20 = v2 - v0;
    const vec3 N = normalize(cross(e10, e20));
    ray.normal = N;

    vec2 Xi = vec2(rand_float(ray.seed), rand_float(ray.seed));

    int instance_id = gl_InstanceCustomIndexEXT;
    Material material = materials.data[instance_id];

    float pdf = 0.0;
    vec3 wi = vec3(0.0);
    vec3 wo = -gl_WorldRayDirectionEXT;
    vec3 base_color = material.albedo.rgb;
    if(materials.data[gl_InstanceCustomIndexEXT].maps[0] != -1)
    {
        base_color *= texture(images[material.maps[0]], uv).rgb;
    }
    float metal = material.metallic_roughness[1];
    float roughness = material.metallic_roughness[0];
    if(material.maps[1] != -1)
    {
        vec2 mr = texture(images[material.maps[1]], uv).bg;
        metal *= mr.x;
        roughness *= mr.y;
    }

    roughness = clamp(roughness, 0.001, 0.999);

    float anisotropy = 0.0;
    vec3 X = vec3(0.0);
    vec3 Y = vec3(0.0);
    direction_of_anisotropicity(N, X, Y);
    vec3 color_according_to_disney = sample_disney_bsdf(Xi, wi, wo, N, X, Y, base_color, roughness, metal, anisotropy, pdf);

    ray.hit = true;
    vec3 c = color_according_to_disney;
    if(pdf > 0.0001 && dot(c, c) > 0.0001)
        c /= pdf;
    else
        c = vec3(0);

    ray.color = vec4(c, 1);
    ray.emission = material.emission;
    if(material.maps[3] != -1)
    {
        ray.emission *= texture(images[material.maps[3]], uv);
    }

    c += ray.emission.rgb * ray.emission.a;
    ray.w_out = wi;
    ray.point = gl_WorldRayOriginEXT + gl_WorldRayDirectionEXT * gl_HitTEXT;
}