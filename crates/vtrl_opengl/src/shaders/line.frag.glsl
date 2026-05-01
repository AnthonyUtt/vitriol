#version 450 core

out vec4 FragColor;

struct VertShaderOut
{
  vec2 position;
  float halfLength;
  float thickness;
  float fade;
  vec4 color;
};

layout (location = 0) in VertShaderOut fs_in;

void main()
{
  vec2 p = fs_in.position;
  p.x -= clamp(p.x, -fs_in.halfLength, fs_in.halfLength);
  float distFromSeg = length(p);

  float signedDist = fs_in.thickness * 0.5 - distFromSeg;

  vec2 gradient = vec2(dFdx(signedDist), dFdy(signedDist));
  float pixelDist = signedDist / max(length(gradient), 1e-6);
  float alpha = clamp(pixelDist + 0.5, 0.0, 1.0);

  if (alpha <= 0.0)
    discard;

  FragColor = fs_in.color;
  FragColor.a *= alpha;
}
