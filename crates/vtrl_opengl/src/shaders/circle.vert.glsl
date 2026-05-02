#version 450 core

layout (location = 0) in vec2 aPos; // unit quad
layout (location = 1) in vec2 iPosPx;
layout (location = 2) in vec2 iSize;
layout (location = 3) in float iThickness;
layout (location = 4) in float iFade;
layout (location = 5) in vec4 iColor;
layout (location = 6) in vec4 iUV; // u0, v0, u1, v1
layout (location = 7) in float iTexIdx;

uniform mat4 uOrtho; // top-left origin ortho

struct VertShaderOut
{
  vec2 position;
  float thickness;
  float fade;
  vec4 color;
  vec2 uv;
  float texIdx;
};

layout (location = 0) out VertShaderOut vs_out;

void main()
{
  // Padding so we don't cut off the circle on the edge of the quad
  const float PAD = 1.05;
  // Scale the unit quad by the radius
  vec2 p = aPos * iSize * PAD;

  // Translate to final position in pixel coordinates
  vec2 posPx = iPosPx + p;

  // Transform pixel coordinates to NDC using matrix
  gl_Position = uOrtho * vec4(posPx, 0.0f, 1.0f);

  // map unit quad [-0.5..0.5] to [0..1] then lerp u0..u1
  vec2 uv01 = aPos + vec2(0.5);
  vs_out.uv = mix(iUV.xy, iUV.zw, uv01);

  vs_out.position = aPos * 2.0 * PAD;
  vs_out.thickness = iThickness;
  vs_out.fade = iFade;
  vs_out.color = iColor;
  vs_out.texIdx = iTexIdx;
}
