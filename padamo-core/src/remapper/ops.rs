use std::fmt::Debug;

use padamo_api::lazy_array_operations::{ArrayND, LazyArrayOperation, LazyArrayOperationBox};


#[derive(Clone, Debug)]
pub struct LazyRemapper<T:Clone+Debug+abi_stable::StableAbi>{
    source:LazyArrayOperationBox<ArrayND<T>>,
    remapper:index_remapper::BakedTransformer<T>
}

impl<T: Clone + Debug + abi_stable::StableAbi> LazyRemapper<T> {
    pub fn new(source: LazyArrayOperationBox<ArrayND<T>>, remapper: index_remapper::BakedTransformer<T>) -> Self {
        Self { source, remapper }
    }
}


impl<T:Clone+Debug+abi_stable::StableAbi+Send+Sync> LazyArrayOperation<ArrayND<T>> for LazyRemapper<T>{
    fn length(&self,) -> usize where {
        self.source.length()
    }

    fn request_range(&self,start:usize,end:usize,) -> ArrayND<T>{
        let mut unmapped:ArrayND<T> = self.source.request_range(start, end);

        let mut target_shape = vec![end-start];
        target_shape.extend(self.remapper.target_shape.clone());
        let target_shape = target_shape;
        let mut res = Vec::new();
        while let Some(frame) = unmapped.take_frame(){
            res.extend(self.remapper.apply(&frame).unwrap().flat_data);
        }
        ArrayND { flat_data: res.into(), shape: target_shape.into() }
    }
}
