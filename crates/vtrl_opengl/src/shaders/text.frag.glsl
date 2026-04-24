#version 450 core

out vec4 FragColor;

struct VertShaderOut
{
  vec2 textureCoordinates;
  vec4 color;
};

layout (location = 0) in VertShaderOut fs_in;

layout (binding = 0) uniform sampler2D fontAtlas;

void main()
{
  FragColor = vec4(1.0f, 1.0f, 1.0f, texture(fontAtlas, fs_in.textureCoordinates).r) * fs_in.color;
  // FragColor = texture(fontAtlas, fs_in.textureCoordinates) * fs_in.color;
  // FragColor = fs_in.color;
}
