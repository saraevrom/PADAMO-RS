use padamo_api::lazy_array_operations::cache::Cache;
use padamo_api::lazy_array_operations::LazyArrayOperation;
use padamo_api::lazy_array_operations::ArrayND;
use std::fmt::Debug;

#[derive(Clone,Debug)]
pub struct ConstantArray<T:Clone+Debug+Sync+Send+abi_stable::StableAbi>{
    data:ArrayND<T>
}

impl<T: Clone + Debug + Sync + Send + abi_stable::StableAbi> ConstantArray<T> {
    pub fn new(data: ArrayND<T>) -> Self { Self { data } }
}

impl<T:Clone+Debug+Sync+Send+abi_stable::StableAbi> LazyArrayOperation<ArrayND<T>> for ConstantArray<T>{
    #[allow(clippy::let_and_return)]
    fn length(&self,) -> usize where {
        if let Some(v) = self.data.shape.iter().next(){
            *v
        }
        else{
            0
        }
    }

    fn request_range(&self,start:usize,end:usize,) -> ArrayND<T> where {
        let mut data = self.data.clone();
        let length = self.length();
        if end==length && start==0{
            return data;
        }
        data = data.cut_front(start);
        data = data.cut_end(length-end);
        data
    }
}
