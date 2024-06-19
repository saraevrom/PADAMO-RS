use std::hash::{DefaultHasher,Hasher};
use abi_stable::StableAbi;

#[repr(C)]
#[derive(StableAbi,Clone,Copy,Debug)]
pub struct RandomState{
    pub current_seed: u64,
    pub current_state:u64,
}

pub fn get_byte(value:u64, offset:u64)->String{
    let byte = ((value>>offset)& 0xFF) as u8;
    let mapping = "0123456789ABCDEF";
    format!("{}{}",mapping.chars().nth((byte/16) as usize).unwrap(),mapping.chars().nth((byte%16) as usize).unwrap())
}

pub fn bytify(value:u64)->String{
    (0..8).map(|i| get_byte(value, i)).collect()
}

impl RandomState{
    pub fn new(seed:u64)->Self{
        Self { current_seed: seed, current_state: seed }
    }

    pub fn reset(&mut self){
        self.current_state = self.current_seed;
    }


    pub fn separate(&self,mutator:u64)->Self{
        let state = self.current_state;
        let mut hasher = DefaultHasher::new();
        hasher.write_u64(state);
        hasher.write_u64(mutator);
        Self::new(hasher.finish())
    }

    /// Generate new u64 value
    pub fn generate_new(&mut self)->u64{
        let mut hasher = DefaultHasher::new();
        hasher.write_u64(self.current_state);
        let hashed = hasher.finish();
        self.current_state = hashed;
        hashed
    }

    ///Generate uniformly distributed value in [0..1]
    pub fn generate_uniform_normalized(&mut self)->f64{
        let norm = u64::MAX as f64;
        (self.generate_new() as f64)/norm
    }

    ///Generate uniformly distributed value in [a..b]
    pub fn generate_uniform(&mut self,a:f64,b:f64)->f64{
        self.generate_uniform_normalized()*(b-a)+a
    }

    pub fn generate_uuid(&mut self)->String{
        let a = self.generate_new();
        let b = self.generate_new();
        let mut a = bytify(a);
        a.insert(8, '-');
        a.insert(13, '-');
        let mut b = bytify(b);
        b.insert(4, '-');
        format!("{}-{}",a,b)
    }
}
