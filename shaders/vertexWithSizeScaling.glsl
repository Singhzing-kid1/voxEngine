#version 330 core
layout (location = 0) in vec3 aPos;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform vec3 position;
uniform vec3 chunkSpaceTransform;
uniform vec3 chunk;
uniform float size;

void main(){
    gl_Position = projection * view * model * vec4(aPos + position * 2 * size + chunk * 34 + chunkSpaceTransform, 1.0);
}