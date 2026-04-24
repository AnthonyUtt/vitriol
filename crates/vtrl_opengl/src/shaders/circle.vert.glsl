#version 450 core

layout (location = 0) in vec3 aWorldPos;
layout (location = 1) in vec3 aLocalPos;
layout (location = 2) in vec4 aColor;
layout (location = 3) in float aThickness;
layout (location = 4) in float aFade;

layout (std140, binding = 0) uniform Matrices {
    mat4 projViewMat;
};

struct VertShaderOut
{
  vec3 localPos;
  vec4 color;
  float thickness;
  float fade;
};

layout (location = 0) out VertShaderOut vs_out;

void main()
{
  vs_out.localPos = aLocalPos;
  vs_out.color = aColor;
  vs_out.thickness = aThickness;
  vs_out.fade = aFade;

  gl_Position = projViewMat * vec4(aWorldPos, 1.0f);
}
