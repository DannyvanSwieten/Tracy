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
#include "bsdf.glsl"
#include "random.glsl"

struct BufferAddresses {
    uint64_t index_address;
    uint64_t vertex_address;
    uint64_t normal_address;
    uint64_t tangent_address;
    uint64_t texcoord_address;
};

struct InstanceProperties{
    uint32_t geometry_id;
    uint32_t material_id;
};

layout(set = 0, binding = 0) uniform accelerationStructureEXT topLevelAS;

hitAttributeEXT vec2 attribs;
layout(location = 0) rayPayloadInEXT RayPayload ray;

layout(set = 1, binding = 1) uniform BufferAddressBuffer {
    uint64_t material_address;
    uint64_t instance_properties_address;
};

layout(set = 1, binding = 2, scalar) buffer AddressBuffer { BufferAddresses addresses[]; } meshes;
layout(set = 1, binding = 3) uniform sampler2D images[];

void direction_of_anisotropicity(vec3 N, out vec3 tangent, out vec3 binormal){
    tangent = cross(N, vec3(1.,0.,1.));
    binormal = normalize(cross(N, tangent));
    tangent = normalize(cross(N, binormal));
}

layout(buffer_reference, scalar) readonly buffer Vertices { vec3 data[]; };
layout(buffer_reference, scalar) readonly buffer Normals { vec3 data[]; };
layout(buffer_reference, scalar) readonly buffer Tangents { vec3 data[]; };
layout(buffer_reference, scalar) readonly buffer Indices { int32_t data[]; };
layout(buffer_reference, scalar) readonly buffer TextureCoordinates { vec2 data[]; };
layout(buffer_reference, scalar) readonly buffer Materials { Material data[]; };
layout(buffer_reference, scalar) readonly buffer InstanceIds { InstanceProperties data[]; };

void main()
{
    InstanceIds ids = InstanceIds(instance_properties_address);
    InstanceProperties properties = ids.data[gl_InstanceCustomIndexEXT];
    BufferAddresses mesh = meshes.addresses[properties.geometry_id];

    Materials materials = Materials(material_address);
    Normals normals = Normals(mesh.normal_address);
    Tangents tangents = Tangents(mesh.tangent_address);
    Indices indices = Indices(mesh.index_address);
    Vertices vertices = Vertices(mesh.vertex_address);
    TextureCoordinates tex_coords = TextureCoordinates(mesh.texcoord_address);

    const vec3 barycentric = vec3(1 - attribs.x - attribs.y, attribs.x, attribs.y);

    const int32_t start_index = 3 * gl_PrimitiveID;
    const int32_t i0 = indices.data[start_index];
    const int32_t i1 = indices.data[start_index + 1];
    const int32_t i2 = indices.data[start_index + 2];

    const vec3 v0 = gl_ObjectToWorldEXT * vec4(vertices.data[i0], 1);
    const vec3 v1 = gl_ObjectToWorldEXT * vec4(vertices.data[i1], 1);
    const vec3 v2 = gl_ObjectToWorldEXT * vec4(vertices.data[i2], 1);

    const vec3 pv0 = barycentric.x * v0;
    const vec3 pv1 = barycentric.y * v1;
    const vec3 pv2 = barycentric.z * v2;

    const vec2 uv0 = barycentric.x * tex_coords.data[i0];
    const vec2 uv1 = barycentric.y * tex_coords.data[i1];
    const vec2 uv2 = barycentric.z * tex_coords.data[i2];

    const vec2 uv = uv0 + uv1 + uv2;

    const vec3 n0 = gl_ObjectToWorldEXT * vec4(normals.data[i0], 0);
    const vec3 n1 = gl_ObjectToWorldEXT * vec4(normals.data[i1], 0);
    const vec3 n2 = gl_ObjectToWorldEXT * vec4(normals.data[i2], 0);

    const vec3 N0 = barycentric.x * n0;
    const vec3 N1 = barycentric.y * n1;
    const vec3 N2 = barycentric.z * n2;

    vec2 Xi = vec2(rand_float(ray.seed), rand_float(ray.seed));

    int instance_id = gl_InstanceCustomIndexEXT;
    Material material = materials.data[properties.material_id];

    const vec3 t0 = gl_ObjectToWorldEXT * vec4(tangents.data[i0], 0);
    const vec3 t1 = gl_ObjectToWorldEXT * vec4(tangents.data[i1], 0);
    const vec3 t2 = gl_ObjectToWorldEXT * vec4(tangents.data[i2], 0);

    const vec3 T0 = barycentric.x * t0;
    const vec3 T1 = barycentric.y * t1;
    const vec3 T2 = barycentric.z * t2;

    vec3 N = normalize(N0 + N1 + N2);
    const vec3 T = (T0 + T1 + T2) / 3;
    const vec3 B = cross(N, T);

    if(material.maps[2] != -1)
    {
        mat3 m = mat3(T, B, N);
        N = m *  (-1 + 0.5 * texture(images[material.maps[2]], uv).rgb);
        N = normalize(N);
    }

    ray.normal = N;
    vec3 base_color = material.base_color.rgb * material.base_color.a;

    if(material.maps[0] != -1)
    {
        vec4 base_color_texel = texture(images[material.maps[0]], uv);
        base_color = base_color_texel.rgb;// * (1.0 - base_color_texel.a) + base_color_texel.a * base_color;
    }
    base_color = pow(base_color, vec3(2.2));
    float metal = material.properties[1];
    float roughness = material.properties[0];
    if(material.maps[1] != -1)
    {
        vec2 mr = texture(images[material.maps[1]], uv).bg;
        metal *= mr.x;
        roughness *= mr.y;
    }

    vec3 random = random_pcg3d(uvec3(gl_LaunchIDEXT.xy, ray.seed));
    vec3 nextFactor = vec3(0);
    vec3 wo = normalize(-gl_WorldRayDirectionEXT);
    vec3 nextDir = sampleMicrofacetBRDF(wo, N, base_color, metal, 0.5, roughness, material.transmission.y, material.transmission.x, random, nextFactor);

    ray.hit = true;
    ray.color = vec4(max(nextFactor, 0), 1);
    ray.emission = material.emission;
    if(material.maps[3] != -1)
    {
        ray.emission = texture(images[material.maps[3]], uv);
    }

    ray.color.rgb += ray.emission.rgb * ray.emission.a;
    ray.direct = vec3(0);
    ray.w_out = nextDir;
    ray.point = gl_WorldRayOriginEXT + gl_WorldRayDirectionEXT * gl_HitTEXT;;
}