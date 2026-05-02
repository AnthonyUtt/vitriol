#version 450 core

layout (location = 0) in vec2 aPos; // unit quad
layout (location = 1) in vec2 iPosPx;
layout (location = 2) in vec2 iSizePx;
layout (location = 3) in float iRot;
layout (location = 4) in float iZIndex;
layout (location = 5) in vec4 iColor;
layout (location = 6) in vec4 iUV; // u0,v0,u1,v1
layout (location = 7) in float iTexIdx;

uniform mat4 uOrtho; // top-left origin ortho

out vec2 v_uv;
out vec4 v_color;
out float v_texIdx;

void main()
{
    float s = sin(iRot), c = cos(iRot);

    // Scale the unit quad by the instance size
    vec2 p = aPos * iSizePx;

    // Rotate around the pivot (assuming center of quad)
    vec2 pr = vec2(c * p.x - s * p.y, s * p.x + c * p.y);

    // Translate to final position in pixel coordinates
    vec2 posPx = iPosPx + pr;

    // Transform pixel coordinates to NDC using matrix
    gl_Position = uOrtho * vec4(posPx, 0.0f, 1.0f);

    // map unit quad [-0.5..0.5] to [0..1] then lerp u0..u1
    vec2 uv01 = aPos + vec2(0.5);
    v_uv = mix(iUV.xy, iUV.zw, uv01);

    // pass color and texture to frag shader
    v_color = iColor;
    v_texIdx = iTexIdx;
}
