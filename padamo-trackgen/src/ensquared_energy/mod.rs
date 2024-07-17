use padamo_api::lazy_array_operations::ArrayND;
use padamo_detectors::polygon::{DetectorContent,DetectorPixel};

use std::f64::consts::PI;

pub mod detector;
pub mod integral;


pub fn load_detector(path:&str)->Option<detector::DetectorWireframe>{
    let s = match std::fs::read_to_string(path) {
        Ok(v)=>{v},
        Err(_)=> {return None;}
    };
    let d= serde_json::from_str(&s);
    let d:DetectorContent = match d{
        Ok(v)=>{v},
        Err(_)=> {return None;}
    };

    Some(detector::wireframe(d))
}

pub fn make_frame<T: Fn(f64,f64) -> f64>(wireframe:&detector::DetectorWireframe, f:&T)->ArrayND<f64>{
    let mut arr = ArrayND::new(wireframe.shape.clone().into(), 0.0);
    for i in wireframe.triangles.iter(){
        if arr.index_compatible(&i.index){
            let mut accum = 0.0;
            for tri in i.triangles.iter(){
                accum += integral::integral_arbitrary_triangle(f, tri[0], tri[1], tri[2]);
            }
            arr[&i.index] = accum;
        }
        else{
            println!("Index {:?} is incompatible with array shape {:?}",i.index, arr.shape);
        }
    }
    arr
}


pub fn gauss_spot(wireframe:&detector::DetectorWireframe,x0:f64,y0:f64,sigma_x:f64,sigma_y:f64, energy:f64)->ArrayND<f64>{
    let gauss = |x:f64,y:f64| {
        let m1 = (x-x0)*(x-x0)/(2.0*sigma_x*sigma_x);
        let m2 = (y-y0)*(y-y0)/(2.0*sigma_y*sigma_y);
        energy*f64::exp(-m1-m2)/(2.0*PI*sigma_x*sigma_y)
    };
    make_frame(wireframe, &gauss)
}


pub fn moffat_spot(wireframe:&detector::DetectorWireframe,x0:f64,y0:f64,alpha:f64,beta:f64, energy:f64)->ArrayND<f64>{
    let alpha_sqr = alpha*alpha;
    let gauss = |x:f64,y:f64| {
        let dx = x-x0;
        let dy = y-y0;
        let r_sqr = dx*dx+dy*dy;
        energy*(1.0+(r_sqr/alpha_sqr)).powf(-beta)
    };
    make_frame(wireframe, &gauss)
}
