#version 330 core

uniform sampler2D tex;

/*
in vec3 color;
in vec3 adjNormal;
in vec3 pos;
in vec3 lightDir;
*/
in vec2 texcoords;

out vec4 outColor;

const float ambient = 0.0;

void main() {
	/*
	vec3 ambient = color * ambient;

	vec3 diffuse = color * max(dot(-lightDir, adjNormal), 0.0);

	vec3 viewDir = normalize(-pos);
	vec3 reflectDir = reflect(viewDir, adjNormal);
	float spec = max(dot(reflectDir, lightDir), 0.0);
	if(dot(-lightDir, adjNormal) <= 0.0)
		spec = 0.0;
	vec3 specular = vec3(0.8) * pow(spec, 4);

	outColor = vec4(ambient + diffuse + specular, 1);
	*/
	outColor = texture(tex, texcoords);
}
