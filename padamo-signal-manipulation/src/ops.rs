use std::{collections::VecDeque, fmt::Debug, sync::{Arc, Mutex}, thread};

use abi_stable::std_types::RVec;
use padamo_api::lazy_array_operations::{ndim_array::ArrayND, LazyArrayOperation, LazyArrayOperationBox, LazyDetectorSignal, LazyTimeSignal};

pub fn free_threads(threads: &mut VecDeque<thread::JoinHandle<()>>, threadcount:usize){
    while threads.len()>=threadcount{
        //println!("Working threads: {}",threads.len());
        if let Some(handle)= threads.pop_front() {
            if handle.is_finished(){
                //println!("Freeing one handle");
                if let Err(e) = handle.join(){
                    println!("{:?}",e);
                }
                //println!("Freed one handle");
            }
            else{
                threads.push_back(handle)
            }
        }
        else{
            break;
        }
    }
}

#[derive(Clone,Debug)]
pub struct LazySpaceConverter{
    divider:usize,
    source:LazyDetectorSignal,
    is_sum:bool,
    frame_shape:Vec<usize>
}

impl LazySpaceConverter {
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

// fn compress<D:ndarray::Dimension+ndarray::RemoveAxis>(x:ndarray::ArrayBase<ndarray::ViewRepr<&f64>,D>, is_sum:bool)->ndarray::ArrayBase<ndarray::OwnedRepr<f64>,D::Smaller>{
//     if is_sum{
//         x.sum_axis(ndarray::Axis(0))
//     }
//     else{
//         x.mean_axis(ndarray::Axis(0)).unwrap()
//     }
// }

impl LazyArrayOperation<ArrayND<f64>> for LazySpaceConverter{

    fn length(&self,) -> usize {
        let src_len = self.source.length();
        src_len/self.divider
    }

    fn calculate_overhead(&self,start:usize,end:usize,)->usize{
        self.source.calculate_overhead(start*self.divider, end*self.divider)
    }

    fn request_range(&self,start:usize,end:usize,) -> ArrayND<f64>{
        // let start_src = start*self.divider;
        // let end_src = end*self.divider;
        let divider = self.divider;
        let is_sum = self.is_sum;
        // let raw_data:ArrayND<f64> = self.source.request_range(start_src,end_src);
        let mut tgt_shape = vec![end-start];
        tgt_shape.extend(&self.frame_shape);
        let target = Arc::new(Mutex::new(ArrayND::<f64>::new(tgt_shape.into(),0.0)));
        let threadcount = num_cpus::get();
        let mut threads:VecDeque<thread::JoinHandle<()>> = VecDeque::with_capacity(10);
        let frame_size = self.frame_shape.iter().fold(1usize,|a,b| a*b);
        for frame in 0..end-start{
            let src_part = Arc::new(self.source.request_range((start+frame)*self.divider,(start+frame+1)*self.divider));
            for pixel in 0..frame_size{
                // let length = end-start;
                let part_length = divider;
                let src = src_part.clone();
                let tgt = target.clone();
                free_threads(&mut threads, threadcount);
                let handle = thread::spawn(move || {
                    let mut sum = 0.0;
                    for j in 0..part_length{
                        sum += src.flat_data[j*frame_size+pixel];
                    }
                    if !is_sum{
                        sum/=divider as f64;
                    }
                    tgt.lock().unwrap().flat_data[frame*frame_size+pixel] = sum;
                });
                threads.push_back(handle);
            }
        }
        free_threads(&mut threads, 1);
        let lock = Arc::try_unwrap(target).unwrap();
        lock.into_inner().unwrap()

        // let frame_size = raw_data.shape.iter().skip(1).fold(1usize,|a,b| a*b);
        // // let stepped_data:Vec<_> = (0..end-start).par_bridge()
        // //     .map(|i| raw_data.slice(ndarray::s![i*divider..(i+1)*divider,..,..]))
        // //     .map(|x| compress(x, is_sum))
        // //     .collect();
        //
        // let mut tgt_shape = raw_data.shape.clone();
        // tgt_shape[0] = end-start;
        //
        // let threadcount = num_cpus::get();
        // let mut threads:VecDeque<thread::JoinHandle<()>> = VecDeque::with_capacity(10);
        //
        // let target = Arc::new(Mutex::new(ArrayND::<f64>::new(tgt_shape.into(),0.0)));
        // let source = Arc::new(raw_data);
        //
        // for pixel in 0usize..frame_size{
        //     free_threads(&mut threads, threadcount);
        //
        //     let length = end-start;
        //     let src = source.clone();
        //     let tgt = target.clone();
        //
        //     let handle = thread::spawn(move || {
        //         for i in 0..length{
        //             let mut sum:f64 = 0.0;
        //             for j in i*divider..i*divider+divider{
        //                 sum += src.flat_data[j*frame_size+pixel];
        //             }
        //             if !is_sum{
        //                 sum/=divider as f64;
        //             }
        //             tgt.lock().unwrap().flat_data[i*frame_size+pixel] = sum;
        //         }
        //     });
        //
        //     threads.push_back(handle);
        // }
        //
        // free_threads(&mut threads, 1);
        //
        // let lock = Arc::try_unwrap(target).unwrap();
        // lock.into_inner().unwrap()

        //let views:Vec<_> = stepped_data.iter().map(|x| x.view()).collect();
        //let res_data = ndarray::stack(ndarray::Axis(0), &views).unwrap();
        //let res:ArrayND<f64> = res_data.into();
        //println!("{:?}",&res.shape);
    }
}

#[derive(Clone,Debug)]
pub struct LazyTimeConverter{
    divider:usize,
    source:LazyTimeSignal,
}

impl LazyTimeConverter {
    pub fn new(divider: usize, source: LazyTimeSignal) -> Self { Self { divider, source } }
}


impl LazyArrayOperation<RVec<f64>> for LazyTimeConverter{
    fn length(&self,) -> usize {
        let src_len = self.source.length();
        src_len/self.divider
    }

    fn request_range(&self,start:usize,end:usize,) -> RVec<f64>{
        let unrarified: RVec<f64> = self.source.request_range(start*self.divider, end*self.divider);
        let rarified:Vec<_> = unrarified.into_iter().skip(self.divider/2).step_by(self.divider).collect();
        rarified.into()
    }
}

#[derive(Clone,Debug)]
pub struct CutterOperator<T:Clone+Debug>{
    start:usize,
    end:usize,
    source:LazyArrayOperationBox<T>
}

impl<T:Clone+Debug> CutterOperator<T>{
    pub fn new(start:usize, end:usize, source:LazyArrayOperationBox<T>)->Self{
        Self{start,end,source}
    }
}

impl<T:Clone+Debug> LazyArrayOperation<T> for CutterOperator<T>{
    fn length(&self,) -> usize where {
        self.end-self.start
    }
    fn calculate_overhead(&self,start:usize,end:usize,) -> usize where {
        self.source.calculate_overhead(start+self.start,end+self.start)
    }
    fn request_range(&self,start:usize,end:usize,) -> T where {
        self.source.request_range(start+self.start,end+self.start)
    }
}


#[derive(Clone,Debug)]
pub struct TimeShift{
    source: LazyTimeSignal,
    offset: f64
}

impl TimeShift {
    pub fn new(source: LazyTimeSignal, offset: f64) -> Self {
        Self { source, offset }
    }
}

impl LazyArrayOperation<RVec<f64>> for TimeShift{
    #[allow(clippy::let_and_return)]
    fn length(&self,) -> usize {
        self.source.length()
    }

    #[allow(clippy::let_and_return)]
    fn request_range(&self,start:usize,end:usize,) -> RVec<f64> {
        self.source.request_range(start,end).iter().map(|x| x+self.offset).collect()
    }

    #[allow(clippy::let_and_return)]
    fn calculate_overhead(&self,start:usize,end:usize,) -> usize {
        self.source.calculate_overhead(start,end)
    }
}
