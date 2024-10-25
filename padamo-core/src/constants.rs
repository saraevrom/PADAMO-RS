use abi_stable::rvec;
use abi_stable::std_types::ROption::RSome;
use abi_stable::std_types::{RString, RVec, RResult};
use padamo_api::{prelude::*, constants, nodes_vec};
use padamo_api::ports;

#[derive(Debug,Clone)]
pub struct Constant(pub ConstantContentType);

impl Constant{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>{
        let value = constants.request_type(&self.0, "Value")?;
        outputs.set_value("Value", value.into())
    }
}

impl CalculationNode for Constant{
    fn name(&self,) -> RString{
        format!("Constant {:?}", self.0).into()
    }

    fn category(&self,) -> RVec<RString> {
        rvec![
            "Constants".into()
        ]
    }

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
        RSome(format!("Constants/Constant {:?}", self.0).into())
    }

    fn identifier(&self,) -> RString where {
        let idmark = format!("{:?}",self.0).to_lowercase();
        format!("padamocore.constant.{}",idmark).into()
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Value", self.0.into())
        )
    }

    fn inputs(&self,) -> RVec<CalculationIO>{
        RVec::new()
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!(
            ("Value", self.0.default_constant())
        )
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError>{
        self.calculate(inputs, outputs, constants, environment).into()
    }
}

pub fn nodes()->RVec<CalculationNodeBox>{
    nodes_vec![
        Constant(ConstantContentType::Integer),
        Constant(ConstantContentType::Float),
        Constant(ConstantContentType::String),
        Constant(ConstantContentType::Boolean)
    ]
}
