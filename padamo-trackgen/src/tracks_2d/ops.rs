use std::f64::consts::PI;

use padamo_api::lazy_array_operations::ArrayND;
use padamo_api::lazy_array_operations::LazyArrayOperation;
use padamo_api::lazy_array_operations::LazyDetectorSignal;
use crate::ensquared_energy;
use crate::ensquared_energy::detector::DetectorWireframe;


use padamo_api::function_operator::DoubleFunctionOperatorBox;

#[derive(Clone,Debug)]
pub struct LazyAnyLCGaussTrack{
    pub data:LazyDetectorSignal,
    pub detector:DetectorWireframe,
    pub pivot_frame:f64,
    pub lc:DoubleFunctionOperatorBox,


    pub v0:f64,
    pub a0:f64,
    pub phi0:f64,
    pub x0:f64,
    pub y0:f64,


    pub sigma_x:f64,
    pub sigma_y:f64,

    pub motion_blur_steps:usize
}

impl LazyAnyLCGaussTrack{

    fn calculate_lc(&self, absolute_time:f64)->f64{
        self.lc.calculate(absolute_time-self.pivot_frame)
    }
}

impl LazyArrayOperation<ArrayND<f64>> for LazyAnyLCGaussTrack{
    fn calculate_overhead(&self,start:usize,end:usize,) -> usize where {
        // match &self.data{
        //     TrackData::Background(src)=>{src.calculate_overhead(start,end)},
        //     TrackData::Artificial { length: _ }=>{end-start},
        // }
        self.data.calculate_overhead(start,end)

    }

    #[allow(clippy::let_and_return)]
    fn length(&self,) -> usize where {
        self.data.length()
    }

    #[allow(clippy::let_and_return)]
    fn request_range(&self,start:usize,end:usize,) -> ArrayND<f64> where {
        let bg = self.data.request_range(start,end);

        let mut data = bg;

        let frame_size = data.shape.iter().skip(1).fold(1, |a,b| a*b);

        //let actual_start = (effect_start.floor() as usize).max(start);
        //let actual_end = (effect_end.ceil() as usize).min(end);

        //let actual_offset = actual_start-start;
        //println!("ACTUAL SLICE {}..{}",actual_start,actual_end);
        for t_usize in start..end{
            for t_sweep_index in 0..self.motion_blur_steps{
                let (divider,offset) = if self.motion_blur_steps>1{
                    (self.motion_blur_steps as f64,(t_sweep_index as f64)/((self.motion_blur_steps-1) as f64)-0.5)
                }
                else{
                    (1.0,0.0)
                };

                let t = t_usize as f64 + offset;
                let lc = self.calculate_lc(t)/divider;
                let dt = t-self.pivot_frame;
                let displacement = self.v0*dt+self.a0*dt*dt/2.0;
                let x = self.x0+displacement*(self.phi0*PI/180.0).cos();
                let y = self.y0+displacement*(self.phi0*PI/180.0).sin();

                let spot = ensquared_energy::gauss_spot(&self.detector, x, y, self.sigma_x, self.sigma_y,lc);
                //println!("{:?}",&spot.flat_data);
                for i in 0..spot.flat_data.len(){
                    data.flat_data[(t_usize-start)*frame_size+i] += spot.flat_data[i];
                }
            }

        }
        //println!("{:?}",&data.flat_data);
        data

    }
}


#[derive(Clone,Debug)]
pub struct LazyAnyLCMoffatTrack{
    pub data:LazyDetectorSignal,
    pub detector:DetectorWireframe,
    pub pivot_frame:f64,
    pub lc:DoubleFunctionOperatorBox,


    pub v0:f64,
    pub a0:f64,
    pub phi0:f64,
    pub x0:f64,
    pub y0:f64,


    pub alpha:f64,
    pub beta:f64,
    pub normalize:bool,

    pub motion_blur_steps:usize
}


impl LazyAnyLCMoffatTrack{

    fn calculate_lc(&self, absolute_time:f64)->f64{
        self.lc.calculate(absolute_time-self.pivot_frame)
    }
}

impl LazyArrayOperation<ArrayND<f64>> for LazyAnyLCMoffatTrack{
    fn calculate_overhead(&self,start:usize,end:usize,) -> usize where {
        // match &self.data{
        //     TrackData::Background(src)=>{src.calculate_overhead(start,end)},
        //     TrackData::Artificial { length: _ }=>{end-start},
        // }
        self.data.calculate_overhead(start,end)

    }

    #[allow(clippy::let_and_return)]
    fn length(&self,) -> usize where {
        self.data.length()
    }

    #[allow(clippy::let_and_return)]
    fn request_range(&self,start:usize,end:usize,) -> ArrayND<f64> where {
        let bg = self.data.request_range(start,end);

        let mut data = bg;

        let frame_size = data.shape.iter().skip(1).fold(1, |a,b| a*b);

        //let actual_start = (effect_start.floor() as usize).max(start);
        //let actual_end = (effect_end.ceil() as usize).min(end);

        //let actual_offset = actual_start-start;
        //println!("ACTUAL SLICE {}..{}",actual_start,actual_end);
        let norm = if self.normalize {(self.beta-1.0)/(std::f64::consts::PI*self.alpha*self.alpha)} else {1.0};
        for t_usize in start..end{
            for t_sweep_index in 0..self.motion_blur_steps{
                let (divider,offset) = if self.motion_blur_steps>1{
                    (self.motion_blur_steps as f64,(t_sweep_index as f64)/((self.motion_blur_steps-1) as f64)-0.5)
                }
                else{
                    (1.0,0.0)
                };

                let t = t_usize as f64 + offset;
                let lc = self.calculate_lc(t)*norm/divider;
                let dt = t-self.pivot_frame;
                let displacement = self.v0*dt+self.a0*dt*dt/2.0;
                let x = self.x0+displacement*(self.phi0*PI/180.0).cos();
                let y = self.y0+displacement*(self.phi0*PI/180.0).sin();

                let spot = ensquared_energy::moffat_spot(&self.detector, x, y, self.alpha, self.beta,lc);
                //println!("{:?}",&spot.flat_data);
                for i in 0..spot.flat_data.len(){
                    data.flat_data[(t_usize-start)*frame_size+i] += spot.flat_data[i];
                }
            }

        }
        //println!("{:?}",&data.flat_data);
        data

    }
}
