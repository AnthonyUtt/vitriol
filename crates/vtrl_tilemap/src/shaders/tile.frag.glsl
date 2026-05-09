#version 450 core

in vec2 vUV;
flat in float vTexLayer;

uniform sampler2DArray uTileAtlas;

out vec4 fragColor;

void main()
{
    fragColor = texture(uTileAtlas, vec3(vUV, vTexLayer));
    if (fragColor.a < 0.01) discard;
}
