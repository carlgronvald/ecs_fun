#version 330 core

layout(location=0) in vec2 vertexPosition_modelspace;
in vec3 incolor;
out vec3 vertexcolor;

void main() {
	gl_Position = vec4(vertexPosition_modelspace, 0, 1);

	vertexcolor = incolor;
}