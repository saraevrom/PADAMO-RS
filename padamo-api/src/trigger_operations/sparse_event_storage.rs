use abi_stable::{std_types::{RString,RVec}, StableAbi};


#[repr(C)]
#[derive(StableAbi,Clone)]
pub struct SparseTagArray{
    tags:RVec<RString>
}
