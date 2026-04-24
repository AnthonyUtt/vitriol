extern crate gl;

#[derive(Debug, Clone, Copy)]
pub struct IndexBuffer {
    pub id: u32,
    pub count: i32,
}

impl IndexBuffer {
    pub fn new(indices: &[u32]) -> Self {
        let mut id: u32 = 0;

        unsafe {
            gl::GenBuffers(1, &mut id);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, id);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * 4) as isize,
                indices.as_ptr().cast(),
                gl::STATIC_DRAW,
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }

        IndexBuffer { id, count: 0 }
    }

    pub fn dynamic_new(count: usize) -> Self {
        let mut id: u32 = 0;

        unsafe {
            gl::GenBuffers(1, &mut id);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, id);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (count * 4) as isize,
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }

        IndexBuffer { id, count: 0 }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id);
        }
    }

    pub fn set_data(&self, data: &[u32], count: usize, offset: usize) {
        unsafe {
            gl::BufferSubData(
                gl::ELEMENT_ARRAY_BUFFER,
                offset as isize,
                (count * 4) as isize,
                data.as_ptr().cast(),
            );
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }

    pub fn destroy(&self) {
        self.unbind();

        unsafe {
            gl::DeleteBuffers(1, &self.id as *const u32);
        }
    }
}
