extern crate gl;

#[derive(Debug, Clone, Copy)]
pub struct IndexBuffer {
    pub id: u32,
    pub count: i32,
}

impl IndexBuffer {
    pub unsafe fn new(indices: Vec<u32>) -> Self {
        let mut id: u32 = 0;
        gl::GenBuffers(1, &mut id);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, id);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * 4) as isize,
            indices.as_ptr().cast(),
            gl::STATIC_DRAW,
        );
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

        IndexBuffer { id, count: 0 }
    }

    pub unsafe fn new_from_arr(indices: &[u32]) -> Self {
        let mut id: u32 = 0;
        gl::GenBuffers(1, &mut id);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, id);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * 4) as isize,
            indices.as_ptr().cast(),
            gl::STATIC_DRAW,
        );
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

        IndexBuffer { id, count: 0 }
    }

    pub unsafe fn dynamic_new(count: usize) -> Self {
        let mut id: u32 = 0;
        gl::GenBuffers(1, &mut id);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, id);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (count * 4) as isize,
            std::ptr::null(),
            gl::DYNAMIC_DRAW,
        );
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

        IndexBuffer { id, count: 0 }
    }

    pub unsafe fn bind(&self) {
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id);
    }

    pub unsafe fn set_data(&self, data: &[u32], count: usize, offset: usize) {
        self.bind();
        gl::BufferSubData(
            gl::ELEMENT_ARRAY_BUFFER,
            offset as isize,
            (count * 4) as isize,
            data.as_ptr().cast(),
        );
        self.unbind();
    }

    pub unsafe fn unbind(&self) {
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
    }

    pub unsafe fn destroy(&self) {
        self.unbind();
        gl::DeleteBuffers(1, &self.id as *const u32);
    }
}
