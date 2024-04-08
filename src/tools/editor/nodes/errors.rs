use std::fmt::Display;
use std::error::Error;

#[derive(Debug,Clone)]
pub enum NodeError{
    NoInput(String),
    NoOutput(String),
    SameNodeLink,
    NodeIndexError(usize),
    IncompatiblePorts(super::PortType,super::PortType),
    IncompatibleConstants,
    MissingConstant(String)
}

impl Display for NodeError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeError::NoInput(x) =>  write!(f,"No such input {}",x),
            NodeError::NoOutput(x) => write!(f,"No such output {}", x),
            NodeError::SameNodeLink=>write!(f, "Cannot link to same node"),
            NodeError::NodeIndexError(x)=>write!(f,"No such node with index {}", x),
            NodeError::IncompatiblePorts(x, y)=>write!(f,"Incompatible port types: {:?} and {:?}", x, y),
            NodeError::IncompatibleConstants=> write!(f,"Incompatible constants"),
            NodeError::MissingConstant(x) => write!(f,"No constant named {}", x),
        }
    }
}

impl Error for NodeError{}
