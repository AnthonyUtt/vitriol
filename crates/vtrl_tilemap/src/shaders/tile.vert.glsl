#version 450 core

layout (location = 0) in vec2 aPos;
layout (location = 1) in vec2 iGridPos;
layout (location = 2) in int iTileId;

uniform mat4 uViewProjection;
uniform vec2 uTilemapOffset;
uniform float uTileSize;
uniform uint uColumns;
uniform uint uRows;
uniform float uTexId;

out vec2 vUV;
flat out float vTexLayer;

void main()
{
    // Position: shift unit quad (-0.5..0.5) to (0..1), add grid pos, scale by tile size, offset to world
    vec2 worldPos = uTilemapOffset + (iGridPos + aPos + vec2(0.5)) * uTileSize;
    gl_Position = uViewProjection * vec4(worldPos, 0.0, 1.0);

    // UV: convert tile ID to column/row in the tileset, then map quad corners to that tile's UV rect
    int col = iTileId % int(uColumns);
    int row = iTileId / int(uColumns);
    vec2 tileUV = vec2(float(col), float(row)) / vec2(float(uColumns), float(uRows));
    vec2 uvSize = 1.0 / vec2(float(uColumns), float(uRows));
    vec2 localUV = aPos + vec2(0.5);
    vUV = tileUV + localUV * uvSize;

    vTexLayer = uTexId;
}
