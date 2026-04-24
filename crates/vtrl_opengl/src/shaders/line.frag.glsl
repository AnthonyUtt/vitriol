#version 450 core

out vec4 FragColor;

struct VertexOutput
{
  vec4 color;
};

layout (location = 0) in VertexOutput fs_in;

void main()
{
  FragColor = fs_in.color;
}
