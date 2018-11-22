#version 330 core

uniform bool textured;
uniform sampler2D tex;
uniform vec3 ambient;
uniform vec3 diffuse;

in vec3 adjNormal;
in vec3 lightDir;
in vec2 texcoords;

out vec4 outColor;


void main() {
	vec3 ambientC;
	vec3 diffuseC;

	if(textured) {
		ambientC = diffuseC = texture(tex, texcoords).rgb;
	} else {
		ambientC = ambient;
		diffuseC = diffuse;
	}

	ambientC *= 0.1;
	diffuseC *= max(dot(-lightDir, adjNormal), 0.0);

	outColor = vec4(ambientC + diffuseC, 1.0);
}
