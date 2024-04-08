use abi_stable::std_types::RResult;
use padamo_api::prelude::*;
use padamo_api::{ports,constants};
use abi_stable::std_types::{RVec,RString};
use abi_stable::rvec;
use abi_stable::sabi_trait::prelude::TD_Opaque;
use padamo_api::lazy_array_operations::{LazyArrayOperationBox, LazyTriSignal, LazyTrigger, LazyDetectorSignal, LazyArrayOperation};
use padamo_api::lazy_array_operations::ndim_array;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

#[derive(Clone,Debug)]
pub struct LazyPixelThresholdTrigger{
    src: LazyDetectorSignal,
    threshold:f64,
}


impl LazyPixelThresholdTrigger{
    pub fn new(src: LazyDetectorSignal, threshold:f64)->Self{
        Self{src,threshold}
    }
}

impl LazyArrayOperation<ndim_array::ArrayND<bool>> for LazyPixelThresholdTrigger{
    fn length(&self) -> usize {
        self.src.length()
    }
    fn calculate_overhead(&self, start: usize, end: usize) -> usize{
        self.src.calculate_overhead(start,end)
    }
    fn request_range(&self, start: usize, end: usize) -> ndim_array::ArrayND<bool> {
        let base: ndim_array::ArrayND<f64> = self.src.request_range(start,end);
        let shape = base.shape;
        let flat_data = base.flat_data.into_vec();
        let flat_data:Vec<bool> = flat_data.par_iter().map(|x| *x>self.threshold).collect();
        ndim_array::ArrayND{flat_data:flat_data.into(),shape}
    }
}

#[derive(Clone,Debug)]
pub struct LazyLCThresholdTrigger{
    src: LazyDetectorSignal,
    threshold:f64,
}


impl LazyLCThresholdTrigger{
    pub fn new(src: LazyDetectorSignal, threshold:f64)->Self{
        Self{src,threshold}
    }
}

impl LazyArrayOperation<ndim_array::ArrayND<bool>> for LazyLCThresholdTrigger{
    fn length(&self) -> usize {
        self.src.length()
    }
    fn calculate_overhead(&self, start: usize, end: usize) -> usize{
        self.src.calculate_overhead(start,end)
    }
    fn request_range(&self, start: usize, end: usize) -> ndim_array::ArrayND<bool> {
        let base: ndim_array::ArrayND<f64> = self.src.request_range(start,end);
        let mut lc:Vec<f64> = Vec::with_capacity(end-start);
        lc.resize(end-start, 0.0);

        for index in base.enumerate(){
            lc[index[0]] += base[&index];
        }

        let flat_data:RVec<bool> = lc.iter().map(|x| *x>self.threshold).collect();
        let shape = vec![end-start];
        //let flat_data:RVec<bool> = base.flat_data.iter().map(|x| *x>self.threshold).collect();
        ndim_array::ArrayND{flat_data,shape:shape.into()}
    }
}
