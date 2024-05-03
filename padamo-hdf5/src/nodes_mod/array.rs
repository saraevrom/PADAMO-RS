use abi_stable::rvec;
use abi_stable::std_types::{RString,RArc,ROption};
use padamo_api::prelude::*;
use padamo_api::{ports, constants};
use abi_stable::std_types::RVec;
use crate::ops::{LazyHDF5Reader3D,LazyTimeHDF5Reader};
use padamo_api::lazy_array_operations::{LazyTriSignal,LazyArrayOperationBox,LazyDetectorSignal};
use abi_stable::sabi_trait::prelude::TD_Opaque;


#[derive(Debug,Clone)]
pub struct LazyHDF5ArrayNode;

impl LazyHDF5ArrayNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>{
        let filename = inputs.request_string("Filename")?;
        let spatial = constants.request_string("Field")?.into();
        //let spatial = super::make_spatial()
        //let spatial_reader = LazyHDF5Reader3D::<f64>::new(filename.clone().into(), spatial);
        let spatial_reader = super::make_spatial(filename.clone().into(), spatial).map_err(|e| ExecutionError::OtherError(format!("HDF error : {}",e).into()))?;
        outputs.set_value("Array", spatial_reader.into())
    }
}

impl CalculationNode for LazyHDF5ArrayNode{
    fn name(&self,) -> RString where {
        "Lazy HDF5 Array reader".into()
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
            ("Array", ContentType::DetectorSignal)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!(
            ("Field", "some_field")
        )
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> abi_stable::std_types::RResult<(),ExecutionError> {
        self.calculate(inputs, outputs, constants, environment).into()
    }

}
