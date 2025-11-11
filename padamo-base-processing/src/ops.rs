use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use padamo_api::lazy_array_operations::{LazyArrayOperation, LazyDetectorSignal, LazyArrayOperationBox};
use padamo_api::lazy_array_operations::ndim_array::ArrayND;
use rayon::prelude::*;

use standalone_quantiles::slide_quantile;

#[repr(C)]
#[derive(Clone,Debug, abi_stable::StableAbi)]
pub struct LazySlidingQuantile{
    source:LazyDetectorSignal,
    window:usize,
    q:f64,
}

impl LazySlidingQuantile{
    pub fn new(source:LazyDetectorSignal,window:usize, q:f64)->Self{
        Self { window, q, source }
    }

}

impl LazyArrayOperation<ArrayND<f64>> for LazySlidingQuantile{
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

        let mut target_shape = sourced.shape.clone();
        target_shape[0] = end-start;

        // let mut res: ArrayND<f64> = ArrayND::defaults(target_shape.to_vec());
        let res = Arc::new(Mutex::new(ArrayND::defaults(target_shape.to_vec())));

        let iterated_array = sourced.make_pixel_iterators();

        let q = self.q;
        let window = self.window;

        iterated_array.enumerate().par_bridge().for_each(|index_id|{
            let src = slide_quantile(iterated_array[&index_id].clone(), window, q);

            for i in 0..src.len(){
                let mut target_id = vec![i];
                target_id.extend(index_id.clone());
                res.lock().unwrap()[&target_id] = src[i];
            }

        });

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
        Arc::try_unwrap(res).unwrap().into_inner().unwrap()
    }
}

#[repr(C)]
#[derive(Clone,Debug, abi_stable::StableAbi)]
pub struct LazySkipper<T>
where
    T:Clone+Debug
{
    source: LazyArrayOperationBox<T>,
    window:usize
}

impl<T> LazySkipper<T>
where
    T:Clone+Debug
{
    pub fn new(source: LazyArrayOperationBox<T>, window: usize) -> Self { Self { source, window } }
}

impl<T> LazyArrayOperation<T> for LazySkipper<T>
where
    T:Clone+Debug
{
    fn length(&self,) -> usize where {
        self.source.length()-self.window+1
    }

    fn calculate_overhead(&self,start:usize,end:usize)->usize{
        self.source.calculate_overhead(start+self.window/2,end+self.window/2)
    }

    fn request_range(&self,start:usize,end:usize) -> T {
        self.source.request_range(start+self.window/2,end+self.window/2)
    }
}


#[repr(C)]
#[derive(Clone,Debug, abi_stable::StableAbi)]
pub struct LazySubtractor{
    a:LazyDetectorSignal,
    b:LazyDetectorSignal
}

impl LazySubtractor {
    pub fn new(a: LazyDetectorSignal, b: LazyDetectorSignal) -> Self {
        if a.length()!=b.length(){
            panic!("Signals length mismatch");
        }
        Self { a, b }

    }
}


impl LazyArrayOperation<ArrayND<f64>> for LazySubtractor{
    fn length(&self,) -> usize where {
        self.a.length()
    }

    fn calculate_overhead(&self,start:usize,end:usize) -> usize{
        self.a.calculate_overhead(start,end)+self.b.calculate_overhead(start,end)+end-start
    }

    fn request_range(&self,start:usize,end:usize) -> ArrayND<f64> {
        let a = self.a.request_range(start,end).to_ndarray();
        let b = self.b.request_range(start,end).to_ndarray();
        //println!("Shape test, {:?}, {:?}", a.shape, b.shape);
        let res = (a-b).to_owned();
        res.into()
    }


}




#[repr(C)]
#[derive(Clone,Debug, abi_stable::StableAbi)]
pub struct LazySlidingQuantileNormalize{
    source:LazyDetectorSignal,
    window:usize,
    q:f64,
    gaussmode:bool,
    variance:bool,
}

impl LazySlidingQuantileNormalize{
    pub fn new(source:LazyDetectorSignal,window:usize, q:f64, gaussmode:bool, variance:bool)->Self{
        Self { window, q, source, gaussmode, variance}
    }
}

// fn clamp_zero(x:f64)->f64{
//     if x==0.0{
//         1.0
//     }
//     else{
//         x
//     }
// }

// fn collapse_med<A,D:ndarray::Dimension>(array:ndarray::ArrayBase<A,D>){
//
// }

impl LazyArrayOperation<ArrayND<f64>> for LazySlidingQuantileNormalize{
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

        let mut target_shape = sourced.shape.clone();
        target_shape[0] = end-start;

        let res = Arc::new(Mutex::new(ArrayND::defaults(target_shape.to_vec())));


        let k = if self.gaussmode {1.4826} else {1.0};
        let q = self.q;
        let use_variance = self.variance;
        let window = self.window;

        let iterated_array = sourced.make_pixel_iterators();

        iterated_array.enumerate().par_bridge().for_each(|index_id|{
            let src = slide_quantile(iterated_array[&index_id].clone(), window, q);
            let divisors = iterated_array[&index_id].clone().skip(window/2).clone();

            for (i, divisor) in divisors.enumerate(){

                let mut divider = src[i].abs()*k;

                if use_variance{
                    divider *= divider;
                }

                let mut target_id = vec![i];
                target_id.extend(index_id.clone());
                if divider != 0.0{
                    res.lock().unwrap()[&target_id] = divisor/divider;
                }

            }

        });

        //println!("{:?}",sourced);
        // let sourced = sourced.to_ndarray();
        // let slices:Vec<_> = (0..end-start).par_bridge().map(|i| sourced.slice(ndarray::s![i..i+window,..,..])).collect();
        // let q = self.q;  // Excuse me what the f... moment. Won't work if inserted directly due to Sync+Send mumbo-jumbo;
        //
        // let slices:Vec<_> =
        //     slices.par_iter().map(|x|
        //         x
        //             .mapv(|y| f64::abs(y)*k)
        //             //.mapv(|y| if use_variance {y*y} else {y})
        //             .to_owned().quantile_axis_skipnan_mut(
        //             Axis(0),
        //             n64(q),
        //             &ndarray_stats::interpolate::Linear).unwrap()
        //             .mapv(clamp_zero))
        //     .collect();
        //
        // //println!("Calculated sliding median (after norm calculation) normalize in {:.2?}", time_start.elapsed());
        // let views:Vec<_> = slices.par_iter().map(|x| x.view()).collect();
        // //println!("FRAMES: {}",views.len());
        // let mut divider = ndarray::stack(Axis(0), &views).unwrap();

        // if use_variance{
        //     divider.par_map_inplace(|x| {*x = *x * *x});
        // }
        //
        // let divisor = sourced.slice(ndarray::s![self.window/2..self.window/2+end-start,..,..]);
        //println!("Calculated sliding median (no division) normalize in {:.2?}", time_start.elapsed());
        // let preres = (divisor.to_owned()/divider).to_owned();
        //println!("Calculated sliding median normalize in {:.2?}", time_start.elapsed());
        //println!("{:?}", preres);
        // let res:ArrayND<f64> = preres.into();
        //assert_eq!(res.shape[0],end-start);
        Arc::try_unwrap(res).unwrap().into_inner().unwrap()
    }
}

#[derive(Clone,Debug)]
pub struct LazyFlashSuppress{
    source:LazyDetectorSignal,
    q:f64
}

impl LazyFlashSuppress {
    pub fn new(source: LazyDetectorSignal, q: f64) -> Self { Self { source, q } }
}



// fn suppress_flash<D:ndarray::Dimension>(src:&mut ndarray::ArrayBase<ndarray::ViewRepr<&mut f64>,D>,q:f64){
//     let arrnd:ArrayND<f64> = src.to_owned().into();
//     let mut arrnd = arrnd.flatten().to_ndarray();
//
//     let quant = s
//     *src -= quant;
// }

fn suppress_flash(frame:&mut [f64], q:f64){
    let mut workon = frame.to_vec();
    let base = standalone_quantiles::quantile(&mut workon, q);
    frame.iter_mut().for_each(|x| *x -= base);
}

impl LazyArrayOperation<ArrayND<f64>> for LazyFlashSuppress{
    #[allow(clippy::let_and_return)]
    fn length(&self,) -> usize where {
        self.source.length()
    }

    fn calculate_overhead(&self,start:usize,end:usize,) -> usize{
        self.source.calculate_overhead(start,end)+end-start
    }

    #[allow(clippy::let_and_return)]
    fn request_range(&self,start:usize,end:usize,) -> ArrayND<f64> where {
        let mut src = self.source.request_range(start,end);
        //let q = self.q;
        //let tgt = src.clone();
        //(0..end-start).par_bridge().for_each(|i| suppress_flash(&mut src.index_axis_mut(Axis(0),i), q));
        let frame_size = src.frame_size();
        for i in 0..end-start{
            suppress_flash(&mut src.flat_data[i*frame_size..(i+1)*frame_size], self.q);
             // suppress_flash(&mut src.index_axis_mut(Axis(0),i), self.q);
        }
        src
    }
}


#[derive(Clone,Debug)]
pub struct LazyThreshold{
    pub source:LazyDetectorSignal,
    pub threshold_value:f64,
    pub blank_value:f64,
    pub invert:bool,
}

impl LazyArrayOperation<ArrayND<f64>> for LazyThreshold{
    #[allow(clippy::let_and_return)]
    fn length(&self,) -> usize where {
        self.source.length()
    }

    #[allow(clippy::let_and_return)]
    fn request_range(&self,start:usize,end:usize,) -> ArrayND<f64> where {
        let mut workon:ArrayND<f64> = self.source.request_range(start,end);
        let thresh = self.threshold_value;
        let inv = self.invert;
        let blank = self.blank_value;
        workon.flat_data.par_iter_mut().for_each(|x| {
            if (*x>thresh) == inv{
                *x = blank;
            }
        });
        workon
    }

    #[allow(clippy::let_and_return)]
    fn calculate_overhead(&self,start:usize,end:usize,) -> usize where {
        self.source.calculate_overhead(start,end)
    }
}
