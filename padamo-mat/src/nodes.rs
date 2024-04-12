use abi_stable::{rvec, std_types::{RResult, RString, RVec}};
use padamo_api::{constants, ports, prelude::*};
use crate::ops::{self, ConstantArray};

#[derive(Clone,Debug)]
pub struct MatReadNode;

impl MatReadNode{
    pub fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>{
        let filename = inputs.request_string("Filename")?.to_string();
        let field = constants.request_string("field")?.to_string();
        let file = std::fs::File::open(&filename).map_err(|e| ExecutionError::OtherError(format!("{}",e).into()))?;
        let mat_file = matfile::MatFile::parse(file).map_err(|e| ExecutionError::OtherError(format!("{}",e).into()))?;
        if let Some(d) = mat_file.find_by_name(&field){

            let data:ndarray::ArrayD<f64> = d.try_into().map_err(|e| ExecutionError::OtherError(format!("{}",e).into()))?;
            let data:padamo_api::lazy_array_operations::ArrayND<f64> = data.into();

            let data:ConstantArray<f64> = ops::ConstantArray::new(data);
            outputs.set_value("Array", Content::DetectorSignal(make_lao_box(data)))?;
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
    fn name(&self,) -> RString where {
        "MAT file matrix".into()
    }

    fn category(&self,) -> RVec<RString>where {
        rvec!["MAT".into()]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Filename", ContentType::String)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Output definition of node"]
    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Array", ContentType::DetectorSignal)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Constants definition of node with default values."]
    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![
            ("field", "data")
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> RResult<(),ExecutionError> {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}
