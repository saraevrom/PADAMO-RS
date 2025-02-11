use abi_stable::{rvec, std_types::{RResult, RString, RVec}};
use padamo_api::{constants, nodes_vec, ports, prelude::*};
use crate::ops::FilterOp;

#[derive(Clone,Debug)]
pub struct STFTNode;

impl STFTNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let mut signal_in = args.inputs.request_detectorfulldata("Signal")?;
        let filter = args.inputs.request_function("Filter")?;
        let window = args.constants.request_integer("window")?;
        if window<0{
            return Err(ExecutionError::OtherError("STFT window must not be negative".into()));
        }
        if window as usize > signal_in.0.length(){
            return Err(ExecutionError::OtherError("STFT window is bigger than signal".into()));
        }
        let window = window as usize;
        let time_length:usize = signal_in.1.length();
        let time_length = time_length.min(1000);
        let time_test:Vec<f64> = signal_in.1.request_range(0,time_length).to_vec();
        let sample_period:f64 = time_test.windows(2).map(|vs| {
            let [x, y] = vs else { unreachable!() };
            y - x
        }).fold(0.0, |a,b| a+b)/(time_length as f64);
        let sample_rate = 1.0/sample_period;
        signal_in.0 = make_lao_box(FilterOp::new(signal_in.0, filter, window, sample_rate));
        args.outputs.set_value("Signal", signal_in.into())?;
        Ok(())
    }
}

impl CalculationNode for STFTNode{
    fn name(&self,) -> RString where {
        "STFT filter".into()
    }

    fn category(&self,) -> RVec<RString>where {
        rvec!["Data Processing".into()]
    }

    fn identifier(&self,) -> RString where {
        "padamosfft.stft_filter".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Signal", ContentType::DetectorFullData),
            ("Filter", ContentType::Function)
        ]
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Signal", ContentType::DetectorFullData),
        ]
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![
            ("window", 100)
        ]
    }

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}


pub fn nodes()->RVec<CalculationNodeBox>{
    nodes_vec![
        STFTNode
    ]
}
