#version 450 core

out vec4 FragColor;

struct VertShaderOut
{
  vec3 localPos;
  vec4 color;
  float thickness;
  float fade;
};

layout (location = 0) in VertShaderOut fs_in;

void main()
{
  float distance = 1.0 - length(fs_in.localPos);
  float circle = smoothstep(0.0, fs_in.fade, distance);
  circle *= smoothstep(fs_in.thickness + fs_in.fade, fs_in.thickness, distance);

  if (circle == 0.0)
    discard;

  FragColor = fs_in.color;
  FragColor.a *= circle;
}
