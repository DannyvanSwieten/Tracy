#version 460

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 outputColor;

layout(binding = 0) uniform sampler2D image_a;
layout(binding = 1) uniform sampler2D image_b;

void main()
{
	outputColor = texture(image_a, uv) + texture(image_b, uv);
}