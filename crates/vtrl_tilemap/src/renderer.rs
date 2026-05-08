use vtrl_common::prelude::*;
use vtrl_opengl::prelude::*;

use crate::atlas::*;
use crate::shaders::*;

const MAX_TILES: usize = 1_000_000;

const UNIT_QUAD: [Vec4; 6] = [
    Vec4::new(-0.5, 0.5, 0., 1.),  // top left
    Vec4::new(0.5, 0.5, 0., 1.),   // top right
    Vec4::new(-0.5, -0.5, 0., 1.), // bottom left
    Vec4::new(0.5, 0.5, 0., 1.),   // top right
    Vec4::new(-0.5, -0.5, 0., 1.), // bottom left
    Vec4::new(0.5, -0.5, 0., 1.),  // bottom right
];

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TileInstance {
    pub grid_position: Vec2,
    pub tile_id: u32,
}

pub struct TilemapRenderer {
    shader: ShaderProgram,
    vao: VertexArray,
    _quad_vbo: VertexBuffer,
    instance_vbo: VertexBuffer,
}

impl TilemapRenderer {
    pub fn new() -> Self {
        let shader = ShaderProgram::new()
            .with_vert_shader(TILE_VERT_SHADER_SOURCE)
            .with_frag_shader(TILE_FRAG_SHADER_SOURCE)
            .build();

        let mut vao = VertexArray::new();

        let mut _quad_vbo = VertexBuffer::new_from_arr::<Vec4>(&UNIT_QUAD);
        _quad_vbo.layout = BufferLayout::new(vec![BufferElement::new(
            0,
            "aPos",
            UniformType::Vec4,
            false,
            0,
        )]);
        vao.add_vertex_buffer(_quad_vbo.clone());

        let mut instance_vbo = VertexBuffer::dynamic_new::<TileInstance>(MAX_TILES);
        instance_vbo.layout = BufferLayout::new(vec![
            BufferElement::new(1, "iGridPos", UniformType::Vec2, false, 1),
            BufferElement::new(2, "iTileId", UniformType::Int, false, 1),
        ]);
        vao.add_vertex_buffer(instance_vbo.clone());

        Self {
            shader,
            vao,
            _quad_vbo,
            instance_vbo,
        }
    }

    pub fn draw_tiles(
        &self,
        matrix: Mat4,
        offset: Vec2,
        tile_size: f32,
        column_count: u32,
        row_count: u32,
        tex_id: f32,
        atlas: &TileAtlas,
        tiles: &[TileInstance],
    ) {
        if tiles.is_empty() {
            return;
        }

        self.shader.activate();
        self.shader.set_uniform_mat4("uViewProjection", &matrix);
        self.shader.set_uniform_vec2("uTilemapOffset", &offset);
        self.shader.set_uniform_float("uTileSize", tile_size);
        self.shader.set_uniform_uint("uColumns", column_count);
        self.shader.set_uniform_uint("uRowCount", row_count);
        self.shader.set_uniform_float("uTexId", tex_id);

        atlas.bind(0);

        self.vao.bind();
        self.instance_vbo.bind();
        self.instance_vbo
            .set_data::<TileInstance>(tiles, tiles.len(), 0);

        commands::draw_instanced(tiles.len() as u32);

        self.instance_vbo.unbind();
        self.vao.unbind();
        self.shader.deactivate();
    }
}

impl Default for TilemapRenderer {
    fn default() -> Self { Self::new() }
}
