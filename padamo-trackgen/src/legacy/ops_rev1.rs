use std::f64::consts::PI;
use std::hash::DefaultHasher;
use std::hash::Hasher;

use abi_stable::std_types::RVec;
use padamo_api::lazy_array_operations::ArrayND;
use padamo_api::lazy_array_operations::LazyArrayOperation;
use padamo_api::lazy_array_operations::LazyDetectorSignal;
use statrs::distribution::ContinuousCDF;
use crate::ensquared_energy;
use crate::ensquared_energy::detector::DetectorWireframe;

use statrs::distribution::Normal;

use padamo_api::function_operator::DoubleFunctionOperatorBox;

/// Compatible implementation for legacy trackgen

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


#[derive(Clone,Debug)]
pub struct LazyTriangularE0Track{
    pub data:LazyDetectorSignal,
    pub detector:DetectorWireframe,
    pub pivot_frame:f64,

    pub attack_frames:f64,
    pub sustain_frames:f64,
    pub decay_frames:f64,

    pub v0:f64,
    pub a0:f64,
    pub phi0:f64,
    pub x0:f64,
    pub y0:f64,

    pub e_min:f64,
    pub e_max:f64,

    pub sigma_x:f64,
    pub sigma_y:f64,

    pub motion_blur_steps:usize
}

impl LazyTriangularE0Track{


    #[inline]
    fn start_effect_time(&self)->f64{
        self.pivot_frame-self.attack_frames
    }

    #[inline]
    fn end_effect_time(&self)->f64{
        self.pivot_frame+self.sustain_frames+self.decay_frames
    }

    fn calculate_lc(&self, absolute_time:f64)->f64{
        let t1 = self.pivot_frame-self.attack_frames;
        let t2 = self.pivot_frame;
        let t3 = self.pivot_frame+self.sustain_frames;
        let t4 = self.pivot_frame+self.sustain_frames+self.decay_frames;
        if absolute_time<t1{
            //Before event
            0.0
        }
        else if absolute_time<t2{
            //attack
            if t2 == t1{
                self.e_max
            }
            else{
                self.e_min+(absolute_time-t1)/(t2-t1)*(self.e_max-self.e_min)
            }
        }
        else if absolute_time<t3{
            //sustain
            self.e_max
        }
        else if absolute_time<t4{
            //decay
            if t3==t4{
                self.e_max
            }
            else{
                self.e_max+(absolute_time-t3)/(t4-t3)*(self.e_min-self.e_max)
            }
        }
        else{
            0.0
        }
    }
}

impl LazyArrayOperation<ArrayND<f64>> for LazyTriangularE0Track{
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
        // match &self.data{
        //     TrackData::Background(src)=>{src.length()},
        //     TrackData::Artificial { length }=>{*length},
        // }
    }

    #[allow(clippy::let_and_return)]
    fn request_range(&self,start:usize,end:usize,) -> ArrayND<f64> where {
        // let bg = self.data.request_range(start,end); /*match &self.data {
        //     TrackData::Background(src) => src.request_range(start,end),
        //     TrackData::Artificial { length:_} => {
        //         let src_shape = self.detector.shape.clone();
        //         let mut tgt_shape = vec![end-start];
        //         tgt_shape.extend(src_shape);
        //         ArrayND::new(tgt_shape, 0.0)
        //     },
        // };*/
        let bg = self.data.request_range(start,end);


        let effect_start = self.start_effect_time();
        let effect_end = self.end_effect_time();

        if (start as f64)>effect_end || (end as f64)<effect_start{
            bg
        }
        else{
            let mut data = bg;

            let frame_size = data.shape.iter().skip(1).fold(1, |a,b| a*b);

            let actual_start = (effect_start.floor() as usize).max(start);
            let actual_end = (effect_end.ceil() as usize).min(end);

            //let actual_offset = actual_start-start;
            //println!("ACTUAL SLICE {}..{}",actual_start,actual_end);
            for t_usize in actual_start..actual_end{
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
}





#[derive(Clone,Debug)]
pub struct ArtificialTime{
    length:usize,
    base:f64,
}

impl ArtificialTime {
    pub fn new(length: usize, base:f64) -> Self {
        Self { length, base }
    }
}

impl LazyArrayOperation<RVec<f64>> for ArtificialTime{
    #[allow(clippy::let_and_return)]
    fn length(&self,) -> usize where {
        self.length
    }

    #[allow(clippy::let_and_return)]
    fn request_range(&self,start:usize,end:usize,) -> RVec<f64> where {
        (start..end).map(|x| x as f64 + self.base).collect()
    }

}


#[derive(Clone,Debug)]
pub struct ArtificialBlankSignal{
    length:usize,
    shape:Vec<usize>
}

impl ArtificialBlankSignal {
    pub fn new(length: usize, shape:Vec<usize>) -> Self {
        Self { length, shape }
    }
}

impl LazyArrayOperation<ArrayND<f64>> for ArtificialBlankSignal{
    #[allow(clippy::let_and_return)]
    fn length(&self,) -> usize where {
        self.length
    }

    #[allow(clippy::let_and_return)]
    fn request_range(&self,start:usize,end:usize,) -> ArrayND<f64> where {
        let mut tgt_shape = Vec::with_capacity(self.shape.len()+1);
        tgt_shape.push(end-start);
        tgt_shape.extend(&self.shape);
        ArrayND::new(tgt_shape, 0.0)
    }

}


#[derive(Clone,Debug)]
pub struct LazyAdditiveNormalNoise{
    source:LazyDetectorSignal,
    seed:i64,
    sigma:f64,
}

impl LazyAdditiveNormalNoise {
    pub fn new(source: LazyDetectorSignal, seed: i64, sigma: f64) -> Self { Self { source, seed, sigma } }
}

impl LazyArrayOperation<ArrayND<f64>> for LazyAdditiveNormalNoise{
    fn length(&self,) -> usize where {
        self.source.length()
    }

    fn calculate_overhead(&self,start:usize,end:usize,) -> usize where {
        self.source.calculate_overhead(start,end)
    }

    fn request_range(&self,start:usize,end:usize,) -> ArrayND<f64> where {
        let mut background = self.source.request_range(start, end);
        let norm = u64::MAX as f64;
        let dist = Normal::new(0.0,self.sigma).unwrap();
        for item in background.enumerate(){
            let mut hashing_index = item.clone();
            hashing_index[0] += start;
            let mut hasher = DefaultHasher::new();
            hasher.write_i64(self.seed);
            for x in hashing_index.iter(){
                // usize is system dependent type. Using u64 to make it consistent.
                hasher.write_u64(*x as u64);
            }
            let hashed = hasher.finish();

            // This value is uniformly distributed in [0..1]
            let data = (hashed as f64)/norm;

            //Let's make it normal. It is easy to transform uniform distribution in [0..1] into any distribution with known quantile.
            //quantile(N) = mu + sigma*sqrt(2)*erfinv(2*p-1)
            //Here mu=0.

            let normalized = dist.inverse_cdf(data);

            background[&item] += normalized;
        }
        background
    }
}
