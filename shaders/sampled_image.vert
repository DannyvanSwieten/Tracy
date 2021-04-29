#version 460

layout(location = 0) out vec2 uv;

const vec2 vertices[6] = vec2[](
	vec2(-1, 1), vec2(1, -1), vec2(-1, -1),
	vec2(-1, 1), vec2(1, 1), vec2(1, -1)
);

void main()
{
	gl_Position = vec4(vertices[gl_VertexIndex], 0, 1);
	uv = 0.5 * gl_Position.xy + 0.5;
}