#version 430 core
layout(binding = 0) uniform sampler2D text;

in vec2 texCoord;



out vec4 FragColor;

void main()
{
    FragColor = texture(text, texCoord);
}
