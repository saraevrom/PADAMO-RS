use abi_stable::{rvec, std_types::{ROption::RSome, RResult, RString, RVec}};
use padamo_api::{constants, ports, prelude::*};
use padamo_api::function_operator::DoubleFunctionOperatorBox;
use crate::{implement_binary_combinator, implement_onearg_function, implement_unary_combinator};


#[derive(Clone,Debug)]
pub struct FCalculateNode;


impl FCalculateNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let f0 = args.inputs.request_function("F")?;
        let x = args.inputs.request_float("x")?;
        let y = f0.calculate(x);
        args.outputs.set_value("y", y.into())?;
        Ok(())
    }
}


#[derive(Clone,Debug)]
pub struct FCalculateNode2;


impl FCalculateNode2{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let f0 = args.inputs.request_function("F")?;
        let x = args.constants.request_float("x")?;
        let y = f0.calculate(x);
        args.outputs.set_value("y", y.into())?;
        Ok(())
    }
}



#[derive(Clone,Debug)]
pub struct ConstantNode;
impl ConstantNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let v = args.inputs.request_float("Value")?;
        //let f = make_function_box(crate::ops::Constant(v));
        let f:DoubleFunctionOperatorBox = (move |_| {v}).into();
        args.outputs.set_value("F", f.into())?;
        Ok(())
    }
}


#[derive(Clone,Debug)]
pub struct ConstantNode2;
impl ConstantNode2{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let v = args.constants.request_float("Value")?;
        //let f = make_function_box(crate::ops::Constant(v));
        let f:DoubleFunctionOperatorBox = (move |_| {v}).into();
        args.outputs.set_value("F", f.into())?;
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
        let mut res = rvec!["Legacy".into()];
        res.extend(category0());
        res
    }

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
        RSome("Functions/Constant".into())
    }

    fn identifier(&self,) -> RString where {
        "padamofunctions.constant2".into()
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

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}

impl CalculationNode for ConstantNode2{
    fn name(&self,) -> RString where {
        "Constant".into()
    }

    fn category(&self,) -> RVec<RString>where {
        category0()
    }

    fn identifier(&self,) -> RString where {
        "padamofunctions.constant".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports![

        ]
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("F", ContentType::Function)
        ]
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![
            ("Value", 0.0)
        ]
    }

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}

impl CalculationNode for FCalculateNode{
    fn name(&self,) -> RString where {
        "Calculate value (Legacy)".into()
    }

    fn category(&self,) -> RVec<RString>where {
        let mut res = rvec!["Legacy".into()];
        res.extend(category0());
        res
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

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}

impl CalculationNode for FCalculateNode2{
    fn name(&self,) -> RString where {
        "Calculate value".into()
    }

    fn category(&self,) -> RVec<RString>where {
        category0()
    }

    fn identifier(&self,) -> RString where {
        "padamofunctions.calculate_value2".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("F", ContentType::Function)
        ]
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("y", ContentType::Float)
        ]
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![
            ("x", 0.0),
        ]
    }

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}

implement_onearg_function!(LinearNode, "X", category, "linear", "X", |x| {x});
implement_onearg_function!(SquareNode, "X^2", category, "square", "X^2", |x| {x});
implement_onearg_function!(LowerStepNode, "Lower step function", category, "lowerstep", "Lower step function", |x| if x>0.0 {1.0} else {0.0});

implement_binary_combinator!(SumNode, "Sum", category, "sum", "Sum", |x,y| x+y);
implement_binary_combinator!(MultiplyNode, "Multiply", category, "mul", "Multiply", |x,y| x*y);
implement_binary_combinator!(MinNode,"Min",category, "min", "Min", |x,y| x.min(y));
implement_binary_combinator!(MaxNode,"Max",category, "max", "Max", |x,y| x.max(y));

implement_unary_combinator!(ExponentNode, "Exponent", category,"exp","Exponent",|x| x.exp());
implement_unary_combinator!(LogNode, "Log", category,"log","Log", |x| x.ln());
implement_unary_combinator!(AbsNode, "Abs", category,"abs","Abs",|x| x.abs());
implement_unary_combinator!(NegNode, "Negate", category,"neg","Negate", |x| -x);
implement_unary_combinator!(InvNode, "Invert", category,"inv","Invert", |x| if x==0.0 {0.0} else {1.0/x});

#[derive(Clone,Debug)]
pub struct LinearModificationNode;


impl LinearModificationNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let f0 = args.inputs.request_function("F")?;
        let k = args.constants.request_float("coefficient")?;
        let b = args.constants.request_float("offset")?;

        let f = f0.map(move |x| x*k+b);
        //let f = make_function_box(crate::ops::Exponent(f0));
        args.outputs.set_value("F", f.into())?;
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
    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}
