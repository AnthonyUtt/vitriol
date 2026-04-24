extern crate gl;

pub struct UniformBuffer {
    pub id: u32,
}

impl UniformBuffer {
    pub unsafe fn new<T>(data: Vec<T>, binding: u32) -> UniformBuffer {
        let mut id: u32 = 0;
        gl::GenBuffers(1, &mut id);

        gl::BindBuffer(gl::UNIFORM_BUFFER, id);
        gl::BufferData(
            gl::UNIFORM_BUFFER,
            std::mem::size_of_val(&data) as isize,
            data.as_ptr().cast(),
            gl::STATIC_DRAW,
        );
        gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        gl::BindBufferBase(gl::UNIFORM_BUFFER, binding, id);

        UniformBuffer { id }
    }

    pub unsafe fn new_from_arr<T>(data: &[T], binding: u32) -> UniformBuffer {
        let mut id: u32 = 0;
        gl::GenBuffers(1, &mut id);

        gl::BindBuffer(gl::UNIFORM_BUFFER, id);
        gl::BufferData(
            gl::UNIFORM_BUFFER,
            std::mem::size_of_val(data) as isize,
            data.as_ptr().cast(),
            gl::STATIC_DRAW,
        );
        gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        gl::BindBufferBase(gl::UNIFORM_BUFFER, binding, id);

        UniformBuffer { id }
    }

    pub unsafe fn dynamic_new(size: usize, binding: u32) -> UniformBuffer {
        let mut id: u32 = 0;
        gl::GenBuffers(1, &mut id);

        gl::BindBuffer(gl::UNIFORM_BUFFER, id);
        gl::BufferData(
            gl::UNIFORM_BUFFER,
            size as isize,
            std::ptr::null(),
            gl::DYNAMIC_DRAW,
        );
        gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        gl::BindBufferBase(gl::UNIFORM_BUFFER, binding, id);

        UniformBuffer { id }
    }

    pub unsafe fn bind(&self) {
        gl::BindBuffer(gl::UNIFORM_BUFFER, self.id);
    }

    pub unsafe fn set_data<T>(&self, data: &[T], count: usize, offset: usize) {
        self.bind();
        gl::BufferSubData(
            gl::UNIFORM_BUFFER,
            offset as isize,
            (count * std::mem::size_of::<T>()) as isize,
            data.as_ptr().cast(),
        );
        self.unbind();
    }

    pub unsafe fn unbind(&self) {
        gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
    }

    pub unsafe fn destroy(&self) {
        self.unbind();
        gl::DeleteBuffers(1, &self.id as *const u32);
    }
}
