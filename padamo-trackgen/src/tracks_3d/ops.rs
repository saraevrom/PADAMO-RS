use padamo_api::lazy_array_operations::ArrayND;
use padamo_api::lazy_array_operations::LazyArrayOperation;
use padamo_api::lazy_array_operations::LazyDetectorSignal;
use crate::ensquared_energy;
use crate::ensquared_energy::detector::DetectorWireframe;

use padamo_api::function_operator::DoubleFunctionOperatorBox;

#[derive(Clone,Debug)]
pub struct LazyGaussPSFMeteorTrack{
    pub motion_blur_steps:usize,
    pub modify_intensity:bool,

    pub data:LazyDetectorSignal,
    pub detector:DetectorWireframe,

    pub pivot_frame:f64,
    pub lc:DoubleFunctionOperatorBox,

    pub x0:f64,
    pub y0:f64,
    pub z0:f64,

    pub v0_x:f64,
    pub v0_y:f64,
    pub v0_z:f64,

    pub a0_x:f64,
    pub a0_y:f64,
    pub a0_z:f64,

    pub f:f64,

    pub sigma_x:f64,
    pub sigma_y:f64,

}

impl LazyGaussPSFMeteorTrack{
    fn kinematics(&self, t:f64)->(f64,f64,f64){
        let dt = t-self.pivot_frame;
        (
            self.x0+self.v0_x*dt+self.a0_x*dt*dt/2.0,
            self.y0+self.v0_y*dt+self.a0_y*dt*dt/2.0,
            self.z0+self.v0_z*dt+self.a0_z*dt*dt/2.0,
        )
    }

    fn kinematics_2d(&self, t:f64)->(f64,f64){
        let (x,y,z) = self.kinematics(t);
        (x*self.f/z,y*self.f/z)
    }

    fn intensity_mod(&self, t:f64)->f64{
        let d0 = self.x0*self.x0+self.y0*self.y0+self.z0*self.z0;
        let (x,y,z) = self.kinematics(t);
        let d = x*x+y*y+z*z;
        if d==0.0{
            return 0.0;
        }
        else{
            return d0/d;
        }
    }

    fn calculate_lc(&self, absolute_time:f64)->f64{
        let mut res = self.lc.calculate(absolute_time-self.pivot_frame);
        if self.modify_intensity{
            res = res*self.intensity_mod(absolute_time);
        }
        res
    }
}

impl LazyArrayOperation<ArrayND<f64>> for LazyGaussPSFMeteorTrack{
    fn calculate_overhead(&self,start:usize,end:usize,) -> usize where {
        // match &self.data{
        //     TrackData::Background(src)=>{src.calculate_overhead(start,end)},
        //     TrackData::Artificial { length: _ }=>{end-start},
        // }
        self.data.calculate_overhead(start,end)
    }

    fn length(&self,) -> usize{
        self.data.length()
    }

    fn request_range(&self,start:usize,end:usize,) -> ArrayND<f64>where {
        let bg = self.data.request_range(start,end);

        let mut data = bg;

        let frame_size = data.shape.iter().skip(1).fold(1, |a,b| a*b);

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
                let (x,y) = self.kinematics_2d(t);

                let spot = ensquared_energy::gauss_spot(&self.detector, x, y, self.sigma_x, self.sigma_y,lc);
                for i in 0..spot.flat_data.len(){
                    data.flat_data[(t_usize-start)*frame_size+i] += spot.flat_data[i];
                }
            }

        }
        data
    }
}
