#version 450 core

in vec2 v_uv;
in vec4 v_color;
in float v_texIdx;

out vec4 FragColor;

layout(binding = 0) uniform sampler2DArray uTextures;

void main()
{
  vec4 src = v_color;

  if (v_texIdx >= 0.0) {
    vec4 t = texture(uTextures, vec3(v_uv, v_texIdx));
    src *= t;
  }

  FragColor = src;
}
