#version 450 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aTexCoords;
layout (location = 2) in float aTexId;
layout (location = 3) in vec4 aColor;

layout(std140, binding = 0) uniform Matrices {
    mat4 projViewMat;
};

struct VertShaderOut
{
  vec2 textureCoordinates;
  float textureId;
  vec4 color;
};

layout (location = 0) out VertShaderOut vs_out;

void main()
{
    gl_Position = projViewMat * vec4(aPos, 1.0f);
    vs_out.textureCoordinates = aTexCoords;
    vs_out.textureId = aTexId;
    vs_out.color = aColor;
}
