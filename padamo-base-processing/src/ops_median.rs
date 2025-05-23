use std::fmt::Debug;

use padamo_api::lazy_array_operations::{LazyArrayOperation, LazyDetectorSignal};
use padamo_api::lazy_array_operations::ndim_array::ArrayND;
use rayon::prelude::*;
use crate::moving_median::temporal_moving_median;

#[repr(C)]
#[derive(Clone,Debug, abi_stable::StableAbi)]
pub struct LazySlidingMedian{
    source:LazyDetectorSignal,
    window:usize,
}

impl LazySlidingMedian{
    pub fn new(source:LazyDetectorSignal,window:usize)->Self{
        Self { window, source }
    }

}

impl LazyArrayOperation<ArrayND<f64>> for LazySlidingMedian{
    fn length(&self,) -> usize where {
        self.source.length()-self.window+1
    }

    fn calculate_overhead(&self,start:usize,end:usize) -> usize{
        2*(end-start)+self.window-1
    }

    fn request_range(&self,start:usize,end:usize) -> ArrayND<f64> {

        //let time_start = Instant::now();
        let range_start = start;
        let range_end = end+self.window-1;
        //let range_len = range_end - range_start;

        let sourced: ArrayND<f64> = self.source.request_range(range_start,range_end);

        //println!("{:?}",sourced);
        // let sourced = sourced.to_ndarray();
        // let slices:Vec<_> = (0..end-start).map(|i| sourced.slice(ndarray::s![i..i+self.window,..,..])).collect();
        // let q = self.q; // Sync+Send witchery.
        //
        // let slices:Vec<_> =
        //     slices.par_iter().map(|x| {
        //         x.to_owned().quantile_axis_skipnan_mut(
        //                 Axis(0),
        //                 n64(q),
        //                 &ndarray_stats::interpolate::Linear).unwrap()
        //     })
        //     .collect();
        // let views:Vec<_> = slices.par_iter().map(|x| x.view()).collect();
        // //println!("FRAMES: {}",views.len());
        // let preres = ndarray::stack(Axis(0), &views).unwrap();
        // let res:ArrayND<f64> = preres.into();
        //println!("Calculated sliding median in {:.2?}", time_start.elapsed());
        //println!("SHAPE OF RESULT {:?}", &res.shape);


        let res = temporal_moving_median(sourced,self.window);
        //let testdata = res.clone().to_ndarray();
        //println!("{:?}", testdata);
        //assert_eq!(res.shape[0],end-start);
        res
    }
}




#[repr(C)]
#[derive(Clone,Debug, abi_stable::StableAbi)]
pub struct LazySlidingMedianNormalize{
    source:LazyDetectorSignal,
    window:usize,
    gaussmode:bool,
    variance:bool
}

impl LazySlidingMedianNormalize{
    pub fn new(source:LazyDetectorSignal,window:usize, gaussmode:bool, variance:bool)->Self{
        Self { window, source, gaussmode, variance}
    }
}


// fn collapse_med<A,D:ndarray::Dimension>(array:ndarray::ArrayBase<A,D>){
//
// }
/*
fn safe_divide(a:f64,b:f64)->f64{
    if b==0{
        return 0.0;
    }
    else{
        return a/b;
    }
}
*/

fn safe_divide_arrs(a:ArrayND<f64>,b:ArrayND<f64>)->ArrayND<f64>{
    let mut a = a;
    //let shape = b.shape;
    for i in 0..b.flat_data.len(){
        if b.flat_data[i]==0.0{
            a.flat_data[i] = 0.0;
        }
        else{
            a.flat_data[i] = a.flat_data[i]/b.flat_data[i];
        }
    }
    a
}

impl LazyArrayOperation<ArrayND<f64>> for LazySlidingMedianNormalize{
    fn length(&self,) -> usize where {
        self.source.length()-self.window+1
    }

    fn calculate_overhead(&self,start:usize,end:usize) -> usize{
        2*(end-start)+self.window-1
    }

    fn request_range(&self,start:usize,end:usize) -> ArrayND<f64> {
        //let time_start = Instant::now();
        let range_start = start;
        let range_end = end+self.window-1;
        //let range_len = range_end - range_start;

        let sourced = self.source.request_range(range_start,range_end);
        let k = if self.gaussmode {1.4826} else {1.0};
        let use_variance = self.variance;
        let window = self.window;

        let dims = sourced.shape.len();
        let sourced1 = sourced.clone();
        let sourced1 = sourced1.to_ndarray();

        let mut slice_part:Vec<ndarray::SliceInfoElem> = vec![ndarray::SliceInfoElem::Slice { start: (self.window/2) as isize, end: Some((self.window/2+end-start) as isize), step: 1 }];
        for _ in 1..dims{
            slice_part.push(ndarray::SliceInfoElem::Slice { start: 0, end: None, step: 1 });
        }

        let divisor = sourced1.slice(slice_part.as_slice());
        let divisor = divisor.to_owned();
        let divisor:ArrayND<f64> = divisor.into();

        let sourced = {
            let flat_data = sourced.flat_data;
            let flat_data: Vec<f64> = flat_data.par_iter().map(|x| (*x).abs()*k).collect();
            let flat_data = flat_data.into(); // Turn into rvec
            let shape = sourced.shape;
            ArrayND{shape, flat_data}
        };


        let mut divider = temporal_moving_median(sourced,window);
        if use_variance{
            divider.flat_data = divider.flat_data.to_vec().par_drain(..).map(|x| {x*x}).collect::<Vec<f64>>().into();
        }


        //divisor
        safe_divide_arrs(divisor,divider)
        // let preres = (divisor/divider).to_owned();
        //
        //
        // preres.into()
    }
}

