use vtrl_common::prelude::*;

use crate::primitives::*;
use crate::shaders::*;
use crate::types::*;

const MAX_QUADS: usize = 1_000_000;
const MAX_TEXTURES: usize = 32;

const UNIT_QUAD: [Vec4; 6] = [
    Vec4::new(-0.5, 0.5, 0., 1.),  // top left
    Vec4::new(0.5, 0.5, 0., 1.),   // top right
    Vec4::new(-0.5, -0.5, 0., 1.), // bottom left
    Vec4::new(0.5, 0.5, 0., 1.),   // top right
    Vec4::new(-0.5, -0.5, 0., 1.), // bottom left
    Vec4::new(0.5, -0.5, 0., 1.),  // bottom right
];

const DEFAULT_TEX_BYTES: [u8; 4] = [255; 4];

pub struct QuadRenderer {
    shader: ShaderProgram,
    vao: VertexArray,
    quad_vbo: VertexBuffer,
    instance_vbo: VertexBuffer,
    default_texture_id: usize,
    texture_array: TextureArray,
}

impl QuadRenderer {
    pub fn new() -> Self {
        let shader = ShaderProgram::new()
            .with_vert_shader(QUAD_VERTEX_SHADER_SOURCE)
            .with_frag_shader(QUAD_FRAGMENT_SHADER_SOURCE)
            .build();

        let mut vao = VertexArray::new();

        let mut quad_vbo = VertexBuffer::new_from_arr::<Vec4>(&UNIT_QUAD);
        quad_vbo.layout = BufferLayout::new(vec![BufferElement::new(
            0,
            "aPos",
            UniformType::Vec4,
            false,
            0,
        )]);
        vao.add_vertex_buffer(quad_vbo.clone());

        let mut instance_vbo = VertexBuffer::dynamic_new::<QuadInstance>(MAX_QUADS);
        instance_vbo.layout = BufferLayout::new(vec![
            BufferElement::new(1, "iPosPx", UniformType::Vec2, false, 1),
            BufferElement::new(2, "iSizePx", UniformType::Vec2, false, 1),
            BufferElement::new(3, "iRot", UniformType::Float, false, 1),
            BufferElement::new(4, "iZIndex", UniformType::Float, false, 1),
            BufferElement::new(5, "iColor", UniformType::Vec4, false, 1),
            BufferElement::new(6, "iUV", UniformType::Vec4, false, 1),
            BufferElement::new(7, "iTexIdx", UniformType::Float, false, 1),
        ]);
        vao.add_vertex_buffer(instance_vbo.clone());

        let mut texture_array = TextureArray::new(1024, 1024, MAX_TEXTURES as u32, None);
        let default_texture = TextureData {
            bytes: DEFAULT_TEX_BYTES.to_vec(),
            width: 1,
            height: 1,
        };
        let default_texture_id = texture_array
            .add_texture(&default_texture)
            .expect("Unable to create default texture!");

        Self {
            shader,
            vao,
            quad_vbo,
            instance_vbo,
            default_texture_id,
            texture_array,
        }
    }

    pub fn register_texture(&mut self, texture: &TextureData) -> Result<usize> {
        self.texture_array.add_texture(texture)
    }

    pub fn compute_uv(&self, texture_id: usize, uv: Vec4) -> Vec4 {
        let scalar = self.texture_array.get_uv_scalar(texture_id);

        Vec4::new(
            uv.x * scalar.x,
            uv.y * scalar.y,
            uv.z * scalar.x,
            uv.w * scalar.y,
        )
    }

    pub fn draw_quad_instances(&self, matrix: Mat4, instances: &[QuadInstance]) {
        if instances.is_empty() {
            return;
        }

        self.shader.activate();
        self.shader.set_uniform_mat4("uOrtho", &matrix);
        self.texture_array.bind(0);

        self.vao.bind();
        self.instance_vbo.bind();
        self.instance_vbo
            .set_data::<QuadInstance>(instances, instances.len(), 0);

        unsafe {
            gl::DrawArraysInstanced(gl::TRIANGLES, 0, 6, instances.len() as i32);
        }

        self.instance_vbo.unbind();
        self.vao.unbind();
        self.shader.deactivate();
    }
}
