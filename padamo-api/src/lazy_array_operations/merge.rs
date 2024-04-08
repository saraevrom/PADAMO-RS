use abi_stable::std_types::RVec;
use std::fmt::Debug;

use super::{LazyArrayOperation, LazyArrayOperationBox};


#[derive(Clone,Debug)]
pub struct MergedLazyArrayOperation<T>
where
    T:Merge+Clone+Debug
{
    pub a: LazyArrayOperationBox<T>,
    pub b: LazyArrayOperationBox<T>,
    split_index:usize
}

pub trait Merge{
    fn merge(self,other:Self)->Self;
}

impl<T> Merge for RVec<T>{
    fn merge(self,other:Self)->Self {
        let mut result = self;
        result.extend(other);
        result
    }
}

impl<T> MergedLazyArrayOperation<T>
where
    T:Merge+Clone+Debug
{
    pub fn new(a: LazyArrayOperationBox<T>, b: LazyArrayOperationBox<T>) -> Self {
        let split_index = a.length();
        Self { a, b ,split_index}

    }
}


impl<T> LazyArrayOperation<T> for MergedLazyArrayOperation<T>
where
    T:Merge+Clone+Debug
{
    fn length(&self) -> usize where {
        self.a.length()+self.b.length()
    }

    #[allow(clippy::let_and_return)]
    fn request_range(&self,start:usize,end:usize,) -> T where {
        if end<=self.split_index{
            self.a.request_range(start,end)
        }
        else if start>= self.split_index{
            self.b.request_range(start-self.split_index,end-self.split_index)
        }
        else{
            let a = self.a.request_range(start,self.split_index);
            let b = self.b.request_range(0,end-self.split_index);
            a.merge(b)
        }
    }

    fn calculate_overhead(&self,start:usize,end:usize,) -> usize where {
        if end<=self.split_index{
            self.a.calculate_overhead(start,end)
        }
        else if start>= self.split_index{
            self.b.calculate_overhead(start-self.split_index,end-self.split_index)
        }
        else{
            let a = self.a.calculate_overhead(start,self.split_index);
            let b = self.b.calculate_overhead(0,end-self.split_index);
            a+b
        }
    }
}
