use thiserror::Error;

//     #[error("Video error: {0}")]
//     Video(#[from] video_rs::Error),
// }

#[derive(Error,Debug)]
pub enum VideoBackendError{
    #[error("H264 error: {0}")]
    H264Error(#[from] openh264::Error),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}
