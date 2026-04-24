#version 450 core

layout (location = 0) in vec2 aPos;     // unit quad vertices
layout (location = 1) in vec2 iPosPx;   // top-left position in pixels
layout (location = 2) in vec2 iSizePx;  // glyph size in pixels
layout (location = 3) in vec4 iUV;      // u0, v0, u1, v1
layout (location = 4) in vec4 iColor;   // RGBA tint
layout (location = 5) in float iZIndex; // depth

uniform mat4 uOrtho; // top-left origin orthographic projection

out vec2 v_uv;
out vec4 v_color;

void main()
{
    // Scale the unit quad by the glyph size
    vec2 p = aPos * iSizePx;
    
    // Translate to final position in pixel coordinates
    // aPos is centered at origin, so we offset by half size
    // to make iPosPx the top-left corner
    vec2 posPx = iPosPx + p + (iSizePx * 0.5);
    
    // Transform pixel coordinates to NDC using orthographic matrix
    gl_Position = uOrtho * vec4(posPx, iZIndex, 1.0);
    
    // Map unit quad [-0.5..0.5] to [0..1] then lerp between UV bounds
    vec2 uv01 = aPos + vec2(0.5);
    v_uv = mix(iUV.xy, iUV.zw, uv01);
    
    // Pass color to fragment shader
    v_color = iColor;
}
