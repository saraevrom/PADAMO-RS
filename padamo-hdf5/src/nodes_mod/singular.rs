use abi_stable::rvec;
use abi_stable::std_types::ROption::RSome;
use abi_stable::std_types::{RString,ROption};
use padamo_api::prelude::*;
use padamo_api::{ports, constants};
use abi_stable::std_types::RVec;
use crate::ops::LazyTimeHDF5Reader;
use padamo_api::lazy_array_operations::{LazyTriSignal,LazyTimeSignal};
use abi_stable::sabi_trait::prelude::TD_Opaque;

#[derive(Debug,Clone)]
pub struct LazyHDF5SignalNode;

impl LazyHDF5SignalNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let filename = args.inputs.request_string("Filename")?;
        let spatial = args.constants.request_string("Spatial")?.into();
        //let spatial = super::make_spatial()
        let temporal = args.constants.request_string("Temporal")?.into();
        //let spatial_reader = LazyHDF5Reader3D::<f64>::new(filename.clone().into(), spatial);
        let spatial_reader = super::make_spatial(filename.clone().into(), spatial);
        let temporal_reader = LazyTimeHDF5Reader::<f64>::new(filename.into(), temporal);
        match (spatial_reader,temporal_reader){
            (Ok(sp),Ok(tmp))=>{
                let signal:LazyTriSignal = (sp,LazyTimeSignal::from_value(tmp,TD_Opaque) ,ROption::RNone).into();
                //let signal:LazyTriSignal = (LazyDetectorSignal::from_value(sp,TD_Opaque),LazyTimeSignal::from_value(tmp,TD_Opaque) ,ROption::RNone).into();
                args.outputs.set_value("Signal", Content::DetectorFullData(signal))
            },
            (Err(sp),Err(tmp))=>{
                Err(ExecutionError::OtherError(format!("HDF error (spatiotemporal): {}; {}",sp,tmp).into()))
            },
            (Ok(_),Err(tmp))=>{
                Err(ExecutionError::OtherError(format!("HDF error (temporal): {}",tmp).into()))
            },
            (Err(sp),Ok(_))=>{
                Err(ExecutionError::OtherError(format!("HDF error (spatial): {}",sp).into()))
            }
        }

    }
}

impl CalculationNode for LazyHDF5SignalNode{
    fn name(&self,) -> RString where {
        "Lazy HDF5 Signal node".into()
    }

    fn category(&self,) -> abi_stable::std_types::RVec<abi_stable::std_types::RString>where {
        rvec![
            "HDF5".into()
        ]
    }

    fn old_identifier(&self,) -> ROption<RString>where {
        RSome("HDF5/Lazy HDF5 Signal node".into())
    }

    fn identifier(&self,) -> RString where {
        "padamohdf5.file_reader".into()
    }

    fn inputs(&self) -> RVec<CalculationIO>{
        ports!(
            ("Filename", ContentType::String)
        )
    }

    fn outputs(&self) -> RVec<CalculationIO> {
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!(
            ("Spatial", "pdm_2d_rot_global"),
            ("Temporal", "unixtime_dbl_global")
        )
    }

    fn calculate(&self, args:CalculationNodeArguments) -> abi_stable::std_types::RResult<(),ExecutionError> {
        self.calculate(args).into()
    }

}
