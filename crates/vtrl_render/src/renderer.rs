use vtrl_common::prelude::*;
use vtrl_opengl::prelude::*;

use crate::{
    font_atlas::*,
    texture_atlas::*,
};


const MAX_QUADS: usize = 1_000_000;

const UNIT_QUAD: [Vec4; 6] = [
    Vec4::new(-0.5, 0.5, 0., 1.),  // top left
    Vec4::new(0.5, 0.5, 0., 1.),   // top right
    Vec4::new(-0.5, -0.5, 0., 1.), // bottom left
    Vec4::new(0.5, 0.5, 0., 1.),   // top right
    Vec4::new(-0.5, -0.5, 0., 1.), // bottom left
    Vec4::new(0.5, -0.5, 0., 1.),  // bottom right
];

pub struct Renderer {
    quad_shader: ShaderProgram,
    text_shader: ShaderProgram,
    line_shader: ShaderProgram,
    circle_shader: ShaderProgram,
    vao: VertexArray,
    _quad_vbo: VertexBuffer,
    instance_vbo: VertexBuffer,
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

        Self {
            quad_shader,
            text_shader,
            line_shader,
            circle_shader,
            vao,
            _quad_vbo,
            instance_vbo,
        }
    }

    pub fn draw_quad_instances(
        &self,
        matrix: Mat4,
        texture_atlas: &TextureAtlas,
        font_atlas: &FontAtlas,
        instances: &[QuadInstance],
    ) {
        self.draw_instances(
            &self.quad_shader,
            matrix,
            texture_atlas,
            font_atlas,
            instances_erased(instances),
        );
    }

    pub fn draw_text_instances(
        &self,
        matrix: Mat4,
        texture_atlas: &TextureAtlas,
        font_atlas: &FontAtlas,
        instances: &[GlyphInstance],
    ) {
        self.draw_instances(
            &self.text_shader,
            matrix,
            texture_atlas,
            font_atlas,
            instances_erased(instances),
        );
    }

    pub fn draw_line_instances(
        &self,
        matrix: Mat4,
        texture_atlas: &TextureAtlas,
        font_atlas: &FontAtlas,
        instances: &[LineInstance],
    ) {
        self.draw_instances(
            &self.line_shader,
            matrix,
            texture_atlas,
            font_atlas,
            instances_erased(instances),
        );
    }

    pub fn draw_circle_instances(
        &self,
        matrix: Mat4,
        texture_atlas: &TextureAtlas,
        font_atlas: &FontAtlas,
        instances: &[CircleInstance],
    ) {
        self.draw_instances(
            &self.circle_shader,
            matrix,
            texture_atlas,
            font_atlas,
            instances_erased(instances),
        );
    }

    fn draw_instances(
        &self,
        shader: &ShaderProgram,
        matrix: Mat4,
        textures: &TextureAtlas,
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

        commands::draw_instanced(instances.len() as u32);

        self.instance_vbo.unbind();
        self.vao.unbind();
        shader.deactivate();
    }
}
