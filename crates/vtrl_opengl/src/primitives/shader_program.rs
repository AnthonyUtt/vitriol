extern crate gl;

use gl::types::GLenum;
use vtrl_common::prelude::*;

#[derive(Clone, Debug)]
pub struct Shader {
    pub id: u32,
    pub code: String,
}

#[derive(Debug)]
pub struct ShaderProgram {
    pub id: u32,
    pub vert_shader: Shader,
    pub frag_shader: Shader,
}

impl ShaderProgram {
    pub unsafe fn new() -> Self {
        log::trace!(".new()");
        ShaderProgram {
            id: 0,
            vert_shader: Shader {
                id: 0,
                code: String::from(""),
            },
            frag_shader: Shader {
                id: 0,
                code: String::from(""),
            },
        }
    }

    pub unsafe fn with_vert_shader(mut self, code: String) -> Self {
        log::trace!(".with_vert_shader()");
        let id = self.build_gl_shader(&code, gl::VERTEX_SHADER);
        self.vert_shader = Shader { id, code };
        self
    }

    pub unsafe fn with_vert_shader_path(/*mut*/ self, _path: String) -> Self {
        todo!()
    }

    pub unsafe fn with_frag_shader(mut self, code: String) -> Self {
        log::trace!(".with_frag_shader()");
        let id = self.build_gl_shader(&code, gl::FRAGMENT_SHADER);
        self.frag_shader = Shader { id, code };
        self
    }

    pub unsafe fn with_frag_shader_path(/*mut*/ self, _path: String) -> Self {
        todo!()
    }

    pub unsafe fn build(mut self) -> Self {
        log::trace!(".build()");
        if self.vert_shader.id == 0 || self.frag_shader.id == 0 {
            log::warn!(
                "Building shader program with unregistered shaders!\n{:?}",
                self
            );
        }

        let program = gl::CreateProgram();
        gl::AttachShader(program, self.vert_shader.id);
        gl::AttachShader(program, self.frag_shader.id);
        gl::LinkProgram(program);
        gl::ValidateProgram(program);

        let mut success = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            // non-zero == success
            let mut log: Vec<u8> = Vec::with_capacity(1024);
            let mut len: i32 = 0;

            gl::GetProgramInfoLog(
                program,
                log.capacity() as i32,
                &mut len,
                log.as_mut_ptr().cast(),
            );
            log.set_len(len as usize);
            let log = std::str::from_utf8(&log).unwrap_or("Unknown error!");
            log::error!("Error while linking shader program: {}, {:?}", log, self);

            // Clean up since something went wrong
            gl::DetachShader(program, self.vert_shader.id);
            gl::DeleteShader(self.vert_shader.id);
            gl::DetachShader(program, self.frag_shader.id);
            gl::DeleteShader(self.frag_shader.id);
            gl::DeleteProgram(program);

            return self;
        }

        let mut success = 0;
        gl::GetProgramiv(program, gl::VALIDATE_STATUS, &mut success);
        if success == 0 {
            // non-zero == success
            let mut log: Vec<u8> = Vec::with_capacity(1024);
            let mut len = 0;

            gl::GetProgramInfoLog(
                program,
                log.capacity() as i32,
                &mut len,
                log.as_mut_ptr().cast(),
            );
            log.set_len(len as usize);
            let log = std::str::from_utf8(&log).unwrap_or("Unknown error!");
            log::error!("Error while validating shader program: {}, {:?}", log, self);

            // Clean up since something went wrong
            gl::DetachShader(program, self.vert_shader.id);
            gl::DeleteShader(self.vert_shader.id);
            gl::DetachShader(program, self.frag_shader.id);
            gl::DeleteShader(self.frag_shader.id);
            gl::DeleteProgram(program);

            return self;
        }

        self.id = program;

        // We can marke the shaders for deletion, but they won't be destroyed until
        // they are removed from the proggram
        gl::DeleteShader(self.vert_shader.id);
        gl::DeleteShader(self.frag_shader.id);

        self
    }

    pub unsafe fn activate(&self) {
        if self.id > 0 {
            gl::UseProgram(self.id);
        } else {
            log::warn!(
                "Tried to activate an unregistered shader program!\n{:?}",
                self
            );
        }
    }

    pub unsafe fn deactivate(&self) {
        gl::UseProgram(0);
    }

    pub unsafe fn destroy(&self) {
        // Detach shaders so they are cleaned up properly
        gl::DetachShader(self.id, self.vert_shader.id);
        gl::DetachShader(self.id, self.frag_shader.id);
        gl::DeleteProgram(self.id);
    }

    pub unsafe fn set_uniform_mat4(&self, name: &str, value: &Mat4) {
        let mat_loc = gl::GetUniformLocation(self.id, c_str!(name).as_ptr().cast());
        gl::UniformMatrix4fv(mat_loc, 1, gl::FALSE, value.as_ptr());
    }

    pub unsafe fn set_uniform_vec4(&self, name: &str, value: &Vec4) {
        let loc = gl::GetUniformLocation(self.id, c_str!(name).as_ptr().cast());
        gl::Uniform4f(loc, value.x, value.y, value.z, value.w);
    }

    pub unsafe fn set_uniform_int(&self, name: &str, value: i32) {
        let loc = gl::GetUniformLocation(self.id, c_str!(name).as_ptr().cast());
        gl::Uniform1i(loc, value);
    }

    pub unsafe fn set_uniform_int_arr(&self, name: &str, values: &[u32], count: usize) {
        let loc = gl::GetUniformLocation(self.id, c_str!(name).as_ptr().cast());
        gl::Uniform1iv(loc, count as i32, values.as_ptr().cast());
    }

    unsafe fn build_gl_shader(&self, code: &String, shader_type: GLenum) -> u32 {
        log::trace!("Building GL Shader...");
        let gl_shader = gl::CreateShader(shader_type);
        log::trace!("> Shader ID: {}", gl_shader);
        log::trace!("> Adding shader source");
        gl::ShaderSource(gl_shader, 1, &code.as_ptr().cast(), &(code.len() as i32));
        log::trace!("> Compiling shader");
        gl::CompileShader(gl_shader);

        let mut success = 0;
        gl::GetShaderiv(gl_shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            // non-zero == success
            let mut log: Vec<u8> = Vec::with_capacity(1024);
            let mut len = 0;

            gl::GetShaderInfoLog(
                gl_shader,
                log.capacity() as i32,
                &mut len,
                log.as_mut_ptr().cast(),
            );
            log.set_len(len as usize);
            let log = std::str::from_utf8(&log).unwrap_or("Unknown error!");
            log::error!(
                "Error while compiling shader: {}, type: {:?}, src: {}",
                log,
                shader_type,
                code
            );

            // Clean up since something went wrong
            gl::DeleteShader(gl_shader);

            return 0;
        }

        log::trace!("> Success!");
        // If all went well, we return the inner shader id
        gl_shader
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum ShaderType {
    Vertex = gl::VERTEX_SHADER,
    Fragment = gl::FRAGMENT_SHADER,
}
impl From<ShaderType> for u32 {
    fn from(st: ShaderType) -> u32 {
        st as u32
    }
}
