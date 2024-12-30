use abi_stable::std_types::RVec;
use padamo_api::lazy_array_operations::{ArrayND, LazyArrayOperation, LazyDetectorSignal};
use padamo_api::lazy_array_operations::ndim_array;
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use ndarray::{SliceInfo,SliceInfoElem};
use medians::Medianf64;
use std::sync::{Arc,Mutex};

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


#[derive(Clone,Debug)]
pub struct LazyMedianTrigger{
    src: LazyDetectorSignal,
    threshold:f64,
}


impl LazyMedianTrigger{
    pub fn new(src: LazyDetectorSignal, threshold:f64)->Self{
        Self{src,threshold}
    }
}


impl LazyArrayOperation<ndim_array::ArrayND<bool>> for LazyMedianTrigger{
    fn length(&self,) -> usize where {
        self.src.length()
    }
    fn calculate_overhead(&self,start:usize,end:usize,) -> usize where {
        self.src.calculate_overhead(start,end)
    }
    fn request_range(&self,start:usize,end:usize,) -> ndim_array::ArrayND<bool>where {
        let workon = self.src.request_range(start,end);
        let sublen = end-start;
        let indices=  workon.shape.len();
        let workon = workon.to_ndarray();
        let thresh = self.threshold;

        let res = Arc::new(Mutex::new(ArrayND::new(vec![sublen],false)));
        let passed = res.clone();
        (0..sublen).par_bridge().for_each(move |i|{
            let slices:Vec<SliceInfoElem> = (0..indices).map(
                |j| if j==0{
                    SliceInfoElem::Index(i as isize)
                }
                else{
                    SliceInfoElem::Slice { start: 0, end: None, step: 1 }
                }).collect();

            let slicing = SliceInfo::<&[SliceInfoElem],ndarray::Dim<ndarray::IxDynImpl>,ndarray::Dim<ndarray::IxDynImpl>>::try_from(slices.as_slice()).expect("Slicing error");

            let part = workon.slice(slicing);
            let (vector,off_opt) = part.to_owned().into_raw_vec_and_offset();
            let mut vector = vector;
            if let Some(off) = off_opt{
                vector.drain(..off);
            }
            let vector = vector;

            if let Ok(v) = vector.medf_checked(){
                passed.lock().unwrap().flat_data[i] = v>thresh;
                //res.flat_data[i] = v>thresh;
            }
        });

        let lock = Arc::try_unwrap(res).expect("Lock still has multiple owners");
        lock.into_inner().expect("Mutex cannot be locked")
    }
}
