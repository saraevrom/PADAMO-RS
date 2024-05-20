use abi_stable::std_types::RString;
use abi_stable::StableAbi;
use std::error::Error;
use std::fmt::Display;

#[repr(C)]
#[derive(StableAbi,Debug,Clone)]
pub enum ExecutionError{
    TypeError,
    ConstantMissing,
    NotConnected(RString),
    UnfilledOutputs,
    MissingPort(RString),
    CycleError,
    OtherError(RString)
}

impl Display for ExecutionError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TypeError=>write!(f, "Type error"),
            Self::ConstantMissing=>write!(f, "Constant is missing"),
            Self::NotConnected(x)=>write!(f, "Port {} is not connected",x),
            Self::UnfilledOutputs=>write!(f,"Unfilled outputs"),
            Self::MissingPort(p)=>write!(f,"Missing port {}", p),
            Self::CycleError=>write!(f,"Cycle connection detected"),
            Self::OtherError(s)=>write!(f, "Error: {}", s),
        }
    }
}

impl Error for ExecutionError{}

impl ExecutionError{
    pub fn from_dyn_error(err:Box<dyn Error>)->Self{
        let v = format!("{}",err);
        Self::OtherError(v.into())
    }
}
