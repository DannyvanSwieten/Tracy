#version 460

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 outputColor;

layout(binding = 0) uniform sampler2D image;

void main()
{
	outputColor = texture(image, uv);
}