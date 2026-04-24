#version 450 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec4 aColor;

layout(std140, binding = 0) uniform Matrices {
    mat4 projViewMat;
};

struct VertShaderOut
{
  vec4 color;
};

layout (location = 0) out VertShaderOut vs_out;

void main()
{
  vs_out.color = aColor;
  gl_Position = projViewMat * vec4(aPos, 1.0f);
}
