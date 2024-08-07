use padamo_api::{constants, nodes_vec, ports, prelude::*};
use abi_stable::{rvec, std_types::{ROption::RSome, RResult, RString, RVec}};



fn category()->RVec<RString>{
    rvec![
        "Legacy".into(),
        "Strings".into()
    ]
}

#[derive(Clone,Debug)]
pub struct StringConcatNodeOld;

impl StringConcatNodeOld{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> Result<(),ExecutionError>where {
        let a = inputs.request_string("a")?.to_string();
        let b = inputs.request_string("b")?.to_string();
        let c = format!("{}{}",a,b);
        outputs.set_value("s", c.into())?;
        Ok(())
    }
}


impl CalculationNode for StringConcatNodeOld{
    fn name(&self,) -> RString where {
        "String Concatenate (Legacy)".into()
    }

    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
        RSome("Strings/String Concatenate".into())
    }

    fn identifier(&self,) -> RString where {
        "padamocore.strings.concatenate".into()
        //format!("padamocore.constant.{}",idmark).into()
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
        constants!(
        )
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment, rng).into()
    }
}



#[derive(Clone,Debug)]
pub struct StringReplaceNodeOld;

impl StringReplaceNodeOld{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> Result<(),ExecutionError>where {
        let s = inputs.request_string("s")?.to_string();
        let pattern = inputs.request_string("pattern")?.to_string();
        let rep = inputs.request_string("rep")?.to_string();
        let c = s.replace(&pattern, &rep);

        outputs.set_value("s", c.into())?;
        Ok(())
    }
}


impl CalculationNode for StringReplaceNodeOld{
    fn name(&self,) -> RString where {
        "String Replace (Legacy)".into()
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

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
        RSome("Strings/String Replace".into())
    }

    fn identifier(&self,) -> RString where {
        "padamocore.strings.replace".into()
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
pub struct StringReplaceRegexNodeOld;

impl StringReplaceRegexNodeOld{
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


impl CalculationNode for StringReplaceRegexNodeOld{
    fn name(&self,) -> RString where {
        "String Replace (Regex) (Legacy)".into()
    }

    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
        RSome("Strings/String Replace (Regex)".into())
    }

    fn identifier(&self,) -> RString where {
        "padamocore.strings.replace_regex".into()
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
        StringConcatNodeOld,
        StringReplaceNodeOld,
        StringReplaceRegexNodeOld
    ]
}
