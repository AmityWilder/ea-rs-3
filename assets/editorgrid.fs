#version 330

// Input vertex attributes (from vertex shader)
in vec2 fragTexCoord;
in vec4 fragColor;

// Input uniform values
uniform sampler2D texture0;
uniform vec4 colDiffuse;

const float GRID_SIZE = 8;
uniform vec2 offset = vec2(0.0);
uniform float zoom_exp = 0.0;
uniform vec2 size = vec2(1280.0, 720.0);

// Output fragment color
out vec4 finalColor;

// NOTE: Add your custom variables here

void main()
{
    // Texel color fetching from texture sampler
    vec4 texelColor = texture(texture0, fragTexCoord);

    float zoom = 1.0/pow(2.0, zoom_exp);
    float grid_width = GRID_SIZE * zoom;
    vec2 rel = mod((fragTexCoord*size - offset) * zoom, vec2(GRID_SIZE));

    // NOTE: Implement here your fragment shader code

    // final color is the color from the texture
    //    times the tint color (colDiffuse)
    //    times the fragment color (interpolated vertex color)
    finalColor = (rel.x < 0.5 || rel.y < 0.5) ? texelColor : vec4(0.0);
}
