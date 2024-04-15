use abi_stable::rvec;
use abi_stable::std_types::{RResult, RString, RVec};
use padamo_api::function_operator::{DoubleFunctionOperator,DoubleFunctionOperatorBox, make_function_box};
use padamo_api::{constants, ports, prelude::*};

fn category() -> RVec<RString>where {
    rvec!["Functions".into(), "LC".into()]
}


#[derive(Clone,Debug)]
pub struct LCSwitch{
    pub left:DoubleFunctionOperatorBox,
    pub right:DoubleFunctionOperatorBox
}

impl DoubleFunctionOperator for LCSwitch{
    #[allow(clippy::let_and_return)]
    fn calculate(&self,x:f64,) -> f64 where {
       if x<0.0{
            self.left.calculate(x)
        }
        else{
            self.right.calculate(x)
        }
    }
}

#[derive(Clone,Debug)]
pub struct LCSwitchNode;

impl LCSwitchNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        let mut f_left = inputs.request_function("Left")?;
        let mut f_right = inputs.request_function("Right")?;
        let ampl = constants.request_float("amplitude")?;

        if !constants.request_boolean("left_ascending")?{
            f_left = f_left.invmap(|x| -x);
        }

        if constants.request_boolean("right_descending")?{
            f_right = f_right.invmap(|x| -x);
        }

        let combined = make_function_box(LCSwitch{left:f_left, right:f_right});
        let combined = combined.map(move |x| x*ampl);

        outputs.set_value("LC", combined.into())?;
        Ok(())
    }
}

impl CalculationNode for LCSwitchNode{
    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString {
        "LC centering node".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self,) -> RVec<CalculationIO>{
        ports![
            ("Left", ContentType::Function),
            ("Right", ContentType::Function)
        ]
    }

    fn category(&self,) -> RVec<RString>where {
        category()
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
            ("amplitude", 1.0),
            ("left_ascending", true),
            ("right_descending", true)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}


#[derive(Clone,Debug)]
pub struct LinearLCNode;

impl LinearLCNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        let tau = constants.request_float("tau")?;
        if tau==0.0{
            return Err(ExecutionError::OtherError("Linear LC tau must be nonzero".into()));
        }
        let output:DoubleFunctionOperatorBox = (move |x| x/tau).into();
        outputs.set_value("LC", output.into())?;
        Ok(())
    }
}


impl CalculationNode for LinearLCNode{
    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString {
        "Linear LC".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self,) -> RVec<CalculationIO>{
        ports![]
    }

    fn category(&self,) -> RVec<RString>where {
        category()
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
            ("tau", 1.0)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}



#[derive(Clone,Debug)]
pub struct ExponentLCNode;

impl ExponentLCNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        let tau = constants.request_float("tau")?;
        if tau==0.0{
            return Err(ExecutionError::OtherError("Exponent LC tau must not be zero".into()));
        }
        let output:DoubleFunctionOperatorBox = (move |x:f64| (x/tau).exp()).into();
        outputs.set_value("LC", output.into())?;
        Ok(())
    }
}


impl CalculationNode for ExponentLCNode{
    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString {
        "Exponent LC".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self,) -> RVec<CalculationIO>{
        ports![]
    }

    fn category(&self,) -> RVec<RString>where {
        category()
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
            ("tau", 1.0)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}
