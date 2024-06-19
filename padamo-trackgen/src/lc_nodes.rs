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
    pub right:DoubleFunctionOperatorBox,
    pub pivot:f64,
}

impl DoubleFunctionOperator for LCSwitch{
    #[allow(clippy::let_and_return)]
    fn calculate(&self,x:f64,) -> f64 where {
       if x<self.pivot{
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

        let combined = make_function_box(LCSwitch{left:f_left, right:f_right, pivot:0.0});
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
pub struct LCPivotNode;

impl LCPivotNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        let f_left = inputs.request_function("Left")?;
        let mut f_right = inputs.request_function("Right")?;
        //let ampl = constants.request_float("amplitude")?;

        let pivot = constants.request_float("pivot")?;

        let k_left = f_left.calculate(pivot);
        let k_right = f_right.calculate(pivot);

        let combined = make_function_box(
            match (k_left==0.0, k_right==0.0) {
                    (true,true)=>{
                        LCSwitch{left:f_left, right:f_right, pivot}
                    }
                    (false,false)=>{
                        let merging_coeff = k_left/k_right;
                        f_right = f_right.map(move |x| x*merging_coeff);
                        LCSwitch{left:f_left, right:f_right, pivot}
                    }
                    _=>{
                        return Err(ExecutionError::OtherError("Cannot make LC pivot point".into()));
                    }
            }
        );
        //let combined = make_function_box(LCSwitch{left:f_left, right:f_right, pivot:0.0});
        //let combined = combined.map(move |x| x*ampl);

        outputs.set_value("LC", combined.into())?;
        Ok(())
    }
}


impl CalculationNode for LCPivotNode{
    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString {
        "LC pivot switch".into()
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
            ("pivot", 0.0)
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
        let tau = inputs.request_float("tau")?;
        if tau==0.0{
            return Err(ExecutionError::OtherError("Linear LC tau must be nonzero".into()));
        }
        let output:DoubleFunctionOperatorBox = (move |x| x/tau+1.0).into();
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
        ports![
            ("tau", ContentType::Float)
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
            //("tau", 1.0)
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
        let tau = inputs.request_float("tau")?;
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
        ports![
            ("tau", ContentType::Float)
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
            //("tau", 1.0)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}

#[derive(Clone,Debug)]
pub struct TerminationLCNode;

impl TerminationLCNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        let output:DoubleFunctionOperatorBox = (|x:f64| if x==0.0 {1.0} else {0.0}).into();
        outputs.set_value("LC", output.into())?;
        Ok(())
    }
}

impl CalculationNode for TerminationLCNode{
    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString {
        "Terminate LC".into()
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
        constants![]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}




#[derive(Clone,Debug)]
pub struct ConstantLCNode;

impl ConstantLCNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        let output:DoubleFunctionOperatorBox = (|_:f64| 1.0).into();
        outputs.set_value("LC", output.into())?;
        Ok(())
    }
}

impl CalculationNode for ConstantLCNode{
    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString {
        "Constant LC".into()
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
        constants![]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}


#[derive(Clone,Debug)]
pub struct MultiplyByFloatNode;

impl MultiplyByFloatNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        let m = inputs.request_float("Multiplier")?;
        let inner = inputs.request_function("LC")?;
        let output = inner.map(move |x| x*m);
        outputs.set_value("LC", output.into())?;
        Ok(())
    }
}

impl CalculationNode for MultiplyByFloatNode{
    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString {
        "Multiply by value".into()
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
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}
