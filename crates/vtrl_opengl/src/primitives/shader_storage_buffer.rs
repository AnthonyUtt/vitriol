extern crate gl;

pub struct ShaderStorageBuffer {
    pub id: u32,
}

impl ShaderStorageBuffer {
    pub unsafe fn new<T>(data: Vec<T>, binding: u32) -> ShaderStorageBuffer {
        let mut id: u32 = 0;
        gl::CreateBuffers(1, &mut id);
        gl::NamedBufferStorage(
            id,
            std::mem::size_of_val(&data) as isize,
            data.as_ptr().cast(),
            gl::DYNAMIC_STORAGE_BIT,
        );
        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, binding, id);

        ShaderStorageBuffer { id }
    }

    pub unsafe fn new_from_arr<T>(data: &[T], binding: u32) -> ShaderStorageBuffer {
        let mut id: u32 = 0;
        gl::CreateBuffers(1, &mut id);
        gl::NamedBufferStorage(
            id,
            std::mem::size_of_val(data) as isize,
            data.as_ptr().cast(),
            gl::DYNAMIC_STORAGE_BIT,
        );
        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, binding, id);

        ShaderStorageBuffer { id }
    }

    pub unsafe fn destroy(&self) {
        gl::DeleteBuffers(1, &self.id as *const _);
    }
}
