use std::f64::consts::PI;
use padamo_api::lazy_array_operations::ArrayND;

pub fn window(n:usize, n_full:usize)->f64{
    if n<0 || n>=n_full{
        0.0
    }
    else{
        let n = n as f64;
        let n_full = n_full as f64;
        (PI/n_full*(n+0.5)).sin()
    }

}

pub fn frequencies(n:usize, sampling_rate:f64)->Vec<f64>{

    let mut res = Vec::with_capacity(n);
    let switchpoint = if n%2==0 {n/2} else {(n+1)/2};
    let scaler = sampling_rate/(n as f64);
    res.extend((0..switchpoint).map(|x| (x as f64)*scaler));
    res.extend((switchpoint..n).map(|x| ((n-x) as f64)*scaler));
    res
}
/*
pub struct STFTConverter{
    window: usize,
    planner: rustfft::FftPlanner<f64>,
}


impl STFTConverter{
    pub fn new(window:usize)->Self{
        let planner = rustfft::FftPlanner::new();
        //let fft = planner.plan_fft_forward(window);
        Self { window, planner}
    }

    pub fn stft(&mut self, signal:&[f64])->ndarray::Array2<rustfft::num_complex::Complex<f64>>{
        let full_len = signal.len();
        let short_len = full_len - self.window + 1;
        let mut res:ndarray::Array2<rustfft::num_complex::Complex<f64>> = ndarray::Array2::zeros((0,0));
        res.reserve_columns((self.window))))
        let plan = self.planner.plan_fft_forward(self.window);
        for i in 0..short_len{
            let mut data:Vec<_> = (0..self.window).map(|j|rustfft::num_complex::Complex{re:signal[i+j]*window(j, self.window), im:0.0}).collect();
            plan.process(&mut data);
            res.row_mut(i) = ndarray::Array1::from_vec(data);
            //(0..self.window).for_each(|j|res.row_mut(i)[j] = data[j]);
        }
        res
        // todo!()
    }

    pub fn istft(&self, spectra:&ndarray::Array2<rustfft::num_complex::Complex<f64>>)->ArrayND<f64>{
        todo!()
    }
}*/
