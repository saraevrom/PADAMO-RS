use std::{fmt::Display, error::Error};


#[derive(Debug)]
pub enum NodeRegistryError{
    NodeDuplicate(String),
    LegacyDuplicate(String),
    LibraryError(abi_stable::library::LibraryError),
    PathError,
    NoName,
}

impl Display for NodeRegistryError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NodeDuplicate(x)=>write!(f,"Node {} is defined twice", x),
            Self::LegacyDuplicate(x)=>write!(f,"Legacy name {} is defined twice", x),
            Self::LibraryError(x)=>write!(f,"Library error: {}", x),
            Self::NoName=>write!(f,"No file name"),
            Self::PathError=>write!(f,"Path error ocurred"),
        }
    }
}

impl Error for NodeRegistryError{}
