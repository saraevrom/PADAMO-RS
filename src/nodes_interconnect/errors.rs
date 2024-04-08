use std::{fmt::Display, error::Error};


#[derive(Debug)]
pub enum NodeRegistryError{
    NodeDuplicate(String),
    LibraryError(abi_stable::library::LibraryError),
    NoName,
}

impl Display for NodeRegistryError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NodeDuplicate(x)=>write!(f,"Node {} is defined twice", x),
            Self::LibraryError(x)=>write!(f,"Library error: {}", x),
            Self::NoName=>write!(f,"No file name"),
        }
    }
}

impl Error for NodeRegistryError{}
