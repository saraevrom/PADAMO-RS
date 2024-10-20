use std::{collections::VecDeque, f64::consts::PI, fmt::Debug, sync::{Arc, Mutex}, thread};

use padamo_api::lazy_array_operations::ArrayND;
use rustfft::Fft;
use num::Complex;

pub fn window_func(n:i64, n_full:i64)->f64{
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


fn free_threads(threads: &mut VecDeque<thread::JoinHandle<()>>, threadcount:usize){
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

#[derive(Clone,Copy)]
pub struct STFTConverter{
    pub window: usize,
}

impl Debug for STFTConverter{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"STFTConverter(window={})",self.window)
    }
}


impl STFTConverter{
    pub fn new(window:usize)->Self{
        //let fft = planner.plan_fft_forward(window);
        Self { window }
    }

//     pub fn print_frequencies(&self, sampling_rate:f64){
//
//     }

    fn stft(&self, signal:&[f64], plan:Arc<dyn Fft<f64>>, window_number:usize)->Vec<Complex<f64>>{
        let n_full = self.window as i64;
        let window_start = window_number*(self.window/2);
        let mut s: Vec<Complex<f64>> = (window_start..window_start+self.window).map(|n| Complex{re:signal[n]*window_func(n as i64 - window_start as i64, n_full ), im:0.0}).collect();
        plan.process(&mut s);
        s
    }

    fn istft(&self, spectrum:Vec<Complex<f64>>, plan:Arc<dyn Fft<f64>>, window_number:usize, target_array:&mut Vec<f64>){

        let window = spectrum.len();
        // let m = window_number as i64;
        let n_full = self.window as i64;
        let window_start = window_number*(window)/2;

        let mut data = spectrum;
        plan.process(&mut data);
        for n in window_start..window_start+window{
            target_array[n] += (data[n-window_start]*window_func(n as i64-window_start as i64, n_full)).re/(window as f64);
        }


    }

    pub fn filter(&self, signal:&[f64], filter: &padamo_api::function_operator::DoubleFunctionOperatorBox, sampling_rate:f64)->Vec<f64>{
        let mut planner:rustfft::FftPlanner<f64> = rustfft::FftPlanner::new();
        let frequencies = frequencies(self.window, sampling_rate);
        let h_mod:Vec<f64> = frequencies.iter().map(|x| filter.calculate(*x)).collect();


        // let mut spectra = self.stft(signal, &mut planner);
        // for i in 0..spectra.shape()[0]{
        //     spectra.row_mut(i).iter_mut().enumerate().for_each(|(j,x)| {*x = *x * modifiers[j]});
        // }
        // self.istft(&spectra, &mut planner)
        let signal_len = signal.len();
        let windows_amount = (signal_len-self.window)/(self.window/2)+1;

        let mut final_signal = vec![0.0; signal_len];

        let fft_plan = planner.plan_fft_forward(self.window);
        let ifft_plan = planner.plan_fft_inverse(self.window);
        for m in 0..windows_amount{
            let mut spectrum = self.stft(signal, fft_plan.clone(), m);
            spectrum.iter_mut().enumerate().for_each(|(i,x)|{
                *x = *x * h_mod[i];
            });
            self.istft(spectrum, ifft_plan.clone(), m, &mut final_signal);
        }
        final_signal
    }

    pub fn filter_arrays(self, signal:ArrayND<f64>, filter: padamo_api::function_operator::DoubleFunctionOperatorBox, sampling_rate:f64)->ArrayND<f64>{
        let shape = signal.shape.clone().to_vec();
        if shape[0]<self.window{
            panic!("Cannot use stft with array smaller than window");
        }
        let frame_size:usize = shape.iter().skip(1).fold(1, |a,b| a*b);
        let target_shape = shape.clone();
        let target_length = target_shape[0];
        let target = Arc::new(Mutex::new(ArrayND::<f64>::new(target_shape.into(),-666.0)));
        let source = Arc::new(signal);
        let filter = Arc::new(filter);

        let threadcount = num_cpus::get();
        let mut threads:VecDeque<thread::JoinHandle<()>> = VecDeque::with_capacity(10);

        for pixel in 0..frame_size{
            free_threads(&mut threads, threadcount);
            let tgt = target.clone();
            let src = source.clone();
            let filter_cloned = filter.clone();
            let offset = pixel;
            let handle = thread::spawn(move || {
                let mut signal_in_pixel = Vec::with_capacity(target_length);
                for i in 0..target_length{
                    let index = i*frame_size+offset;
                    let value = src.flat_data[index];
                    signal_in_pixel.push(value);
                }
                let signal_data = self.filter(&signal_in_pixel, &filter_cloned, sampling_rate);
                for i in 0..target_length{
                    //tgt.lock().unwrap().flat_data[i*frame_size+offset] = roller.median();
                    tgt.lock().unwrap().flat_data[i*frame_size+offset] = signal_data[i];
                }

            });
            threads.push_back(handle);


        }

        free_threads(&mut threads, 1);
        let lock = Arc::try_unwrap(target).unwrap();
        lock.into_inner().unwrap()
    }
}

