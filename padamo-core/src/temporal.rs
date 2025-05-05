use padamo_api::{constants, lazy_array_operations::LazyTriSignal, nodes_vec, ports, prelude::*};
use abi_stable::{rvec, std_types::{ROption::RNone, RResult, RString, RVec}};



#[derive(Clone,Debug)]
pub struct CombineSpacetime;


fn merge_spacetime(args:&mut CalculationNodeArguments)->Result<(),ExecutionError>{
    let signal = args.inputs.request_detectorsignal("Signal")?;
    let time = args.inputs.request_detectortime("Time")?;
    let res:LazyTriSignal = (signal, time, RNone).into();
    args.outputs.set_value("Combined signal", res.into())
}

impl CalculationNode for CombineSpacetime{
    fn name(&self,) -> RString where {
        "Combine signal and time".into()
    }

    fn category(&self,) -> RVec<RString>where {
        rvec!["Signal manipulation".into()]
    }

    fn identifier(&self,) -> RString where {
        "padamocore.combinespacetime".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Signal", ContentType::DetectorSignal),
            ("Time", ContentType::DetectorTime)
        )
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Combined signal", ContentType::DetectorFullData)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!()
    }

    fn calculate(&self,args:CalculationNodeArguments,) -> RResult<(),ExecutionError>where {
        merge_spacetime(&mut args).into()
    }
}

pub fn nodes()->RVec<CalculationNodeBox>{
    nodes_vec![

        //StringReplaceRegexNode
    ]
}
