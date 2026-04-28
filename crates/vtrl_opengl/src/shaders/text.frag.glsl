#version 450 core

in vec2 v_uv;
in vec4 v_color;
in float v_texIdx;

out vec4 FragColor;

uniform sampler2DArray uFontAtlas;

void main()
{
    // Sample the font atlas - we use the red channel as alpha/coverage
    // since bitmap fonts are typically single-channel
    float coverage = texture(uFontAtlas, vec3(v_uv, v_texIdx)).r;
    
    // Apply the instance color with coverage as alpha
    vec4 color = v_color;
    color.a *= coverage;
    
    // Premultiply alpha for correct blending
    color.rgb *= color.a;
    
    FragColor = color;
}
