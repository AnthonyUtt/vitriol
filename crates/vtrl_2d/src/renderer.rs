use ultraviolet::{Mat4, Vec2, Vec3, Vec4};

use vtrl_common::prelude::*;
use vtrl_opengl::prelude::*;

const MAX_QUADS: usize = 10000;
const MAX_VERTICES: usize = MAX_QUADS * 4;
const MAX_INDICES: usize = MAX_QUADS * 6;
const MAX_TEXTURES: usize = 32;

const QUAD_VERTICES: [Vec4; 4] = [
    Vec4::new(-0.5, 0.5, 0., 1.),  // top left
    Vec4::new(0.5, 0.5, 0., 1.),   // top right
    Vec4::new(-0.5, -0.5, 0., 1.), // bottom left
    Vec4::new(0.5, -0.5, 0., 1.),  // bottom right
];

const QUAD_TEX_COORDS: [Vec2; 4] = [
    Vec2::new(0., 0.), // top left
    Vec2::new(1., 0.), // top right
    Vec2::new(0., 1.), // bottom left
    Vec2::new(1., 1.), // bottom right
];

const DEFAULT_TEX_BYTES: [u8; 4] = [255; 4];

#[repr(C)]
#[derive(Debug, Clone)]
struct QuadVertex {
    pub position: Vec3,
    pub texture_coordinates: Vec2,
    pub texture_id: f32,
    pub color: Vec4,
}
impl Default for QuadVertex {
    fn default() -> QuadVertex {
        QuadVertex {
            position: Vec3::zero(),
            texture_coordinates: Vec2::zero(),
            texture_id: 0.,
            color: Vec4::one(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
struct CircleVertex {
    pub world_position: Vec3,
    pub local_position: Vec3,
    pub color: Vec4,
    pub thickness: f32,
    pub fade: f32,
}
impl Default for CircleVertex {
    fn default() -> CircleVertex {
        CircleVertex {
            world_position: Vec3::zero(),
            local_position: Vec3::zero(),
            color: Vec4::one(),
            thickness: 1.,
            fade: 0.005,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
struct LineVertex {
    pub position: Vec3,
    pub color: Vec4,
}
impl Default for LineVertex {
    fn default() -> LineVertex {
        LineVertex {
            position: Vec3::zero(),
            color: Vec4::one(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
struct TextVertex {
    pub position: Vec3,
    pub texture_coordinates: Vec2,
    pub color: Vec4,
}
impl Default for TextVertex {
    fn default() -> TextVertex {
        TextVertex {
            position: Vec3::zero(),
            texture_coordinates: Vec2::zero(),
            color: Vec4::zero(),
        }
    }
}

pub struct Renderer2D {
    quad_count: usize,
    quad_vao: VertexArray,
    quad_vbo: VertexBuffer,
    quad_shader: ShaderProgram,
    quad_vertices: Vec<QuadVertex>,
    quad_index_count: usize,

    circle_vao: VertexArray,
    circle_vbo: VertexBuffer,
    circle_shader: ShaderProgram,
    circle_vertices: Vec<CircleVertex>,
    circle_index_count: usize,

    line_vao: VertexArray,
    line_vbo: VertexBuffer,
    line_shader: ShaderProgram,
    line_vertices: Vec<LineVertex>,
    line_vertex_count: usize,

    text_vao: VertexArray,
    text_vbo: VertexBuffer,
    text_shader: ShaderProgram,
    text_vertices: Vec<TextVertex>,
    text_index_count: usize,
    // text_font_atlas: FontAtlas,

    // default_texture_id: f32,
    // texture_array: TextureArray,

    ubo: UniformBuffer,
}

impl Renderer2D {
    fn init() -> Result<Self> {
        unsafe {
            commands::init();

            let quad_shader = ShaderProgram::new()
                .with_vert_shader(String::from(QUAD_VERTEX_SHADER_SOURCE))
                .with_frag_shader(String::from(QUAD_FRAGMENT_SHADER_SOURCE))
                .build();

            let mut quad_vao = VertexArray::new();

            let quad_indices = {
                let mut offset = 0;
                let mut i = 0;
                let mut data = Vec::<u32>::with_capacity(MAX_INDICES);
                while i < MAX_INDICES {
                    #[allow(clippy::identity_op)]
                    data.push(offset + 0);
                    data.push(offset + 1);
                    data.push(offset + 2);
                    data.push(offset + 1);
                    data.push(offset + 2);
                    data.push(offset + 3);
                    i += 6;
                    offset += 4;
                }
                data
            };
            let quad_ebo = IndexBuffer::new(quad_indices);
            quad_vao.set_index_buffer(quad_ebo);

            let mut quad_vbo = VertexBuffer::dynamic_new::<QuadVertex>(MAX_VERTICES);
            quad_vbo.layout = BufferLayout::new(vec![
                BufferElement::new(0, "aPos", UniformType::Vec3, false),
                BufferElement::new(1, "aTexCoords", UniformType::Vec2, false),
                BufferElement::new(2, "aTexId", UniformType::Float, false),
                BufferElement::new(3, "aColor", UniformType::Vec4, false),
            ]);
            quad_vao.add_vertex_buffer(quad_vbo.clone());

            let circle_shader = ShaderProgram::new()
                .with_vert_shader(String::from(CIRCLE_VERTEX_SHADER_SOURCE))
                .with_frag_shader(String::from(CIRCLE_FRAGMENT_SHADER_SOURCE))
                .build();

            let mut circle_vao = VertexArray::new();
            circle_vao.set_index_buffer(quad_ebo);

            let mut circle_vbo = VertexBuffer::dynamic_new::<CircleVertex>(MAX_VERTICES);
            circle_vbo.layout = BufferLayout::new(vec![
                BufferElement::new(0, "aWorldPos", UniformType::Vec3, false),
                BufferElement::new(1, "aLocalPos", UniformType::Vec3, false),
                BufferElement::new(2, "aColor", UniformType::Vec4, false),
                BufferElement::new(3, "aThickness", UniformType::Float, false),
                BufferElement::new(4, "aFade", UniformType::Float, false),
            ]);
            circle_vao.add_vertex_buffer(circle_vbo.clone());

            let line_shader = ShaderProgram::new()
                .with_vert_shader(String::from(LINE_VERTEX_SHADER_SOURCE))
                .with_frag_shader(String::from(LINE_FRAGMENT_SHADER_SOURCE))
                .build();

            let mut line_vao = VertexArray::new();
            let mut line_vbo = VertexBuffer::dynamic_new::<LineVertex>(MAX_VERTICES);
            line_vbo.layout = BufferLayout::new(vec![
                BufferElement::new(0, "aPos", UniformType::Vec3, false),
                BufferElement::new(1, "aColor", UniformType::Vec4, false),
            ]);
            line_vao.add_vertex_buffer(line_vbo.clone());

            let text_shader = ShaderProgram::new()
                .with_vert_shader(String::from(TEXT_VERTEX_SHADER_SOURCE))
                .with_frag_shader(String::from(TEXT_FRAGMENT_SHADER_SOURCE))
                .build();

            let mut text_vao = VertexArray::new();
            text_vao.set_index_buffer(quad_ebo);

            let mut text_vbo = VertexBuffer::dynamic_new::<TextVertex>(MAX_VERTICES);
            text_vbo.layout = BufferLayout::new(vec![
                BufferElement::new(0, "aPos", UniformType::Vec3, false),
                BufferElement::new(1, "aTexCoords", UniformType::Vec2, false),
                BufferElement::new(2, "aColor", UniformType::Vec4, false),
            ]);
            text_vao.add_vertex_buffer(text_vbo.clone());
            // let text_font_atlas =
            //     FontAtlas::new(String::from("/home/anthony/Downloads/Roboto-Regular.ttf"))?;

            // let mut texture_array = TextureArray::new(1024, 1024, MAX_TEXTURES as u32, None);
            // let default_texture = TextureData {
            //     bytes: DEFAULT_TEX_BYTES.to_vec(),
            //     width: 1,
            //     height: 1,
            // };
            // let default_texture_id = texture_array.add_texture(default_texture)? as f32;

            let ubo = UniformBuffer::dynamic_new(std::mem::size_of::<Mat4>(), 0);

            Ok(Self {
                quad_count: 0,

                quad_vao,
                quad_vbo,
                quad_shader,
                quad_index_count: 0,
                quad_vertices: vec![QuadVertex::default(); MAX_VERTICES],

                circle_vao,
                circle_vbo,
                circle_shader,
                circle_index_count: 0,
                circle_vertices: vec![CircleVertex::default(); MAX_VERTICES],

                line_vao,
                line_vbo,
                line_shader,
                line_vertex_count: 0,
                line_vertices: vec![LineVertex::default(); MAX_VERTICES],

                text_vao,
                text_vbo,
                text_shader,
                text_index_count: 0,
                text_vertices: vec![TextVertex::default(); MAX_VERTICES],
                // text_font_atlas,
                //
                // default_texture_id,
                // texture_array,

                ubo,
            })
        }
    }

    fn cleanup(&mut self) {
        unsafe {
            self.quad_vao.destroy();
            self.quad_shader.destroy();
            self.circle_vao.destroy();
            self.circle_shader.destroy();
            self.line_vao.destroy();
            self.line_shader.destroy();
            self.text_vao.destroy();
            self.text_shader.destroy();
            self.ubo.destroy();
            // self.texture_array.destroy();
        }
    }

    fn begin_scene(&mut self, camera: &impl Camera) {
        let pvm = camera.get_projection_view_matrix();
        unsafe {
            self.ubo.set_data::<Mat4>(&[*pvm], 1, 0);
        }

        self.start_batch();
    }

    fn end_scene(&mut self) {
        self.flush();
    }

    // fn register_texture(&mut self, texture: &TextureData) -> usize {
    //     match self.texture_array.add_texture(texture.to_owned()) {
    //         Ok(id) => id,
    //         Err(_) => {
    //             log::error!("Failed to add texture to array!");
    //             0
    //         }
    //     }
    // }

    fn draw_quad(
        &mut self,
        position: Vec3,
        size: Vec2,
        rotation: f32,
        color: Vec4,
        texture_id: Option<usize>,
    ) {
        let transform: Mat4 = Mat4::identity().translated(&position);
        let transform = transform * Mat4::from_rotation_z(rotation);
        let transform = transform * Mat4::from_nonuniform_scale(Vec3::new(size.x, size.y, 1.));

        let texture_id = texture_id.unwrap_or(0);
        // let uv_scalar = self.texture_array.get_uv_scalar(texture_id);
        let uv_scalar = Vec2::one();

        // Set the vertex data in our local array
        let vertex_offset = self.quad_index_count / 6 * 4;
        #[allow(clippy::needless_range_loop)]
        for i in 0..4 {
            self.quad_vertices[vertex_offset + i] = QuadVertex {
                position: (transform * QUAD_VERTICES[i]).xyz(),
                texture_coordinates: (uv_scalar * QUAD_TEX_COORDS[i]),
                texture_id: texture_id as f32,
                color,
            };
        }

        // Increment the index count
        self.quad_index_count += 6;
        self.quad_count += 1;

        if self.quad_count >= MAX_QUADS {
            self.next_batch();
        }
    }

    fn draw_circle(
        &mut self,
        position: Vec3,
        size: Vec2,
        color: Vec4,
        thickness: Option<f32>,
        fade: Option<f32>,
    ) {
        let transform: Mat4 = Mat4::identity().translated(&position);
        let transform = transform * Mat4::from_nonuniform_scale(Vec3::new(size.x, size.y, 1.));

        let vertex_offset = self.circle_index_count / 6 * 4;
        #[allow(clippy::needless_range_loop)]
        for i in 0..4 {
            self.circle_vertices[vertex_offset + i] = CircleVertex {
                world_position: (transform * QUAD_VERTICES[i]).xyz(),
                local_position: QUAD_VERTICES[i].xyz() * 2.,
                color,
                thickness: thickness.unwrap_or(1.),
                fade: fade.unwrap_or(0.005),
            }
        }

        self.circle_index_count += 6;
        self.quad_count += 1;

        if self.quad_count >= MAX_QUADS {
            self.next_batch();
        }
    }

    fn draw_line(&mut self, position_0: Vec3, position_1: Vec3, color: Vec4) {
        self.line_vertices[self.line_vertex_count] = LineVertex {
            position: position_0,
            color,
        };
        self.line_vertices[self.line_vertex_count + 1] = LineVertex {
            position: position_1,
            color,
        };

        self.line_vertex_count += 2;
    }

    // fn draw_string(
    //     &mut self,
    //     text: String, /*, font: &Font*/
    //     _position: Vec3,
    //     _scale: Vec2,
    //     color: Vec4,
    // ) {
    //     let scale: Vec2 = {
    //         let window_size = unsafe { commands::get_viewport_size() };
    //         let x: f32 = 2. / window_size.x as f32;
    //         let y: f32 = 2. / window_size.y as f32;
    //         Vec2::new(x, y)
    //     };
    //
    //     let _ = self.render_text(text, 0., 0., scale, color);
    // }

    fn start_batch(&mut self) {
        self.quad_count = 0;
        self.quad_index_count = 0;
        self.circle_index_count = 0;
        self.line_vertex_count = 0;
        self.text_index_count = 0;
    }

    fn next_batch(&mut self) {
        self.flush();
        self.start_batch();
    }

    // fn render_text(
    //     &mut self,
    //     text: String,
    //     x: f32,
    //     y: f32,
    //     scale: Vec2,
    //     color: Vec4,
    // ) -> Result<()> {
    //     let mut x_advance = x;
    //     let mut y_advance = y;
    //     for c in text.chars() {
    //         let code_point = self.text_font_atlas.font_data.code_points.get(&c).ok_or(
    //             VtrlError::Renderer(format!("Character {c} not found in font atlas!"))
    //         )?;
    //         let metrics = &code_point.metrics;
    //
    //         let px = x_advance + metrics.left as f32 * scale.x;
    //         let py = -y_advance - metrics.top as f32 * scale.y;
    //         let w = metrics.width as f32 * scale.x;
    //         let h = metrics.height as f32 * scale.y;
    //
    //         x_advance += metrics.advance_x as f32 * scale.x;
    //         y_advance += metrics.advance_y as f32 * scale.y;
    //
    //         if w <= 0. || h <= 0. {
    //             continue;
    //         }
    //
    //         let vertex_offset = self.text_index_count / 6 * 4;
    //         self.text_vertices[vertex_offset + 0] = TextVertex {
    //             position: Vec3::new(px, -py, 1.),
    //             texture_coordinates: Vec2::new(metrics.tex_coords.x, 0.),
    //             color,
    //         };
    //         self.text_vertices[vertex_offset + 1] = TextVertex {
    //             position: Vec3::new(px + w, -py, 1.),
    //             texture_coordinates: Vec2::new(
    //                 metrics.tex_coords.x
    //                     + metrics.width as f32 / self.text_font_atlas.bitmap.width as f32,
    //                 0.,
    //             ),
    //             color,
    //         };
    //         self.text_vertices[vertex_offset + 2] = TextVertex {
    //             position: Vec3::new(px, -py - h, 1.),
    //             texture_coordinates: Vec2::new(
    //                 metrics.tex_coords.x,
    //                 metrics.height as f32 / self.text_font_atlas.bitmap.height as f32,
    //             ),
    //             color,
    //         };
    //         self.text_vertices[vertex_offset + 3] = TextVertex {
    //             position: Vec3::new(px + w, -py - h, 1.),
    //             texture_coordinates: Vec2::new(
    //                 metrics.tex_coords.x
    //                     + metrics.width as f32 / self.text_font_atlas.bitmap.width as f32,
    //                 metrics.height as f32 / self.text_font_atlas.bitmap.height as f32,
    //             ),
    //             color,
    //         };
    //
    //         self.text_index_count += 6;
    //         self.quad_count += 1;
    //
    //         if self.quad_count >= MAX_QUADS {
    //             self.next_batch();
    //         }
    //     }
    //     Ok(())
    // }

    fn flush(&mut self) {
        unsafe {
            commands::set_clear_color(0.3, 0.3, 0.3, 1.);
            commands::clear();

            if self.quad_index_count > 0 {
                self.quad_vbo.set_data::<QuadVertex>(
                    &self.quad_vertices,
                    self.quad_vertices.len(),
                    0,
                );

                self.quad_shader.activate();
                // self.texture_array.bind(0);
                self.quad_shader.set_uniform_int("textureArray", 0);

                commands::draw_indexed(&self.quad_vao, Some(self.quad_index_count as i32));

                self.quad_shader.deactivate();
            }

            if self.circle_index_count > 0 {
                self.circle_vbo.set_data::<CircleVertex>(
                    &self.circle_vertices,
                    self.circle_vertices.len(),
                    0,
                );

                self.circle_shader.activate();
                commands::draw_indexed(&self.circle_vao, Some(self.circle_index_count as i32));
                self.circle_shader.deactivate();
            }

            if self.line_vertex_count > 0 {
                self.line_vbo.set_data::<LineVertex>(
                    &self.line_vertices,
                    self.line_vertices.len(),
                    0,
                );

                self.line_shader.activate();
                commands::draw_lines(&self.line_vao, self.line_vertex_count as i32);
                self.line_shader.deactivate();
            }

            // if self.text_index_count > 0 {
            //     self.text_vbo.set_data::<TextVertex>(
            //         &self.text_vertices,
            //         self.text_vertices.len(),
            //         0,
            //     );
            //
            //     self.text_shader.activate();
            //     self.text_font_atlas.bind(0);
            //     self.text_shader.set_uniform_int("fontAtlas", 0);
            //
            //     commands::draw_indexed(&self.text_vao, Some(self.text_index_count as i32));
            //
            //     self.text_shader.deactivate();
            // }
        }
    }
}
