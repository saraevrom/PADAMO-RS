use abi_stable::rvec;
use abi_stable::std_types::ROption::RSome;
use abi_stable::std_types::{ROption, RResult, RString, RVec};
use padamo_api::function_operator::{DoubleFunctionOperatorBox, make_function_box};
use padamo_api::{constants, nodes_vec, ports, prelude::*};

fn category() -> RVec<RString>where {
    rvec!["Legacy".into(), "Functions".into(), "LC".into()]
}


#[derive(Clone,Debug)]
pub struct LinearLCNodeOld;

impl LinearLCNodeOld{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        let tau = inputs.request_float("tau")?;
        if tau==0.0{
            return Err(ExecutionError::OtherError("Linear LC tau must be nonzero".into()));
        }
        let output:DoubleFunctionOperatorBox = (move |x| x/tau+1.0).into();
        outputs.set_value("LC", output.into())?;
        Ok(())
    }
}


impl CalculationNode for LinearLCNodeOld{
    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString {
        "Linear LC (Legacy)".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self,) -> RVec<CalculationIO>{
        ports![
            ("tau", ContentType::Float)
        ]
    }

    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn old_identifier(&self,) -> ROption<RString>where {
        RSome("Functions/LC/Linear LC".into())
    }

    fn identifier(&self,) -> RString where {
        "padamotrackgen.lc.lc_linear".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Output definition of node"]
    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("LC", ContentType::Function)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Constants definition of node with default values."]
    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![
            //("tau", 1.0)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}



#[derive(Clone,Debug)]
pub struct ExponentLCNodeOld;

impl ExponentLCNodeOld{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        let tau = inputs.request_float("tau")?;
        if tau==0.0{
            return Err(ExecutionError::OtherError("Exponent LC tau must not be zero".into()));
        }
        let output:DoubleFunctionOperatorBox = (move |x:f64| (x/tau).exp()).into();
        outputs.set_value("LC", output.into())?;
        Ok(())
    }
}


impl CalculationNode for ExponentLCNodeOld{
    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString {
        "Exponent LC (Legacy)".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self,) -> RVec<CalculationIO>{
        ports![
            ("tau", ContentType::Float)
        ]
    }

    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn old_identifier(&self,) -> ROption<RString>where {
        RSome("Functions/LC/Exponent LC".into())
    }

    fn identifier(&self,) -> RString where {
        "padamotrackgen.lc.lc_exponent".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Output definition of node"]
    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("LC", ContentType::Function)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Constants definition of node with default values."]
    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![
            //("tau", 1.0)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}





#[derive(Clone,Debug)]
pub struct MultiplyByFloatNodeOld;

impl MultiplyByFloatNodeOld{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        let m = inputs.request_float("Multiplier")?;
        let inner = inputs.request_function("LC")?;
        let output = inner.map(move |x| x*m);
        outputs.set_value("LC", output.into())?;
        Ok(())
    }
}

impl CalculationNode for MultiplyByFloatNodeOld{
    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString {
        "Multiply by value (Legacy)".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self,) -> RVec<CalculationIO>{
        ports![
            ("LC", ContentType::Function),
            ("Multiplier", ContentType::Float)
        ]
    }

    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn old_identifier(&self,) -> ROption<RString>where {
        RSome("Functions/LC/Multiply by value".into())
    }

    fn identifier(&self,) -> RString where {
        "padamotrackgen.lc.multiply_by_value".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Output definition of node"]
    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("LC", ContentType::Function)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Constants definition of node with default values."]
    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}

pub fn nodes()->RVec<CalculationNodeBox>{
    nodes_vec!(
        LinearLCNodeOld,
        ExponentLCNodeOld,
        MultiplyByFloatNodeOld,

    )
}
