pub type FramebufferId = u32;

#[derive(Debug, Clone, Copy)]
pub enum RenderTarget {
    Screen,
    Framebuffer(FramebufferId),
}

#[derive(Debug, Clone, Copy)]
pub enum BlendMode {
    Alpha,
    Additive,
    Multiply,
    PremultipliedAlpha,
    Screen,
    Subtract,
    Replace,
}

