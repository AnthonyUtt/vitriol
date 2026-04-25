use vtrl_common::prelude::*;

#[derive(Debug, Copy, Clone)]
pub enum UniformType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Int,
    Bool,
}
impl UniformType {
    pub fn num_components(&self) -> i32 {
        match self {
            Self::Float => 1,
            Self::Vec2 => 2,
            Self::Vec3 => 3,
            Self::Vec4 => 4,
            Self::Int => 1,
            Self::Bool => 1,
        }
    }

    pub fn size(&self) -> i32 {
        match self {
            Self::Float => std::mem::size_of::<f32>() as i32,
            Self::Vec2 => std::mem::size_of::<Vec2>() as i32,
            Self::Vec3 => std::mem::size_of::<Vec3>() as i32,
            Self::Vec4 => std::mem::size_of::<Vec4>() as i32,
            Self::Int => std::mem::size_of::<u32>() as i32,
            Self::Bool => std::mem::size_of::<bool>() as i32,
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
    pub divisor: u32,
}
impl BufferElement {
    pub fn new(
        layout: u32,
        name: &str,
        element_type: UniformType,
        normalized: bool,
        divisor: u32,
    ) -> BufferElement {
        BufferElement {
            layout,
            name: String::from(name),
            element_type,
            size: element_type.size(),
            offset: 0,
            normalized,
            divisor,
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QuadInstance {
    pub pos: Vec2,
    pub size: Vec2,
    pub rot: f32,
    pub z: f32,
    pub color: Vec4,
    pub uv: Vec4,
    pub tex: f32,
}
