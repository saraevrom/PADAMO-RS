use abi_stable::{rvec, std_types::{ROption::RSome, RResult, RString, RVec}};
use padamo_api::{constants, ports, prelude::*};
use padamo_api::function_operator::DoubleFunctionOperatorBox;
use crate::{implement_binary_combinator, implement_onearg_function, implement_unary_combinator};


#[derive(Clone,Debug)]
pub struct FCalculateNode;


impl FCalculateNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer) -> Result<(),ExecutionError>where {
        let f0 = inputs.request_function("F")?;
        let x = inputs.request_float("x")?;
        let y = f0.calculate(x);
        outputs.set_value("y", y.into())?;
        Ok(())
    }
}

#[derive(Clone,Debug)]
pub struct ConstantNode;
impl ConstantNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        let v = inputs.request_float("Value")?;
        //let f = make_function_box(crate::ops::Constant(v));
        let f:DoubleFunctionOperatorBox = (move |_| {v}).into();
        outputs.set_value("F", f.into())?;
        Ok(())
    }
}

#[derive(Clone,Debug)]
pub struct LinearNode;

impl LinearNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        //let v = inputs.request_float("Value")?;
        //let f = make_function_box(crate::ops::Linear);
        let f:DoubleFunctionOperatorBox = (|x| {x}).into();
        outputs.set_value("F", f.into())?;
        Ok(())
    }
}

#[derive(Clone,Debug)]
pub struct SquareNode;

impl SquareNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        //let v = inputs.request_float("Value")?;
        //let f = make_function_box(crate::ops::Square);
        let f:DoubleFunctionOperatorBox = (|x| {x*x}).into();

        outputs.set_value("F", f.into())?;
        Ok(())
    }
}

#[derive(Clone,Debug)]
pub struct LowerStepNode;

impl LowerStepNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        //let v = inputs.request_float("Value")?;
        //let f = make_function_box(crate::ops::Square);
        let f:DoubleFunctionOperatorBox = (|x| if x>0.0 {1.0} else {0.0}).into();

        outputs.set_value("F", f.into())?;
        Ok(())
    }
}

#[derive(Clone,Debug)]
pub struct SumNode;

impl SumNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {

        let f1 = inputs.request_function("F1")?;
        let f2 = inputs.request_function("F2")?;
        //let f = make_function_box(crate::ops::TwoSum(f1,f2));
        let f = f1.map2(f2, |x,y| x+y);
        outputs.set_value("F", f.into())?;
        Ok(())
    }
}

#[derive(Clone,Debug)]
pub struct MultiplyNode;

impl MultiplyNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        //let v = inputs.request_float("Value")?;
        let f1 = inputs.request_function("F1")?;
        let f2 = inputs.request_function("F2")?;
        //let f = make_function_box(crate::ops::Multiply(f1,f2));
        let f = f1.map2(f2, |x, y| x*y);
        outputs.set_value("F", f.into())?;
        Ok(())
    }
}


#[derive(Clone,Debug)]
pub struct MinNode;

impl MinNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        //let v = inputs.request_float("Value")?;
        let f1 = inputs.request_function("F1")?;
        let f2 = inputs.request_function("F2")?;
        //let f = make_function_box(crate::ops::Multiply(f1,f2));
        let f = f1.map2(f2, |x, y| x.min(y));
        outputs.set_value("F", f.into())?;
        Ok(())
    }
}

#[derive(Clone,Debug)]
pub struct MaxNode;

impl MaxNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        //let v = inputs.request_float("Value")?;
        let f1 = inputs.request_function("F1")?;
        let f2 = inputs.request_function("F2")?;
        //let f = make_function_box(crate::ops::Multiply(f1,f2));
        let f = f1.map2(f2, |x, y| x.max(y));
        outputs.set_value("F", f.into())?;
        Ok(())
    }
}

#[derive(Clone,Debug)]
pub struct ExponentNode;


impl ExponentNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        let f0 = inputs.request_function("F")?;
        let f = f0.map(|x| x.exp());
        //let f = make_function_box(crate::ops::Exponent(f0));
        outputs.set_value("F", f.into())?;
        Ok(())
    }
}



#[derive(Clone,Debug)]
pub struct LogNode;


impl LogNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        let f0 = inputs.request_function("F")?;
        let f = f0.map(|x| x.ln());
        //let f = make_function_box(crate::ops::Log(f0));
        outputs.set_value("F", f.into())?;
        Ok(())
    }
}

#[derive(Clone,Debug)]
pub struct AbsNode;

impl AbsNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        let f0 = inputs.request_function("F")?;
        let f = f0.map(|x| x.abs());
        //let f = make_function_box(crate::ops::Log(f0));
        outputs.set_value("F", f.into())?;
        Ok(())
    }
}

#[derive(Clone,Debug)]
pub struct NegNode;

impl NegNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        let f0 = inputs.request_function("F")?;
        let f = f0.map(|x| -x);
        //let f = make_function_box(crate::ops::Log(f0));
        outputs.set_value("F", f.into())?;
        Ok(())
    }
}


fn category() -> RVec<RString>where {
    rvec!["Functions".into(), "Generic".into()]
}

fn category0() -> RVec<RString>where {
    rvec!["Functions".into()]
}

impl CalculationNode for ConstantNode{
    fn name(&self,) -> RString where {
        "Constant".into()
    }

    fn category(&self,) -> RVec<RString>where {
        category0()
    }

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
        RSome("Functions/Constant".into())
    }

    fn identifier(&self,) -> RString where {
        "padamofunctions.constant".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Value", ContentType::Float)
        ]
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("F", ContentType::Function)
        ]
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![]
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}

impl CalculationNode for FCalculateNode{
    fn name(&self,) -> RString where {
        "Calculate value".into()
    }

    fn category(&self,) -> RVec<RString>where {
        category0()
    }

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
        RSome("Functions/Calculate value".into())
    }

    fn identifier(&self,) -> RString where {
        "padamofunctions.calculate_value".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("x", ContentType::Float),
            ("F", ContentType::Function)
        ]
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("y", ContentType::Float)
        ]
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![]
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}


implement_onearg_function!(LinearNode, "X", category, "linear", "X");
implement_onearg_function!(SquareNode, "X^2", category, "square", "X^2");
implement_onearg_function!(LowerStepNode, "Lower step function", category, "lowerstep", "Lower step function");

implement_binary_combinator!(SumNode, "Sum", category, "sum", "Sum");
implement_binary_combinator!(MultiplyNode, "Multiply", category, "mul", "Multiply");
implement_binary_combinator!(MinNode,"Min",category, "min", "Min");
implement_binary_combinator!(MaxNode,"Max",category, "max", "Max");

implement_unary_combinator!(ExponentNode, "Exponent", category,"exp","Exponent");
implement_unary_combinator!(LogNode, "Log", category,"log","Log");
implement_unary_combinator!(AbsNode, "Abs", category,"abs","Abs");
implement_unary_combinator!(NegNode, "Negate", category,"neg","Negate");

#[derive(Clone,Debug)]
pub struct LinearModificationNode;


impl LinearModificationNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        let f0 = inputs.request_function("F")?;
        let k = constants.request_float("coefficient")?;
        let b = constants.request_float("offset")?;

        let f = f0.map(move |x| x*k+b);
        //let f = make_function_box(crate::ops::Exponent(f0));
        outputs.set_value("F", f.into())?;
        Ok(())
    }
}

impl CalculationNode for LinearModificationNode{
    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString where {
        "Linear modification".into()
    }

    fn category(&self,) -> RVec<RString>where {
        category0()
    }

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
        RSome("Functions/Linear modification".into())
    }

    fn identifier(&self,) -> RString where {
        "padamofunctions.linear_modification".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("F", ContentType::Function)
        )
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Output definition of node"]
    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("F", ContentType::Function)
        )
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Constants definition of node with default values."]
    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![
            ("coefficient", 1.0),
            ("offset", 0.0)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}
