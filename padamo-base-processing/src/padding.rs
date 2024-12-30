use std::fmt::Debug;

use abi_stable::{rvec, StableAbi};
use padamo_api::lazy_array_operations::{make_lao_box, ArrayND, LazyArrayOperation, LazyArrayOperationBox};
use padamo_api::lazy_array_operations::merge::Merge;


#[derive(Debug,Clone)]
pub struct RepeatFrame<T>
where
    T: Clone+Debug+StableAbi+Send+Sync
{
    frame:ArrayND<T>,
    length:usize,
}

impl<T> RepeatFrame<T>
where
    T: Clone+Debug+StableAbi+Send+Sync
{
    pub fn new(frame: ArrayND<T>, length: usize) -> Self {
        Self { frame, length }
    }
}



impl<T> LazyArrayOperation<ArrayND<T>> for RepeatFrame<T>
where
    T: Clone+Debug+StableAbi+Send+Sync
{
    fn length(&self,) -> usize where {
        self.length
    }

    fn calculate_overhead(&self,_start:usize,_end:usize,) -> usize where {
        0
    }

    fn request_range(&self,start:usize,end:usize,) -> ArrayND<T>where {
        if end>start{
            let req_len = end-start;
            let mut frames = self.frame.clone();
            for _ in 1..req_len{
                frames = frames.merge(self.frame.clone());
            }
            frames
        }
        else{
            let mut shape = self.frame.shape.clone();
            shape[0] = 0;
            ArrayND{flat_data:rvec![],shape}
        }
    }
}


pub fn make_padding<T>(source:LazyArrayOperationBox<ArrayND<T>>, left_padding:usize, right_padding:usize)->LazyArrayOperationBox<ArrayND<T>>
where
    T: Clone+Debug+StableAbi+Send+Sync+'static
{
    let mut res = source.clone();
    if left_padding>0{
        let first_frame = source.request_range(0,1);
        let pad1 = make_lao_box(RepeatFrame::new(first_frame, left_padding));
        res = pad1.merge(res);
    }
    if right_padding>0{
        let srclen = source.length();
        let last_frame = source.request_range(srclen-1,srclen);

        let pad2 = make_lao_box(RepeatFrame::new(last_frame, right_padding));
        res = res.merge(pad2);
    }
    res
}
