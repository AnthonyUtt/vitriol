extern crate freetype as ft;
extern crate gl;

use std::collections::HashMap;

use vtrl_common::prelude::*;

use crate::primitives::*;
use crate::shaders::*;
use crate::types::*;

const MAX_QUADS: usize = 1_000_000;
const MAX_TEXTURES: usize = 32;
const MAX_FONTS: usize = 16;

const DEFAULT_FONT_BYTES: &[u8] = include_bytes!("./assets/monogram-extended.ttf");

const UNIT_QUAD: [Vec4; 6] = [
    Vec4::new(-0.5, 0.5, 0., 1.),  // top left
    Vec4::new(0.5, 0.5, 0., 1.),   // top right
    Vec4::new(-0.5, -0.5, 0., 1.), // bottom left
    Vec4::new(0.5, 0.5, 0., 1.),   // top right
    Vec4::new(-0.5, -0.5, 0., 1.), // bottom left
    Vec4::new(0.5, -0.5, 0., 1.),  // bottom right
];

const DEFAULT_TEX_BYTES: [u8; 4] = [255; 4];

pub struct Renderer {
    quad_shader: ShaderProgram,
    text_shader: ShaderProgram,
    line_shader: ShaderProgram,
    circle_shader: ShaderProgram,
    vao: VertexArray,
    _quad_vbo: VertexBuffer,
    instance_vbo: VertexBuffer,
    _default_texture_id: usize,
    texture_array: TextureArray,
    font_atlas: FontAtlas,
}

impl Renderer {
    pub fn new() -> Self {
        let quad_shader = ShaderProgram::new()
            .with_vert_shader(QUAD_VERTEX_SHADER_SOURCE)
            .with_frag_shader(QUAD_FRAGMENT_SHADER_SOURCE)
            .build();
        let text_shader = ShaderProgram::new()
            .with_vert_shader(TEXT_VERTEX_SHADER_SOURCE)
            .with_frag_shader(TEXT_FRAGMENT_SHADER_SOURCE)
            .build();
        let line_shader = ShaderProgram::new()
            .with_vert_shader(LINE_VERTEX_SHADER_SOURCE)
            .with_frag_shader(LINE_FRAGMENT_SHADER_SOURCE)
            .build();
        let circle_shader = ShaderProgram::new()
            .with_vert_shader(CIRCLE_VERTEX_SHADER_SOURCE)
            .with_frag_shader(CIRCLE_FRAGMENT_SHADER_SOURCE)
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
        let _default_texture_id = texture_array
            .add_texture(&default_texture)
            .expect("Unable to create default texture!");

        let mut font_atlas = FontAtlas::new(1024, 1024, MAX_FONTS as u32);

        let library = ft::Library::init().expect("Unable to initialize FreeType!");
        let face = library
            .new_memory_face(DEFAULT_FONT_BYTES.to_vec(), 0)
            .expect("Unable to load default font!");
        face.set_pixel_sizes(0, DEFAULT_PIXEL_HEIGHT)
            .expect("Unable to size default font!");
        let glyphs = build_glyph_map(&face).expect("Unable to build default glyphs!");
        font_atlas
            .add_font(glyphs)
            .expect("Unable to register default font!");

        Self {
            quad_shader,
            text_shader,
            line_shader,
            circle_shader,
            vao,
            _quad_vbo,
            instance_vbo,
            _default_texture_id,
            texture_array,
            font_atlas,
        }
    }

    pub fn register_texture(&mut self, texture: &TextureData) -> Result<usize> {
        self.texture_array.add_texture(texture)
    }

    pub fn register_font(&mut self, glyphs: HashMap<char, Glyph>) -> Result<usize> {
        self.font_atlas.add_font(glyphs)
    }

    pub fn get_glyph(&self, font_id: u32, c: char) -> Option<&Glyph> {
        self.font_atlas.get_glyph(font_id as usize, c)
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
        self.draw_instances(
            &self.quad_shader,
            matrix,
            &self.texture_array,
            &self.font_atlas,
            instances_erased(instances),
        );
    }

    pub fn draw_text_instances(&self, matrix: Mat4, instances: &[GlyphInstance]) {
        self.draw_instances(
            &self.text_shader,
            matrix,
            &self.texture_array,
            &self.font_atlas,
            instances_erased(instances),
        );
    }

    pub fn draw_line_instances(&self, matrix: Mat4, instances: &[LineInstance]) {
        self.draw_instances(
            &self.line_shader,
            matrix,
            &self.texture_array,
            &self.font_atlas,
            instances_erased(instances),
        );
    }

    pub fn draw_circle_instances(&self, matrix: Mat4, instances: &[CircleInstance]) {
        self.draw_instances(
            &self.circle_shader,
            matrix,
            &self.texture_array,
            &self.font_atlas,
            instances_erased(instances),
        );
    }

    fn draw_instances(
        &self,
        shader: &ShaderProgram,
        matrix: Mat4,
        textures: &TextureArray,
        fonts: &FontAtlas,
        instances: &[RenderInstance],
    ) {
        if instances.is_empty() {
            return;
        }

        shader.activate();
        shader.set_uniform_mat4("uOrtho", &matrix);
        textures.bind(0);
        fonts.bind(1);

        self.vao.bind();
        self.instance_vbo.bind();
        self.instance_vbo
            .set_data::<RenderInstance>(instances, instances.len(), 0);

        unsafe {
            gl::DrawArraysInstanced(gl::TRIANGLES, 0, 6, instances.len() as i32);
        }

        self.instance_vbo.unbind();
        self.vao.unbind();
        shader.deactivate();
    }
}
