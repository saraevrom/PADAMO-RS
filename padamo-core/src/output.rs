use abi_stable::rvec;
use abi_stable::std_types::{RString, RVec, RResult};
use padamo_api::{prelude::*, constants};
use padamo_api::{ports,nodes_vec};
use crate::TD_Opaque;

#[derive(Debug,Clone)]
pub struct Printer(pub ContentType);

impl Printer{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>{
        let value = inputs.request_type(&self.0, "Value")?;
        println!("Value: {:?}", value);
        Ok(())
    }
}

impl CalculationNode for Printer{
    fn name(&self,) -> RString{
        format!("Print {:?}", self.0).into()
    }

    fn category(&self,) -> RVec<RString> {
        rvec![
            "IO".into()
        ]
    }

    fn is_primary(&self,) -> bool {
        true
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Value", self.0)
        )
    }

    fn outputs(&self,) -> RVec<CalculationIO>{
        RVec::new()
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!()
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> RResult<(),ExecutionError>{
        self.calculate(inputs, outputs, constants, environment).into()
    }
}

pub fn nodes()->RVec<CalculationNodeBox>{
    nodes_vec![
        Printer(ContentType::Integer),
        Printer(ContentType::Float),
        Printer(ContentType::String),
        Printer(ContentType::Boolean)
    ]

}
