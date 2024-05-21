use abi_stable::std_types::{RBox, RArc, Tuple3, ROption, RVec};
use std::fmt::Debug;

pub mod ndim_array;
pub mod operators;
pub mod merge;
pub mod cache;
pub mod cutter;
use abi_stable::sabi_trait::prelude::TD_Opaque;

pub use ndim_array::ArrayND;

#[abi_stable::sabi_trait]
pub trait LazyArrayOperation<T>: Clone+Debug+Sync+Send
{
    fn length(&self)->usize;
    fn request_range(&self,start:usize, end:usize)->T;
    fn calculate_overhead(&self,start:usize, end:usize)->usize{
        end-start
    }

}

pub fn make_lao_box<T,U:LazyArrayOperation<T>+'static>(data:U)->LazyArrayOperationBox<T>{
    LazyArrayOperationBox::from_value(data, TD_Opaque)
}

// impl<T> dyn LazyArrayOperation<T>{
//
// }

pub type LazyArrayOperationBox<T> = LazyArrayOperation_TO<'static,RBox<()>,T>;
//pub type LazyArrayOperationArc<T> = LazyArrayOperation_TO<'static,RArc<()>,T>;
pub type LazyDetectorSignal = LazyArrayOperationBox<ndim_array::ArrayND<f64>>;
pub type LazyTrigger = LazyArrayOperationBox<ndim_array::ArrayND<bool>>;
pub type LazyTimeSignal = LazyArrayOperationBox<RVec<f64>>;

pub type LazyTriSignal = Tuple3<LazyDetectorSignal,LazyTimeSignal,ROption<LazyTrigger>>;





// #[derive(Debug,Clone)]
// pub struct LazyArrayOperationLocalCache<T>{
//     src:LazyArrayOperationBox<T>
//     cache:T
//
// };
//
// impl<T> LazyArrayOperationLocalCache<T>{
//     pub fn new(src:LazyArrayOperationBox<T>)->Self{
//         Self{src, cache:Vec::new()}
//     }
// }


impl<T> LazyArrayOperationBox<T>
where
    T:Clone+Debug+'static
{
    pub fn cut(self,start:usize,end:usize)->Result<Self,cutter::CutError>{
        let cutdata = cutter::LazyArrayOperationCutter::new(self, start, end)?;
        Ok(make_lao_box(cutdata))
    }


}


impl<T> LazyArrayOperationBox<T>
where
    T:merge::Merge+Clone+Debug+'static
{
    pub fn merge(self,other:Self)->Self{
        let merged = merge::MergedLazyArrayOperation::new(self,other);
        LazyArrayOperationBox::from_value(merged, TD_Opaque)
    }


}

impl<T> LazyArrayOperationBox<T>
where
    T:cache::Cache+Clone+Debug+'static+Send+Sync
{
    pub fn cached(self)->Self{
        let cached = cache::LazyArrayOperationLocalCache::new(self);
        LazyArrayOperationBox::from_value(cached, TD_Opaque)
    }
}
