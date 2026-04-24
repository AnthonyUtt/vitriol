extern crate gl;

use crate::UniformType;

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
pub struct BufferElement {
    pub layout: u32,
    pub name: String,
    pub element_type: UniformType,
    pub size: i32,
    pub offset: i32,
    pub normalized: bool,
}
impl BufferElement {
    pub fn new(
        layout: u32,
        name: &str,
        element_type: UniformType,
        normalized: bool,
    ) -> BufferElement {
        BufferElement {
            layout,
            name: String::from(name),
            element_type,
            size: element_type.size(),
            offset: 0,
            normalized,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BufferLayout {
    elements: Vec<BufferElement>,
    pub stride: i32,
}
impl BufferLayout {
    pub fn new(elements: Vec<BufferElement>) -> BufferLayout {
        let mut layout = BufferLayout {
            elements,
            stride: 0,
        };
        layout.calculate_stride();
        layout
    }

    pub fn get_elements(&self) -> &Vec<BufferElement> {
        &self.elements
    }

    fn calculate_stride(&mut self) {
        let mut offset = 0i32;
        let mut stride = 0i32;
        for element in self.elements.iter_mut() {
            element.offset = offset;
            offset += element.size;
            stride += element.size;
        }
        self.stride = stride;
    }
}

#[derive(Debug, Clone)]
pub struct VertexBuffer {
    pub id: u32,
    pub layout: BufferLayout,
}

impl VertexBuffer {
    pub unsafe fn new<T>(vertices: Vec<T>) -> Self {
        let mut id: u32 = 0;
        gl::GenBuffers(1, &mut id);

        gl::BindBuffer(gl::ARRAY_BUFFER, id);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<T>()) as isize,
            vertices.as_ptr().cast(),
            gl::STATIC_DRAW,
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);

        VertexBuffer {
            id,
            layout: BufferLayout::new(vec![]),
        }
    }

    pub unsafe fn new_from_arr<T>(vertices: &[T]) -> Self {
        let mut id: u32 = 0;
        gl::GenBuffers(1, &mut id);

        gl::BindBuffer(gl::ARRAY_BUFFER, id);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            std::mem::size_of_val(vertices) as isize,
            vertices.as_ptr().cast(),
            gl::STATIC_DRAW,
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);

        VertexBuffer {
            id,
            layout: BufferLayout::new(vec![]),
        }
    }

    pub unsafe fn dynamic_new<T>(count: usize) -> Self {
        let mut id: u32 = 0;
        gl::GenBuffers(1, &mut id);

        gl::BindBuffer(gl::ARRAY_BUFFER, id);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (count * std::mem::size_of::<T>()) as isize,
            std::ptr::null(),
            gl::DYNAMIC_DRAW,
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);

        VertexBuffer {
            id,
            layout: BufferLayout::new(vec![]),
        }
    }

    pub unsafe fn bind(&self) {
        gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
    }

    pub unsafe fn set_data<T>(&self, data: &[T], count: usize, offset: usize) {
        self.bind();
        gl::BufferSubData(
            gl::ARRAY_BUFFER,
            offset as isize,
            (count * std::mem::size_of::<T>()) as isize,
            data.as_ptr().cast(),
        );
        self.unbind();
    }

    pub unsafe fn unbind(&self) {
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    pub unsafe fn destroy(&self) {
        self.unbind();
        gl::DeleteBuffers(1, &self.id as *const u32);
    }
}
