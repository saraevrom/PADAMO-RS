use thiserror::Error;

#[derive(Error,Debug)]
pub enum VideoBackendError{
    #[error("{0}")]
    ErrorFFMPEG(#[from] playa_ffmpeg::Error),
    #[error("{0}")]
    ErrorIO(#[from] std::io::Error)
}
