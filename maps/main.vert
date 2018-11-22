#version 330 core

uniform mat4 model;
uniform mat4 view;
uniform mat4 proj;

in vec3 xyz;
in vec3 normal;
in vec2 uv;

out vec3 adjNormal;
out vec3 lightDir;
out vec2 texcoords;

const vec3 inLightDir = normalize(vec3(1.0, 2.0, 1.0));


void main() {
   gl_Position = proj * view * model * vec4(xyz, 1.0);

   vec4 norm4 = transpose(inverse(view * model)) * vec4(normal, 0.0);
   adjNormal = normalize(norm4.xyz);

   lightDir = (view * vec4(inLightDir, 0.0)).xyz;

   texcoords = uv;
}
