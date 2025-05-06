use abi_stable::std_types::{ROption::RSome, RResult, RString, RVec};
use padamo_api::{constants, ports, prelude::*};
use crate::ops::{self, ConstantArray, ConstantVec};

#[derive(Clone,Debug)]
pub struct MatReadNode;

impl MatReadNode{
    pub fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let filename = args.inputs.request_string("Filename")?.to_string();
        let field = args.constants.request_string("field")?.to_string();
        let file = std::fs::File::open(&filename).map_err(|e| ExecutionError::OtherError(format!("{}",e).into()))?;
        let mat_file = matfile::MatFile::parse(file).map_err(|e| ExecutionError::OtherError(format!("{}",e).into()))?;
        if let Some(d) = mat_file.find_by_name(&field){

            let data:ndarray::ArrayD<f64> = d.try_into().map_err(|e| ExecutionError::OtherError(format!("{}",e).into()))?;
            let mut data:padamo_api::lazy_array_operations::ArrayND<f64> = data.into();//crate::compat::ndarray_to_arraynd(data);

            if args.constants.request_boolean("flip")?{
                data = data.flip_indices();
            }

            let data:ConstantArray<f64> = ops::ConstantArray::new(data);
            args.outputs.set_value("Array", Content::DetectorSignal(make_lao_box(data)))?;
            Ok(())
        }
        else{
            Err(ExecutionError::OtherError(format!("Field {} is not found", field).into()))
        }
    }
}

impl CalculationNode for MatReadNode{
    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString {
        "MAT file matrix".into()
    }

    fn category(&self,) -> RVec<RString>{
        padamo_api::common_categories::array_sources()
    }

    fn identifier(&self,) -> RString {
        "padamomat.mat_reader".into()
    }

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>{
        RSome("MAT/MAT file matrix".into())
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self,) -> RVec<CalculationIO>{
        ports![
            ("Filename", ContentType::String)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Output definition of node"]
    fn outputs(&self,) -> RVec<CalculationIO>{
        ports![
            ("Array", ContentType::DetectorSignal)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Constants definition of node with default values."]
    fn constants(&self,) -> RVec<CalculationConstant>{
        constants![
            ("field", "data"),
            ("flip", false)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError> {
        self.calculate(args).into()
    }
}

#[derive(Clone,Debug)]
pub struct MatReadTimeNode;

impl MatReadTimeNode{
    pub fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let filename = args.inputs.request_string("Filename")?.to_string();
        let field = args.constants.request_string("field")?.to_string();
        let file = std::fs::File::open(&filename).map_err(|e| ExecutionError::OtherError(format!("{}",e).into()))?;
        let mat_file = matfile::MatFile::parse(file).map_err(|e| ExecutionError::OtherError(format!("{}",e).into()))?;
        if let Some(d) = mat_file.find_by_name(&field){

            let data:ndarray::ArrayD<f64> = d.try_into().map_err(|e| ExecutionError::OtherError(format!("{}",e).into()))?;
            let data:padamo_api::lazy_array_operations::ArrayND<f64> = data.into();
            let data = data.squeeze();
            if data.shape.len()!=1{
                return Err(ExecutionError::OtherError("MAT data time length is wrong".into()));
            }

            let data:ConstantVec<f64> = ops::ConstantVec::new(data.flat_data);
            args.outputs.set_value("Time", Content::DetectorTime(make_lao_box(data)))?;
            Ok(())
        }
        else{
            Err(ExecutionError::OtherError(format!("Field {} is not found", field).into()))
        }
    }
}

impl CalculationNode for MatReadTimeNode{
    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString {
        "MAT file time".into()
    }

    fn category(&self,) -> RVec<RString>{
        padamo_api::common_categories::time_sources()
    }

    fn identifier(&self,) -> RString {
        "padamomat.mat_time_reader".into()
    }


    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self,) -> RVec<CalculationIO>{
        ports![
            ("Filename", ContentType::String)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Output definition of node"]
    fn outputs(&self,) -> RVec<CalculationIO>{
        ports![
            ("Time", ContentType::DetectorTime)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Constants definition of node with default values."]
    fn constants(&self,) -> RVec<CalculationConstant>{
        constants![
            ("field", "data")
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError> {
        self.calculate(args).into()
    }
}
