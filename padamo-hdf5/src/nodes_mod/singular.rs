use abi_stable::rvec;
use abi_stable::std_types::{RString,RArc,ROption};
use padamo_api::prelude::*;
use padamo_api::{ports, constants};
use abi_stable::std_types::RVec;
use crate::ops::{LazyHDF5Reader3D,LazyTimeHDF5Reader};
use padamo_api::lazy_array_operations::{LazyTriSignal,LazyArrayOperationBox,LazyDetectorSignal,LazyTimeSignal};
use abi_stable::sabi_trait::prelude::TD_Opaque;

#[derive(Debug,Clone)]
pub struct LazyHDF5SignalNode;

impl LazyHDF5SignalNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>{
        let filename = inputs.request_string("Filename")?;
        let spatial = constants.request_string("Spatial")?.into();
        //let spatial = super::make_spatial()
        let temporal = constants.request_string("Temporal")?.into();
        //let spatial_reader = LazyHDF5Reader3D::<f64>::new(filename.clone().into(), spatial);
        let spatial_reader = super::make_spatial(filename.clone().into(), spatial);
        let temporal_reader = LazyTimeHDF5Reader::<f64>::new(filename.into(), temporal);
        match (spatial_reader,temporal_reader){
            (Ok(sp),Ok(tmp))=>{
                let signal:LazyTriSignal = (sp,LazyTimeSignal::from_value(tmp,TD_Opaque) ,ROption::RNone).into();
                //let signal:LazyTriSignal = (LazyDetectorSignal::from_value(sp,TD_Opaque),LazyTimeSignal::from_value(tmp,TD_Opaque) ,ROption::RNone).into();
                outputs.set_value("Signal", Content::DetectorFullData(signal))
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

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> abi_stable::std_types::RResult<(),ExecutionError> {
        self.calculate(inputs, outputs, constants, environment).into()
    }

}
