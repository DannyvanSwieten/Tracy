#version 460

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(location = 3) in mat4 instanceTransform;
layout(location = 7) in vec4 instanceColor;
layout(location = 8) in vec4 instanceProperties;

layout(location = 0) out vec3 out_position;
layout(location = 1) out vec3 out_normal;
layout(location = 2) out vec4 out_instanceColor;
layout(location = 3) out vec4 out_instanceProperties;

layout(binding = 0, set = 0) uniform CameraProperties
{
    mat4 view;
    mat4 proj;
} cam;

layout(std430, push_constant) uniform PushConstants
{
    mat4 model;
} constants;

void main()
{
	gl_Position = cam.proj * cam.view * instanceTransform * constants.model * vec4(position, 1);
    out_position = (instanceTransform * constants.model * vec4(position, 1)).xyz;
    out_normal = mat3(instanceTransform * constants.model) * normal;
	out_instanceColor = instanceColor;
	out_instanceProperties = instanceProperties;
}
