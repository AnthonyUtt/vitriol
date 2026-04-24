extern crate gl;

use crate::types::*;

impl From<UniformType> for gl::types::GLenum {
    fn from(val: UniformType) -> gl::types::GLenum {
        match val {
            UniformType::Float => gl::FLOAT,
            UniformType::Vec2 => gl::FLOAT,
            UniformType::Vec3 => gl::FLOAT,
            UniformType::Vec4 => gl::FLOAT,
            UniformType::Int => gl::INT,
            UniformType::Bool => gl::BOOL,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VertexBuffer {
    pub id: u32,
    pub layout: BufferLayout,
}

impl VertexBuffer {
    pub fn new<T>(vertices: Vec<T>) -> Self {
        let mut id: u32 = 0;

        unsafe {
            gl::GenBuffers(1, &mut id);

            gl::BindBuffer(gl::ARRAY_BUFFER, id);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<T>()) as isize,
                vertices.as_ptr().cast(),
                gl::STATIC_DRAW,
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        VertexBuffer {
            id,
            layout: BufferLayout::new(vec![]),
        }
    }

    pub fn new_from_arr<T>(vertices: &[T]) -> Self {
        let mut id: u32 = 0;

        unsafe {
            gl::GenBuffers(1, &mut id);

            gl::BindBuffer(gl::ARRAY_BUFFER, id);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                std::mem::size_of_val(vertices) as isize,
                vertices.as_ptr().cast(),
                gl::STATIC_DRAW,
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        VertexBuffer {
            id,
            layout: BufferLayout::new(vec![]),
        }
    }

    pub fn dynamic_new<T>(count: usize) -> Self {
        let mut id: u32 = 0;

        unsafe {
            gl::GenBuffers(1, &mut id);

            gl::BindBuffer(gl::ARRAY_BUFFER, id);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (count * std::mem::size_of::<T>()) as isize,
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        VertexBuffer {
            id,
            layout: BufferLayout::new(vec![]),
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
        }
    }

    pub fn set_data<T>(&self, data: &[T], count: usize, offset: usize) {
        unsafe {
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                offset as isize,
                (count * std::mem::size_of::<T>()) as isize,
                data.as_ptr().cast(),
            );
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }

    pub fn destroy(&self) {
        self.unbind();

        unsafe {
            gl::DeleteBuffers(1, &self.id as *const u32);
        }
    }
}
