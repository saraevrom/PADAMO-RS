use std::{f64::consts::PI, sync::{Arc, Mutex}};

use rayon::prelude::*;
use padamo_api::lazy_array_operations::{LazyArrayOperation,ArrayND, LazyDetectorSignal};
use crate::lambert::lambertw0;

#[derive(Debug,Clone)]
pub struct PhysicalFF{
    source:LazyDetectorSignal,
    eff_2d:ArrayND<f64>,
    tau_calib:ArrayND<f64>,
}

impl PhysicalFF {
    pub fn new(source: LazyDetectorSignal, eff_2d: ArrayND<f64>, tau_calib: ArrayND<f64>) -> Self { Self { source, eff_2d, tau_calib } }
}


//Константы
const DT: f64 = 2500.0; //Основное временное разрешение в нс
const WT: f64 = 0.86;   //Прозрачность входного окна
const LT:f64 = 0.93;    //Прозрачность линзы
//a = 1;      %Индекс начала отрезка
//b = 100;     %Индекс конца отрезка
const PIX_FOV:f64 = (2.88/160.0)*(2.88/160.0);  //Поле зрения пикселя в стеррадианах
const S:f64 = PI*25.0/4.0; //Площадь входного окна в см^2
//N_ch = 256;  %Число каналов в расчет lightcurvesum_global_cor
//N=240400;
const T:f64 = 0.001;  //Time sample в секундах
const NTS:f64 = T/DT*1e9;
const CR_TO_INT:f64 = 1.0/(WT*LT*PIX_FOV*S*T);

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
        let tgt:Arc<Mutex<ArrayND<f64>>> = Arc::new(Mutex::new(ArrayND::new(src_data.shape.clone().into(), 0.0)));

        let enumerator = src_data.enumerate();

        let tau_calib = &self.tau_calib;
        let eff_2d = &self.eff_2d;

        enumerator.par_bridge().for_each(|i|{
            let mut pixel_index = i.clone();
            pixel_index.drain(0..1);
            let x =src_data[&i];
            let eff = eff_2d[&pixel_index];
            let tau = tau_calib[&pixel_index];
            let b = tau*eff/DT;
            let v = if eff>0.0 && tau>0.0{
                -CR_TO_INT*NTS*lambertw0(-b*x/NTS/eff)
            }
            else{0.0};

            tgt.lock().unwrap()[&i] = v;
        });
        let tgt = Arc::try_unwrap(tgt).unwrap();
        tgt.into_inner().unwrap()
    }
}
