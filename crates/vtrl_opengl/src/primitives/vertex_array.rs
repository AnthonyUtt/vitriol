extern crate gl;
use crate::UniformType;
use super::{IndexBuffer, VertexBuffer};
use vtrl_common::prelude::*;
use std::ffi::c_void;

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum DrawMethod {
    Points = gl::POINTS,
    LineStrip = gl::LINE_STRIP,
    LineLoop = gl::LINE_LOOP,
    Lines = gl::LINES,
    LineStripAdjacency = gl::LINE_STRIP_ADJACENCY,
    LinesAdjacency = gl::LINES_ADJACENCY,
    TriangleStrip = gl::TRIANGLE_STRIP,
    TriangleFan = gl::TRIANGLE_FAN,
    Triangles = gl::TRIANGLES,
    TriangleStripAdjacency = gl::TRIANGLE_STRIP_ADJACENCY,
    TrianglesAdjacency = gl::TRIANGLES_ADJACENCY,
}

#[derive(Debug)]
pub struct VertexArray {
    pub id: u32,
    pub vertex_count: u32,
    index_buffer: IndexBuffer,
    vertex_buffers: Vec<VertexBuffer>,
}

impl VertexArray {
    pub unsafe fn new() -> Self {
        let mut id: u32 = 0;
        gl::CreateVertexArrays(1, &mut id);
        VertexArray {
            id,
            vertex_count: 0,
            index_buffer: IndexBuffer { id: 0, count: 0 },
            vertex_buffers: vec![],
        }
    }

    pub unsafe fn link_attributes(
        &self,
        vbo: &VertexBuffer,
        layout: u32,
        num_components: i32,
        attrib_type: u32,
        stride: i32,
        offset: *const c_void,
    ) {
        self.bind();
        vbo.bind();
        gl::VertexAttribPointer(
            layout,
            num_components,
            attrib_type,
            gl::FALSE,
            stride,
            offset,
        );
        gl::EnableVertexAttribArray(layout);

        vbo.unbind();
        self.unbind();
    }

    pub unsafe fn bind(&self) {
        gl::BindVertexArray(self.id);
    }

    pub unsafe fn unbind(&self) {
        gl::BindVertexArray(0);
    }

    pub unsafe fn destroy(&self) {
        self.index_buffer.destroy();

        for vbo in self.vertex_buffers.iter() {
            vbo.destroy();
        }

        self.unbind();
        gl::DeleteVertexArrays(1, &self.id as *const u32);
    }

    pub fn set_index_buffer(&mut self, buf: IndexBuffer) {
        self.index_buffer = buf;
    }

    pub fn get_index_buffer(&self) -> &IndexBuffer {
        &self.index_buffer
    }

    pub unsafe fn add_vertex_buffer(&mut self, buf: VertexBuffer) {
        let layout = &buf.layout;
        if layout.get_elements().is_empty() {
            log::error!("Tried to add a vertex buffer with no layout! {:?}", buf);
            return;
        }

        self.bind();
        self.index_buffer.bind();
        buf.bind();

        for element in layout.get_elements().iter() {
            use UniformType::*;

            gl::EnableVertexAttribArray(element.layout);
            match element.element_type {
                Float | Vec2 | Vec3 | Vec4 => {
                    gl::VertexAttribPointer(
                        element.layout,
                        element.element_type.num_components(),
                        element.element_type.into(),
                        if element.normalized {
                            gl::TRUE
                        } else {
                            gl::FALSE
                        },
                        layout.stride,
                        element.offset as *const c_void,
                    );
                }
                Int | Bool => {
                    gl::VertexAttribIPointer(
                        element.layout,
                        element.element_type.num_components(),
                        element.element_type.into(),
                        layout.stride,
                        element.offset as *const c_void,
                    );
                }
            }
        }

        self.unbind();
        buf.unbind();
        self.index_buffer.unbind();

        self.vertex_buffers.push(buf);
    }
}
