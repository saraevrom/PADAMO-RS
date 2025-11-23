use std::{f64::consts::PI, sync::{Arc, Mutex}};

use abi_stable::std_types::RVec;
use atomic_float::AtomicF64;
use rayon::prelude::*;
use padamo_api::lazy_array_operations::{LazyArrayOperation,ArrayND, LazyDetectorSignal};
use crate::lambert::lambertw0;
use padamo_api::constants;
use padamo_api::prelude::{CalculationConstant,ConstantContentContainer,ExecutionError};

#[derive(Clone,Copy,Debug)]
pub struct PhysicalFFConstants{
    pub dt: f64, //Основное временное разрешение в нс
    pub wt: f64, //Прозрачность входного окна
    pub lt: f64, //Прозрачность линзы
    pub pix_fov: f64, //Поле зрения пикселя в стеррадианах
    pub s:f64, //Площадь входного окна в см^2
    pub t:f64, //Time sample в секундах
    pub e0: f64 //Энергия одного фотона
}

impl Default for PhysicalFFConstants{
    fn default() -> Self {
        Self {
            dt: 2500.0,
            wt: 0.86,
            lt: 0.93,
            pix_fov: (2.88/160.0)*(2.88/160.0),
            s: PI*25.0/4.0,
            t: 0.001,
            e0: 1.0,
        }
    }
}

impl PhysicalFFConstants{
    pub fn get_nts(&self)->f64{
        self.t/self.dt*1e9
    }
    pub fn get_cr_to_int(&self)->f64{
        1.0/(self.wt*self.lt*self.pix_fov*self.s*self.t*self.e0)
    }

    pub fn constlist()->RVec<CalculationConstant>{
        let x:Self = Self::default();
        constants![
            ("dt",x.dt),
            ("wt",x.wt),
            ("lt",x.lt),
            ("pix_fov",x.pix_fov),
            ("s",x.s),
            ("t",x.t),
            ("e0",x.e0)
        ]
    }

    pub fn from_constlist(constlist:&ConstantContentContainer)->Result<Self, ExecutionError>{
        Ok(Self {
            dt: constlist.request_float("dt")?,
            wt: constlist.request_float("wt")?,
            lt: constlist.request_float("lt")?,
            pix_fov: constlist.request_float("pix_fov")?,
            s: constlist.request_float("s")?,
            t: constlist.request_float("t")?,
            e0: constlist.request_float("e0")?,
        })
    }
}

#[derive(Debug,Clone)]
pub struct PhysicalFF{
    source:LazyDetectorSignal,
    eff_2d:ArrayND<f64>,
    tau_calib:ArrayND<f64>,

    nts:f64,
    cr_to_int:f64,
    dt:f64

}

impl PhysicalFF {
    pub fn new(source: LazyDetectorSignal, eff_2d: ArrayND<f64>, tau_calib: ArrayND<f64>, constants:PhysicalFFConstants) -> Self {
        Self { source, eff_2d, tau_calib,
            nts:constants.get_nts(),
            cr_to_int:constants.get_cr_to_int(),
            dt:constants.dt,
        }

    }
}


//Константы
// const DT: f64 = 2500.0; //Основное временное разрешение в нс
// const WT: f64 = 0.86;   //Прозрачность входного окна
// const LT:f64 = 0.93;    //Прозрачность линзы
//a = 1;      %Индекс начала отрезка
//b = 100;     %Индекс конца отрезка
// const PIX_FOV:f64 = (2.88/160.0)*(2.88/160.0);  //Поле зрения пикселя в стеррадианах
// const S:f64 = PI*25.0/4.0; //Площадь входного окна в см^2
//N_ch = 256;  %Число каналов в расчет lightcurvesum_global_cor
//N=240400;
// const T:f64 = 0.001;  //Time sample в секундах
// const NTS:f64 = T/DT*1e9;
// const CR_TO_INT:f64 = 1.0/(WT*LT*PIX_FOV*S*T);

impl LazyArrayOperation<ArrayND<f64>> for PhysicalFF{
    #[allow(clippy::let_and_return)]
    fn length(&self,) -> usize where {
        self.source.length()
    }

    fn calculate_overhead(&self,start:usize,end:usize,) -> usize {
        2*self.source.calculate_overhead(start,end)
    }

    #[allow(clippy::let_and_return)]
    fn request_range(&self,start:usize,end:usize,) -> ArrayND<f64>{
        let src_data = self.source.request_range(start,end);
        // let tgt:Arc<Mutex<ArrayND<f64>>> = Arc::new(Mutex::new(ArrayND::new(src_data.shape.clone().into(), 0.0)));
        let tgt_flat_len:usize = src_data.shape.iter().map(|x|*x).product();
        let mut tgt_flat:Vec<AtomicF64> = Vec::with_capacity(tgt_flat_len);
        tgt_flat.resize_with(tgt_flat_len, || AtomicF64::new(0.0));

        let enumerator = src_data.enumerate();

        let tau_calib = &self.tau_calib;
        let eff_2d = &self.eff_2d;

        let nts = self.nts;
        let cr_to_int = self.cr_to_int;
        let dt = self.dt;

        enumerator.par_bridge().for_each(|i|{
            let mut pixel_index = i.clone();
            pixel_index.drain(0..1);
            let x =src_data[&i];
            let eff = eff_2d[&pixel_index];
            let tau = tau_calib[&pixel_index];
            let b = tau*eff/dt;
            let v = if eff>0.0 && tau>0.0{
                //-CR_TO_INT*NTS*lambertw0(-b*x/NTS/eff)/b
                -cr_to_int*nts*lambertw0(-b*x/nts/eff)/b
            }
            else{0.0};

            // tgt.lock().unwrap()[&i] = v;
            let off = padamo_arraynd::calculate_offset(&src_data.shape,&i);
            tgt_flat[off].fetch_add(v, std::sync::atomic::Ordering::Relaxed);
        });


        // let tgt = Arc::try_unwrap(tgt).unwrap();
        // tgt.into_inner().unwrap()
        let tgt = ArrayND {shape: src_data.shape.clone().into(), flat_data:tgt_flat.drain(..).map(|x| x.into_inner()).collect()};
        tgt.assert_shape();
        tgt
    }
}


#[derive(Clone,Debug)]
pub struct ApplyByMap{
    source:LazyDetectorSignal,
    coeffs:ArrayND<f64>,
    operation: fn(f64,f64)->f64,
}

impl ApplyByMap {
    pub fn new(source: LazyDetectorSignal, coeffs: ArrayND<f64>, operation: fn(f64,f64)->f64) -> Self {
        Self { source, coeffs, operation }
    }
}

impl LazyArrayOperation<ArrayND<f64>> for ApplyByMap{
    fn length(&self,) -> usize where {
        self.source.length()
    }

    fn calculate_overhead(&self,start:usize,end:usize) -> usize {
        self.source.calculate_overhead(start,end)
    }

    fn request_range(&self,start:usize,end:usize) -> ArrayND<f64>{
        let src_data = self.source.request_range(start,end);
        // let tgt:Arc<Mutex<ArrayND<f64>>> = Arc::new(Mutex::new(ArrayND::new(src_data.shape.clone().into(), 0.0)));
        let tgt_flat_len:usize = src_data.shape.iter().map(|x|*x).product();
        let mut tgt_flat:Vec<AtomicF64> = Vec::with_capacity(tgt_flat_len);
        tgt_flat.resize_with(tgt_flat_len, || AtomicF64::new(0.0));

        let enumerator = src_data.enumerate();

        let coeffs = &self.coeffs;
        enumerator.par_bridge().for_each(|i|{
            let mut pixel_index = i.clone();
            pixel_index.drain(0..1);
            let x =src_data[&i];
            let coeff = coeffs[&pixel_index];
            let v = (self.operation)(x,coeff);
            // tgt.lock().unwrap()[&i] = v;
            let off = padamo_arraynd::calculate_offset(&src_data.shape,&i);
            tgt_flat[off].fetch_add(v, std::sync::atomic::Ordering::Relaxed);
        });
        // let tgt = Arc::try_unwrap(tgt).unwrap();
        // tgt.into_inner().unwrap()
        let tgt = ArrayND {shape: src_data.shape.clone().into(), flat_data:tgt_flat.drain(..).map(|x| x.into_inner()).collect()};
        tgt.assert_shape();
        tgt
    }
}
