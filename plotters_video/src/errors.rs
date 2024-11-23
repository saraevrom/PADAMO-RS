use thiserror::Error;

#[derive(Error,Debug)]
pub enum VideoBackendError{
    #[error("Video error: {0}")]
    Video(#[from] video_rs::Error),
}
