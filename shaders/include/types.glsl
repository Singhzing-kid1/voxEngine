//casts shadows
struct GlobalLight {
    vec3 position;
    vec3 color;
    float intensity;
};

// doesnt cast shadows, but can override
struct LocalLight {
    vec4 posRadius; //vec4(position, radius)
	vec4 colorIntensity; // vec4(color, intensity)
};
