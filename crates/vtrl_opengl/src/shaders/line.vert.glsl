#version 450 core

layout (location = 0) in vec2 aPos;
layout (location = 1) in vec2 iStart;
layout (location = 2) in vec2 iEnd;
layout (location = 3) in float iThickness;
layout (location = 4) in float iFade;
layout (location = 5) in vec4 iColor;
layout (location = 6) in vec4 iUV;
layout (location = 7) in float iTexIdx;

uniform mat4 uOrtho;

struct VertShaderOut
{
  vec2 position;
  float halfLength;
  float thickness;
  float fade;
  vec4 color;
};

layout (location = 0) out VertShaderOut vs_out;

void main()
{
  vec2 segVec = iEnd - iStart;
  float L = length(segVec);
  vec2 dir  = (L > 1e-6) ? segVec / L : vec2(1.0, 0.0);
  vec2 perp = vec2(-dir.y, dir.x);

  const float PAD = 1.5;
  float alongScale = L + 2.0 * PAD;
  float crossScale = iThickness + 2.0 * PAD;

  vec2 localPx = vec2(aPos.x * alongScale, aPos.y * crossScale);
  vec2 center  = 0.5 * (iStart + iEnd);
  vec2 worldPx = center + dir * localPx.x + perp * localPx.y;

  gl_Position = uOrtho * vec4(worldPx, 0.0, 1.0);

  vs_out.position   = localPx;
  vs_out.halfLength = L * 0.5;
  vs_out.thickness  = iThickness;
  vs_out.fade       = iFade;
  vs_out.color      = iColor;
}
