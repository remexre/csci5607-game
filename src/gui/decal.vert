#version 330 core

in vec3 xyz;
in vec2 uv;

out vec2 texcoords;


void main() {
   gl_Position = vec4(xyz - vec3(0.5, 0.5, 0.0), 1.0);
   texcoords = uv;
}
