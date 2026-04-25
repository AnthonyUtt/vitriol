use thiserror::Error;

#[derive(Debug, Error)]
pub enum VtrlError {
    #[error("Message Bus Error: {0}")]
    MessageBus(String),

    #[error("Renderer Error: {0}")]
    Renderer(String),

    #[error("Window Error: {0}")]
    Window(String),

    #[error("Lock Error: {0}")]
    Lock(String),

    #[error(transparent)]
    Image(#[from] image::ImageError),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

pub type Result<T> = anyhow::Result<T, VtrlError>;
