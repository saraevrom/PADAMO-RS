use abi_stable::{rvec, std_types::{RResult, RString, RVec}};
use padamo_api::{constants, ports, prelude::*};
use padamo_api::function_operator::DoubleFunctionOperatorBox;
use crate::{implement_binary_combinator, implement_onearg_function, implement_unary_combinator};


#[derive(Clone,Debug)]
pub struct FCalculateNode;


impl FCalculateNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
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
    rvec!["Functions".into()]
}

impl CalculationNode for ConstantNode{
    fn name(&self,) -> RString where {
        "Constant".into()
    }

    fn category(&self,) -> RVec<RString>where {
        category()
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

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}

impl CalculationNode for FCalculateNode{
    fn name(&self,) -> RString where {
        "Calculate value".into()
    }

    fn category(&self,) -> RVec<RString>where {
        category()
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

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}


implement_onearg_function!(LinearNode, "X", category);
implement_onearg_function!(SquareNode, "X^2", category);
implement_onearg_function!(LowerStepNode, "Lower step function", category);

implement_binary_combinator!(SumNode, "Sum", category);
implement_binary_combinator!(MultiplyNode, "Multiply", category);
implement_binary_combinator!(MinNode,"Min",category);
implement_binary_combinator!(MaxNode,"Max",category);

implement_unary_combinator!(ExponentNode, "Exponent", category);
implement_unary_combinator!(LogNode, "Log", category);
implement_unary_combinator!(AbsNode, "Abs", category);
implement_unary_combinator!(NegNode, "Negate", category);

