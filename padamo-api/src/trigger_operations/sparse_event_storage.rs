use abi_stable::{std_types::{RString,RVec}, StableAbi};
use std::cmp::Ord;

#[repr(C)]
#[derive(StableAbi,Clone)]
struct SparseTag{
    pub tag:RString,
    pub position:usize,
    pub duration:usize,
}

impl SparseTag {
    fn new(tag: RString, position: usize, duration:usize) -> Self {
        Self { tag, position, duration }
    }
}

impl Ord for SparseTag{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Ord<usize>::cmp(self.position, other.position)
    }
}


#[repr(C)]
#[derive(StableAbi,Clone)]
pub struct SparseTagArray{
    tags:RVec<SparseTag>
}

impl SparseTagArray{
    pub fn new()->Self{
        Self { tags: RVec::new() }
    }

    pub fn push<T:Into<RString>>(&mut self, tag:T, position:usize, duration:usize){
        let new_item = SparseTag::new(tag, position, duration);
        self.tags.push(new_item);
        for i in (1..self.tags.len()).rev(){
            if self.tags[i-1]>self.tags[i]{
                self.tags.swap(i-1, i);
            }
        }
    }
}
