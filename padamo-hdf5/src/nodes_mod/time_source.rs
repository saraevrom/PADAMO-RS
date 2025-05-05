use abi_stable::std_types::RString;
use padamo_api::prelude::*;
use padamo_api::{ports, constants};
use abi_stable::std_types::RVec;
use crate::ops::LazyTimeHDF5Reader;

#[derive(Debug,Clone)]
pub struct LazyHDF5TimeNode;

impl LazyHDF5TimeNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let filename = args.inputs.request_string("Filename")?;
        //let spatial = super::make_spatial()
        let temporal = args.constants.request_string("Temporal")?.into();
        //let spatial_reader = LazyHDF5Reader3D::<f64>::new(filename.clone().into(), spatial);
        let temporal_reader = LazyTimeHDF5Reader::<f64>::new(filename.into(), temporal);
        match temporal_reader{
            Ok(tmp)=>args.outputs.set_value("Time", Content::DetectorTime(make_lao_box(tmp))),
            Err(tmp)=>Err(ExecutionError::OtherError(format!("HDF error (temporal): {}",tmp).into())),
        }

    }
}

impl CalculationNode for LazyHDF5TimeNode{
    fn name(&self,) -> RString where {
        "HDF5 Time node".into()
    }

    fn category(&self,) -> abi_stable::std_types::RVec<abi_stable::std_types::RString>where {
        padamo_api::common_categories::time_sources()
    }


    fn identifier(&self,) -> RString where {
        "padamohdf5.time_reader".into()
    }

    fn inputs(&self) -> RVec<CalculationIO>{
        ports!(
            ("Filename", ContentType::String)
        )
    }

    fn outputs(&self) -> RVec<CalculationIO> {
        ports!(
            ("Time", ContentType::DetectorTime)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!(
            ("Temporal", "Time field", "unixtime_dbl_global")
        )
    }

    fn calculate(&self, args:CalculationNodeArguments) -> abi_stable::std_types::RResult<(),ExecutionError> {
        self.calculate(args).into()
    }

}
