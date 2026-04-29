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

    #[error("Type Error: {0}")]
    Type(String),

    #[error("Asset Error: {0}")]
    Asset(String),

    #[error(transparent)]
    Image(#[from] image::ImageError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Freetype(#[from] freetype::Error),

    #[error(transparent)]
    Ron(#[from] ron::de::SpannedError),

    #[error(transparent)]
    RonSer(#[from] ron::Error),

    #[error(transparent)]
    Serde(#[from] erased_serde::Error),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

pub type Result<T> = anyhow::Result<T, VtrlError>;
