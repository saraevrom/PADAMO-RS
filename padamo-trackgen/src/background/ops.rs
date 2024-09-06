use std::f64::consts::PI;
use std::hash::DefaultHasher;
use std::hash::Hasher;

use abi_stable::std_types::RVec;
use padamo_api::lazy_array_operations::ArrayND;
use padamo_api::lazy_array_operations::LazyArrayOperation;
use padamo_api::lazy_array_operations::LazyDetectorSignal;
use statrs::distribution::ContinuousCDF;

use statrs::distribution::Normal;



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
