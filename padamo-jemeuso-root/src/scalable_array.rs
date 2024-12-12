use ndarray::ArrayBase;
use ndarray::IxDyn;
use oxyroot::Unmarshaler;
use oxyroot::RBuffer;
use ndarray::Array;

use {
    once_cell::sync::Lazy,
    regex::Regex,
};

#[derive(Copy,Clone,Debug)]
enum FloatType{
    Single,
    Double
}

impl FloatType{
    pub fn name_len(&self)->usize{
        match self{
            Self::Single=>5,
            Self::Double=>6,
        }
    }
}


#[derive(Clone,Debug)]
struct Sizes{
    pub type_name:FloatType,
    pub shape: Vec<usize>
}


impl Sizes{
    pub fn new(input:&str)->Result<Self,oxyroot::rbytes::Error>{
        let mut workon = input.trim();
        // println!("workon {}",workon);
        let type_name = if workon.starts_with("float"){
            FloatType::Single
        }
        else if workon.starts_with("double"){
            FloatType::Double
        }
        else{
            return Err(oxyroot::rbytes::Error::Misc(format!("Unsupported type of {}", input)));
        };

        workon = workon.split_at(type_name.name_len()).1;
        workon = workon.trim();
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[(\d+)\]").unwrap());
        let shape:Vec<usize> = RE.captures_iter(workon).map(|x| {
            let v = x.get(1).unwrap().as_str();
            //println!("{}",v);
            v.parse().unwrap()

        }).collect();
        Ok(Self { type_name, shape })
    }

    pub fn flat_length(&self)->usize{
        self.shape.iter().fold(1, |a,b| a* *b)
    }
}

#[derive(Default)]
pub struct NDArrayRootWrapper{
    pub data:Array<f64,ndarray::IxDyn>,
}

impl Unmarshaler for NDArrayRootWrapper{
    fn unmarshal(&mut self, r: &mut RBuffer) -> Result<(), oxyroot::rbytes::error::Error>{
        Err(oxyroot::rbytes::Error::Misc("Name required".into()))
    }

    fn unmarshal_named(&mut self, r: &mut RBuffer, actual_name:&str) -> oxyroot::rbytes::Result<()> {
        let sizes = Sizes::new(actual_name)?;
        let n = sizes.flat_length();
        let mut flat_data:Vec<f64> = Vec::with_capacity(n);
        for _ in 0..n{
            let new_element = match sizes.type_name{
                FloatType::Double => r.read_object_into::<f64>()?,
                FloatType::Single => r.read_object_into::<f32>()? as f64
            };
            flat_data.push(new_element);
        }
        let shape = IxDyn(&sizes.shape);
        self.data = ndarray::ArrayBase::from_shape_vec(shape, flat_data).map_err(|e| oxyroot::rbytes::Error::Misc(format!("{}",e)))?;
        // let flat_data:Vec<f64> = (0..n).map(|i| {
        //     match sizes.type_name{
        //         FloatType::Double => r.read_object_into::<f64>(),
        //         FloatType::Single => r.read_object_into::<f32>() as f64
        //     }
        // }).collect();
        Ok(())
    }
}
