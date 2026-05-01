#version 450 core

out vec4 FragColor;

struct VertShaderOut
{
  vec2 position;
  float thickness;
  float fade;
  vec4 color;
  vec2 uv;
  float texIdx;
};

layout (location = 0) in VertShaderOut fs_in;

void main()
{
  float radius = length(fs_in.position);
  float distance = 1.0 - radius;

  vec2 gradient = vec2(dFdx(distance), dFdy(distance));

  float rangeFromLine = abs(distance / length(gradient));

  float lineWeight = clamp(fs_in.thickness - rangeFromLine, 0.0, 1.0);

  if (lineWeight == 0.0)
    discard;

  FragColor = fs_in.color;
  FragColor.a *= lineWeight;
}
