#version 450 core

out vec4 FragColor;

struct VertShaderOut
{
  vec2 textureCoordinates;
  float textureId;
  vec4 color;
};

layout (location = 0) in VertShaderOut fs_in;

uniform sampler2DArray textureArray;

void main()
{
    FragColor = texture(textureArray, vec3(fs_in.textureCoordinates, fs_in.textureId)) * fs_in.color;
}
