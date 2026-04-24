use thiserror::Error;

#[derive(Debug, Error)]
pub enum VtrlError {
    #[error("Message Bus Error: {0}")]
    MessageBus(String),

    #[error("Renderer Error: {0}")]
    Renderer(String),

    #[error("Window Error: {0}")]
    Window(String),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

pub type Result<T> = anyhow::Result<T, VtrlError>;
