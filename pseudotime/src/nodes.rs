use abi_stable::{rvec, std_types::{RResult, RString, RVec}};
use padamo_api::{constants, ports, prelude::*};


#[derive(Clone,Debug)]
pub struct PseudoTime;

impl PseudoTime{
    fn calculate(&self,args:CalculationNodeArguments,) -> Result<(),ExecutionError>{
        let start = args.constants.request_string("tmpbase")?;
        let length = args.constants.request_integer("tmpcount")?;
        if length<0{
            return Err(ExecutionError::OtherError("time length must be nonnegative".into()));
        }
        let length = length as usize;
        let resolution = args.constants.request_float("tmpres")?;

        let time = crate::ops::AddTime::new(length, resolution, &start);
        if let Some(t) = time{
            args.outputs.set_value("Time", make_lao_box(t).into())
        }
        else{
            Err(ExecutionError::OtherError("Could not create pseudotime (check start datetime format)".into()))
        }
    }
}

impl CalculationNode for PseudoTime {
    fn name(&self) -> RString where {
        "Pseudo time".into()
    }

    fn category(&self) -> RVec<RString>{
        padamo_api::common_categories::time_sources()
    }

    fn identifier(&self) -> RString{
        "pseudotime.pseudotime".into()
    }

    fn inputs(&self) -> RVec<CalculationIO>{
        ports!()
    }

    fn outputs(&self) -> RVec<CalculationIO>{
        ports!(
            ("Time", ContentType::DetectorTime)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        let now = chrono::Utc::now();
        let dat = now.format("%Y-%m-%d %H:%M:%S.0");
        let formatted = format!("{}", dat);
        constants!(
            ("tmpbase","Start datetime", formatted),
            ("tmpres","Temporal resolution", 0.000256),
            ("tmpcount","Length", 100),
        )
    }

    fn calculate(&self,args:CalculationNodeArguments,) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}
