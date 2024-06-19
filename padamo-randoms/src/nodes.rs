use abi_stable::{rvec, std_types::{RResult, RString, RVec}, traits::IntoReprC};
use padamo_api::{constants, ports, prelude::*};


#[derive(Clone,Debug)]
pub struct UniformRandomNode;

impl UniformRandomNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> Result<(),ExecutionError>where {
        let a = constants.request_float("lower")?;
        let b = constants.request_float("upper")?;
        let v = rng.generate_uniform(a, b);
        outputs.set_value("Value", v.into())?;
        Ok(())
    }
}

fn category() -> RVec<RString> {
    rvec!["Random".into()]
}

impl CalculationNode for UniformRandomNode{
    fn category(&self,) -> RVec<RString>{
        category()
    }

    fn name(&self,) -> RString where {
        "Random uniform".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>{
        ports!()
    }

    fn outputs(&self,) -> RVec<CalculationIO>{
        ports!(
            ("Value", ContentType::Float)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!(
            ("lower",0.0),
            ("upper",1.0)
        )
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment, rng).into()
    }
}



#[derive(Clone,Debug)]
pub struct UUIDRandomNode;

impl UUIDRandomNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> Result<(),ExecutionError>where {

        let v = rng.generate_uuid();
        outputs.set_value("Value", v.into())?;
        Ok(())
    }
}


impl CalculationNode for UUIDRandomNode{
    fn category(&self,) -> RVec<RString>{
        category()
    }

    fn name(&self,) -> RString where {
        "Random UUID".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>{
        ports!()
    }

    fn outputs(&self,) -> RVec<CalculationIO>{
        ports!(
            ("Value", ContentType::String)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!()
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment, rng).into()
    }
}
