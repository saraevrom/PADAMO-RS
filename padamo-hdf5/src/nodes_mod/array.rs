use abi_stable::rvec;
use abi_stable::std_types::ROption::RSome;
use abi_stable::std_types::{RString,ROption};
use padamo_api::prelude::*;
use padamo_api::{ports, constants};
use abi_stable::std_types::RVec;


#[derive(Debug,Clone)]
pub struct LazyHDF5ArrayNode;

impl LazyHDF5ArrayNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let filename = args.inputs.request_string("Filename")?;
        let spatial = args.constants.request_string("Field")?.into();
        //let spatial = super::make_spatial()
        //let spatial_reader = LazyHDF5Reader3D::<f64>::new(filename.clone().into(), spatial);
        let spatial_reader = super::make_spatial(filename.clone().into(), spatial).map_err(|e| ExecutionError::OtherError(format!("HDF error : {}",e).into()))?;
        args.outputs.set_value("Array", spatial_reader.into())
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

    fn old_identifier(&self,) -> ROption<RString>where {
        RSome("HDF5/Lazy HDF5 Array reader".into())
    }

    fn identifier(&self,) -> RString where {
        "padamohdf5.array_reader".into()
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

    fn calculate(&self, args:CalculationNodeArguments) -> abi_stable::std_types::RResult<(),ExecutionError> {
        self.calculate(args).into()
    }

}
