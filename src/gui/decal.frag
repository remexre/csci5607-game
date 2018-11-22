#version 330 core

uniform sampler2D decal;
in vec2 texcoords;
out vec4 outColor;

void main() {
	outColor = texture(decal, texcoords);
}
