use std::fmt::Debug;

use abi_stable::StableAbi;

use super::ArrayND;
use crate::lazy_array_operations::{cache::Cache, LazyArrayOperation};


impl<T:StableAbi+Clone+Debug+Send+Sync> LazyArrayOperation<ArrayND<T>> for ArrayND<T>{
    fn length(&self,) -> usize where {
        self.shape[0]
    }

    fn request_range(&self,start:usize,end:usize,) -> ArrayND<T>{
        let mut res = self.clone();
        res = res.cut_front(start);
        res = res.cut_end(self.length()-end);
        res
    }
}
