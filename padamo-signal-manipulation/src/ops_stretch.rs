use std::{collections::VecDeque, fmt::Debug, sync::{Arc, Mutex}, thread};

use abi_stable::{std_types::RVec, StableAbi};
use padamo_api::{lazy_array_operations::{ArrayND, LazyArrayOperation, LazyArrayOperationBox, LazyDetectorSignal, LazyTimeSignal, LazyTrigger}, prelude::*};
use super::ops::free_threads;



fn get_bounds(src_time:&LazyTimeSignal,tgt_time:&LazyTimeSignal,start:usize, end:usize)->(usize,usize){
    let start_time = tgt_time.request_range(start, start+1)[0];
    let end_time = tgt_time.request_range(end-1, end)[0];
    let mut start_id = src_time.find_unixtime(start_time);
    let mut end_id = src_time.find_unixtime(end_time);
    println!("Remapping range {}-{}",start,end);


    if start_id>0{
        start_id-=1;
    }
    println!("start {}",start_id);

    let src_len = src_time.length();
    if end_id<src_len{
        end_id += 1;
    }

    while (end_id<src_len) && (src_time.request_range(end_id, end_id+1)[0]<end_time){
    //if end_id<src_len{
        end_id += 1;
        println!("end {}({}) TARGET={}",end_id, src_time.request_range(end_id, end_id+1)[0], end_time);

    }
    println!("end {}({}) TARGET={}",end_id, src_time.request_range(end_id, end_id+1)[0], end_time);


    (start_id,end_id)
}

fn estimate_linear_interp1d(xdata:&[f64],ydata:&[f64], xnew:f64, start_index:usize)->(usize,f64){
    for i in start_index..xdata.len(){
        if xdata[i]>xnew{
            if i==0{
                return (0,ydata[i]);
            }
            else{
                let x1 = xdata[i-1];
                let x2 = xdata[i];
                let y1 = ydata[i-1];
                let y2 = ydata[i];
                if x1==x2{
                    return (i-1,(y1+y2)/2.0);
                }
                else{
                    return (i-1,(y2-y1)*(xnew-x1)/(x2-x1)+y1);
                }
            }
        }

    }
    (xdata.len()-1,ydata[xdata.len()-1])
}

fn resample(xdata:&[f64],ydata:&[f64],x_new:&[f64])->Vec<f64>{
    let mut res = Vec::with_capacity(x_new.len());
    res.resize(x_new.len(), 0.0);
    let mut search_start:usize = 0;
    for (i,x) in x_new.iter().enumerate(){
        let (a,b) = estimate_linear_interp1d(xdata, ydata, *x, search_start);
        search_start = a;
        res[i] = b;
    }
    res
}

#[derive(Clone,Debug)]
pub struct SyncedSignalStretcher{
    pub source:LazyDetectorSignal,
    pub source_time:LazyTimeSignal,
    pub target_time:LazyTimeSignal,
}

impl SyncedSignalStretcher {
    pub fn new(source: LazyDetectorSignal, source_time: LazyTimeSignal, target_time: LazyTimeSignal) -> Self {
        Self { source, source_time, target_time }
    }
}

impl LazyArrayOperation<ArrayND<f64>> for SyncedSignalStretcher{
    fn length(&self)->usize {
        self.target_time.length()
    }

    fn calculate_overhead(&self,start:usize, end:usize)->usize {
        //let start_time = self.target_time.request_range();
        let (src_start,src_end) = get_bounds(&self.source_time, &self.target_time, start, end);
        self.source.calculate_overhead(src_start,src_end)
    }

    fn request_range(&self,start:usize, end:usize)->ArrayND<f64> {
        let (src_start,src_end) = get_bounds(&self.source_time, &self.target_time, start, end);
        let src_spatial = self.source.request_range(src_start,src_end);
        let src_temporal = self.source_time.request_range(src_start,src_end);

        let mut target_shape:Vec<usize> = src_spatial.shape.clone().into();
        target_shape[0] = end-start;


        let threadcount = num_cpus::get();
        let mut threads:VecDeque<thread::JoinHandle<()>> = VecDeque::with_capacity(10);
        let frame_size = target_shape.iter().skip(1).fold(1usize,|a,b| a*b);

        let target = Arc::new(Mutex::new(ArrayND::new(target_shape.clone(), 0.0)));
        let spatial_source = Arc::new(src_spatial);
        let temporal_source = Arc::new(src_temporal);
        let temporal_target = Arc::new(self.target_time.request_range(start, end));

        for pixel in 0usize..frame_size{
            free_threads(&mut threads, threadcount);
            let length = end-start;
            let small_length = src_end-src_start;
            let sp_src = spatial_source.clone();
            let tmp_src = temporal_source.clone();
            let tmp_tgt = temporal_target.clone();

            let tgt = target.clone();
            let handle = thread::spawn(move || {
                let mut src_sliced:Vec<f64> = Vec::with_capacity(small_length);
                for j in 0..small_length{
                    src_sliced.push(sp_src.flat_data[j*frame_size+pixel])
                }
                let res = resample(tmp_src.as_ref(), &src_sliced, tmp_tgt.as_ref());
                let mut tgt_lock = tgt.lock().unwrap();
                for j in 0..length{
                    tgt_lock.flat_data[j*frame_size+pixel] = res[j];
                }
            });
            threads.push_back(handle);
        }

        free_threads(&mut threads, 1);
        let lock = Arc::try_unwrap(target).unwrap();
        lock.into_inner().unwrap()
    }
}

#[derive(Clone,Debug)]
pub struct SyncedTriggerStretcher{
    pub source:LazyTrigger,
    pub source_time:LazyTimeSignal,
    pub target_time:LazyTimeSignal,

}

impl SyncedTriggerStretcher {
    pub fn new(source: LazyTrigger, source_time: LazyTimeSignal, target_time: LazyTimeSignal) -> Self {
        Self { source, source_time, target_time }
    }
}

impl LazyArrayOperation<ArrayND<bool>> for SyncedTriggerStretcher{
    fn length(&self)->usize {
        self.target_time.length()
    }

    fn calculate_overhead(&self,start:usize, end:usize)->usize {
        //let start_time = self.target_time.request_range();
        let (src_start,src_end) = get_bounds(&self.source_time, &self.target_time, start, end);
        self.source.calculate_overhead(src_start,src_end)
    }

    fn request_range(&self,start:usize, end:usize)->ArrayND<bool> {
        let (src_start,src_end) = get_bounds(&self.source_time, &self.target_time, start, end);
        let src_spatial = self.source.request_range(src_start,src_end);
        let src_temporal = self.source_time.request_range(src_start,src_end);

        let mut target_shape:Vec<usize> = src_spatial.shape.clone().into();
        target_shape[0] = end-start;
        println!("Request transformation: {}-{} -> {}-{}",start,end,src_start,src_end);

        let threadcount = num_cpus::get();
        let mut threads:VecDeque<thread::JoinHandle<()>> = VecDeque::with_capacity(10);
        let frame_size = target_shape.iter().skip(1).fold(1usize,|a,b| a*b);

        let target = Arc::new(Mutex::new(ArrayND::new(target_shape.clone(), false)));
        let spatial_source = Arc::new(src_spatial);
        let temporal_source = Arc::new(src_temporal);
        let temporal_target = Arc::new(self.target_time.request_range(start, end));

        for pixel in 0usize..frame_size{
            free_threads(&mut threads, threadcount);
            let length = end-start;
            let small_length = src_end-src_start;
            let sp_src = spatial_source.clone();
            let tmp_src = temporal_source.clone();
            let tmp_tgt = temporal_target.clone();

            let tgt = target.clone();
            let handle = thread::spawn(move || {
                let mut src_sliced:Vec<bool> = Vec::with_capacity(small_length);
                for j in 0..small_length{
                    src_sliced.push(sp_src.flat_data[j*frame_size+pixel])
                }
                let mut res = Vec::with_capacity(small_length);
                let mut scan_index:usize = 0;
                for j in 0..length{
                    if scan_index<tmp_src.len()-1{
                        while tmp_tgt[j]>tmp_src[scan_index]{
                            scan_index += 1;
                        }
                    }
                    res.push(src_sliced[scan_index]);
                }
                let mut tgt_lock = tgt.lock().unwrap();
                for j in 0..length{
                    tgt_lock.flat_data[j*frame_size+pixel] = res[j];
                }
            });
            threads.push_back(handle);
        }

        free_threads(&mut threads, 1);
        let lock = Arc::try_unwrap(target).unwrap();
        lock.into_inner().unwrap()
    }
}
