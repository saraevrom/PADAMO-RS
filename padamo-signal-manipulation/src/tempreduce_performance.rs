use std::{collections::VecDeque, fmt::Debug, sync::{Arc, Mutex}, thread};

use padamo_api::lazy_array_operations::{ndim_array::ArrayND, LazyArrayOperation, LazyDetectorSignal};
use rayon::prelude::*;
use super::ops::free_threads;

#[derive(Clone,Debug)]
pub struct LazySpaceConverterPerformant{
    divider:usize,
    source:LazyDetectorSignal,
    is_sum:bool,
    frame_shape:Vec<usize>
}

impl LazySpaceConverterPerformant {
    pub fn new(divider: usize, source: LazyDetectorSignal, is_sum:bool) -> Self {
        let frame_shape:Vec<usize> = if source.length()>0{
            let mut testframe_size = source.request_range(0,1).shape;
            testframe_size.drain(1..).collect()
        }
        else{
            vec![1]
        };

        Self { divider, source ,is_sum, frame_shape}

    }
}


impl LazyArrayOperation<ArrayND<f64>> for LazySpaceConverterPerformant{

    fn length(&self,) -> usize {
        let src_len = self.source.length();
        src_len/self.divider
    }

    fn calculate_overhead(&self,start:usize,end:usize,)->usize{
        self.source.calculate_overhead(start*self.divider, end*self.divider)
    }

    fn request_range(&self,start:usize,end:usize,) -> ArrayND<f64>{
        let start_src = start*self.divider;
        let end_src = end*self.divider;
        let divider = self.divider;
        let raw_data:ArrayND<f64> = self.source.request_range(start_src,end_src);
        let is_sum = self.is_sum;
        let frame_size = raw_data.shape.iter().skip(1).fold(1usize,|a,b| a*b);
        // let stepped_data:Vec<_> = (0..end-start).par_bridge()
        //     .map(|i| raw_data.slice(ndarray::s![i*divider..(i+1)*divider,..,..]))
        //     .map(|x| compress(x, is_sum))
        //     .collect();

        let mut tgt_shape = raw_data.shape.clone();
        tgt_shape[0] = end-start;

        let threadcount = num_cpus::get();
        let mut threads:VecDeque<thread::JoinHandle<()>> = VecDeque::with_capacity(10);

        let target = Arc::new(Mutex::new(ArrayND::<f64>::new(tgt_shape.into(),0.0)));
        let source = Arc::new(raw_data);

        for pixel in 0usize..frame_size{
            free_threads(&mut threads, threadcount);

            let length = end-start;
            let src = source.clone();
            let tgt = target.clone();

            let handle = thread::spawn(move || {
                for i in 0..length{
                    let mut sum:f64 = 0.0;
                    for j in i*divider..i*divider+divider{
                        sum += src.flat_data[j*frame_size+pixel];
                    }
                    if !is_sum{
                        sum/=divider as f64;
                    }
                    tgt.lock().unwrap().flat_data[i*frame_size+pixel] = sum;
                }
            });

            threads.push_back(handle);
        }

        free_threads(&mut threads, 1);

        let lock = Arc::try_unwrap(target).unwrap();
        lock.into_inner().unwrap()
    }
}
