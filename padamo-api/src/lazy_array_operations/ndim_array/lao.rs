use std::fmt::Debug;

use abi_stable::{rvec, StableAbi};

use super::ArrayND;
use crate::lazy_array_operations::LazyArrayOperation;


impl<T:StableAbi+Clone+Debug+Send+Sync> LazyArrayOperation<ArrayND<T>> for ArrayND<T>{
    fn length(&self,) -> usize where {
        self.shape[0]
    }

    fn request_range(&self,start:usize,end:usize,) -> ArrayND<T>{
        if (self.shape.iter().product::<usize>()) ==0{
            return ArrayND{shape:rvec![], flat_data:rvec![]};
        }
        let frame_size = self.frame_size();
        let mut new_shape = self.shape.clone();
        new_shape[0] = end-start;
        let inner_start = start*frame_size;
        let inner_end = end*frame_size;
        let new_flat_data = self.flat_data[inner_start..inner_end].to_owned().into();
        ArrayND { flat_data: new_flat_data, shape: new_shape }

        // let mut res = self.clone();
        // res = res.cut_front(start);
        // res = res.cut_end(self.length()-end);
        // res
    }
}
