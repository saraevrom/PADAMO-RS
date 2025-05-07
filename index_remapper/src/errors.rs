use std::fmt::{Debug, Display};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReindexError{
    #[error("Pixels {0:?} and {1:?} are trying to write to pixel {2:?}")]
    PixelRaceError(Vec<usize>,Vec<usize>,Vec<usize>),
    #[error("Target shape is empty")]
    TargetIsEmpty,
    #[error("Shape {0:?} is not compatible with shape {1:?}")]
    IncompatibleShapes(Vec<usize>,Vec<usize>),
    #[error("Index with length {0:?} is not compatible with shape length {1:?}")]
    IncompatibleIndexWithShape(usize,usize),
    #[error("Parsing error: {0}")]
    ParsingError(String)

}

impl<T:Display> From<nom::error::Error<T>> for ReindexError{
    fn from(value: nom::error::Error<T>) -> Self {
        ReindexError::ParsingError(format!("{}",value))
    }
}

impl<T:Debug> From<nom::Err<nom::error::Error<T>>> for ReindexError{
    fn from(value: nom::Err<nom::error::Error<T>>) -> Self {
        ReindexError::ParsingError(format!("{:?}",value))
    }
}
