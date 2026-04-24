#version 450 core

in vec2 v_uv;
in vec4 v_color;
flat in int v_texIdx;

out vec4 FragColor;

// uniform sampler2D uTex[8];

void main()
{
  vec4 src = v_color;

  // if (v_texIdx >= 0) {
  //   // t is STRAIGHT alpha by default
  //   vec4 t = texture(uTex[v_texIdx], v_uv);
  //   // convert straight -> premult before applying instance color
  //   t.rgb *= t.a;
  //   src *= t;
  // }

  FragColor = src;
}
