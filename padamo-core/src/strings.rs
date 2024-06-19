use padamo_api::{constants, nodes_vec, ports, prelude::*};
use abi_stable::{rvec, std_types::{RResult, RString, RVec}};



fn category()->RVec<RString>{
    rvec![
        "Strings".into()
    ]
}

#[derive(Clone,Debug)]
pub struct StringConcatNode;

impl StringConcatNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> Result<(),ExecutionError>where {
        let a = inputs.request_string("a")?.to_string();
        let b = inputs.request_string("b")?.to_string();
        let c = format!("{}{}",a,b);
        outputs.set_value("s", c.into())?;
        Ok(())
    }
}


impl CalculationNode for StringConcatNode{
    fn name(&self,) -> RString where {
        "String Concatenate".into()
    }

    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("a",ContentType::String),
            ("b",ContentType::String)
        )
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("s",ContentType::String)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!()
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment, rng).into()
    }
}



#[derive(Clone,Debug)]
pub struct StringReplaceNode;

impl StringReplaceNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> Result<(),ExecutionError>where {
        let s = inputs.request_string("s")?.to_string();
        let pattern = inputs.request_string("pattern")?.to_string();
        let rep = inputs.request_string("rep")?.to_string();
        let c = s.replace(&pattern, &rep);

        outputs.set_value("s", c.into())?;
        Ok(())
    }
}


impl CalculationNode for StringReplaceNode{
    fn name(&self,) -> RString where {
        "String Replace".into()
    }

    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("s",ContentType::String),
            ("pattern",ContentType::String),
            ("rep",ContentType::String)
        )
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("s",ContentType::String)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!()
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment, rng).into()
    }
}


#[derive(Clone,Debug)]
pub struct StringReplaceRegexNode;

impl StringReplaceRegexNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> Result<(),ExecutionError>where {
        let s = inputs.request_string("s")?.to_string();
        let pattern = inputs.request_string("pattern")?.to_string();
        let rep = inputs.request_string("rep")?.to_string();


        let re = regex::Regex::new(&pattern).map_err(|x| ExecutionError::OtherError(format!("Regex error: {:?}",x).into()))?;
        let cow = re.replace_all(&s, rep);
        let c = cow.into_owned();

        outputs.set_value("s", c.into())?;
        Ok(())
    }
}


impl CalculationNode for StringReplaceRegexNode{
    fn name(&self,) -> RString where {
        "String Replace (Regex)".into()
    }

    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("s",ContentType::String),
            ("pattern",ContentType::String),
            ("rep",ContentType::String)
        )
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("s",ContentType::String)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!()
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment, rng).into()
    }
}




pub fn nodes()->RVec<CalculationNodeBox>{
    nodes_vec![
        StringConcatNode,
        StringReplaceNode,
        StringReplaceRegexNode
    ]
}
