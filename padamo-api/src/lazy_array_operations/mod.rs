use abi_stable::std_types::{RBox, Tuple3, ROption, RVec};
use std::fmt::Debug;

pub mod ndim_array;
pub mod operators;
pub mod merge;
pub mod cache;
pub mod cutter;
use abi_stable::sabi_trait::prelude::TD_Opaque;

pub use ndim_array::ArrayND;
use super::trigger_operations::SparseTagArray;

#[allow(non_local_definitions)]
pub mod traits{
    #[abi_stable::sabi_trait]
    pub trait LazyArrayOperation<T>: Clone+Debug+Sync+Send
    {
        fn length(&self)->usize;
        fn request_range(&self,start:usize, end:usize)->T;
        fn calculate_overhead(&self,start:usize, end:usize)->usize{
            end-start
        }

    }
}
pub use traits::{LazyArrayOperation,LazyArrayOperation_TO};

pub fn make_lao_box<T,U:LazyArrayOperation<T>+'static>(data:U)->LazyArrayOperationBox<T>{
    LazyArrayOperationBox::from_value(data, TD_Opaque)
}

// impl<T> dyn LazyArrayOperation<T>{
//
// }

pub type LazyArrayOperationBox<T> = LazyArrayOperation_TO<'static,RBox<()>,T>;
//pub type LazyArrayOperationArc<T> = LazyArrayOperation_TO<'static,RArc<()>,T>;
pub type LazyDetectorSignal = LazyArrayOperationBox<ndim_array::ArrayND<f64>>;
pub type LazyTrigger = LazyArrayOperationBox<SparseTagArray>;
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

impl LazyTimeSignal{
    pub fn find_unixtime(&self,unixtime:f64)->usize{
        let op = self;
        //let unixtime:f64 = (dt.naive_utc().timestamp_millis() as f64)
        let mut start:usize = 0;
        let op_length = op.length();
        let mut end:usize = op_length;
        let mut middle:usize = (start+end)/2;
        if unixtime>op.request_range(end-1,end)[0]{
            return end-1;
        }
        if unixtime<op.request_range(0,1)[0]{
            return 0;
        }
        while start != middle{
            let item = op.request_range(middle,middle+1)[0];
            if item<=unixtime{
                start = middle;
            }
            if item>=unixtime{
                end = middle;
            }
            middle = (start+end)/2;
        }
        //println!("Datetime search result. req: {}, actual: {}",unixtime, op.request_item(middle));
        let mut res = middle;
        if middle>0{
            let twoval = op.request_range(middle-1,middle+1);
            if (twoval[0]-unixtime).abs()<(twoval[1]-unixtime).abs(){
                res = middle-1;
            }
        }
        if middle<op_length-1{
            let twoval = op.request_range(middle,middle+2);
            if (twoval[0]-unixtime).abs()>(twoval[1]-unixtime).abs(){
                res = middle+1;
            }
        }
        res
    }

    #[cfg(feature = "chrono")]
    pub fn find_time(&self,dt:chrono::DateTime<chrono::Utc>)->usize{
        let unixtime:f64 = (dt.naive_utc().and_utc().timestamp_micros() as f64)*1e-6;
        self.find_unixtime(unixtime)
    }

}


impl<T:Clone+Debug+Sync+Send> LazyArrayOperation<RVec<T>> for RVec<T>{
    fn length(&self)->usize{
        self.len()
    }

    fn request_range(&self,start:usize, end:usize)->RVec<T>{
        self[start..end].to_owned().into()
    }
}
