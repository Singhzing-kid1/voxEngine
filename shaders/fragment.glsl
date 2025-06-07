#version 420 core

in vec3 normal;
in vec3 fragPos;
in vec3 color;

out vec4 FragColor;

uniform vec3 lightPosition;
uniform vec3 viewPos;

void main()
{   
    float ambientStrength = 0.1;
    vec3 ambient = ambientStrength * vec3(1.0);

    float specularStrength = 0.5;

    vec3 viewDir = normalize(viewPos - fragPos);

    vec3 norm = normalize(normal);
    vec3 lightDirection = normalize(lightPosition - fragPos);

    vec3 reflectDir = reflect(-lightDirection, norm);

    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32);

    float diff = max(dot(norm, lightDirection), 0.0);
    
    vec3 diffuse = diff * vec3(1.0);
    vec3 specular = specularStrength * spec * vec3(1.0);

    vec3 result = (ambient + diffuse + specular) * color;

    FragColor = vec4(result, 1.0);
}