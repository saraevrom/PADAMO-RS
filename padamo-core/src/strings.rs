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
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let a = args.constants.request_string("a")?.to_string();
        let b = args.constants.request_string("b")?.to_string();
        let c = format!("{}{}",a,b);
        args.outputs.set_value("s", c.into())?;
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

    fn identifier(&self,) -> RString where {
        "padamocore.strings.concatenate2".into()
        //format!("padamocore.constant.{}",idmark).into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            // ("a",ContentType::String),
            // ("b",ContentType::String)
        )
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("s",ContentType::String)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![
            ("a",""),
            ("b",""),
        ]
    }

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}



#[derive(Clone,Debug)]
pub struct StringReplaceNode;

impl StringReplaceNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let is_regex = args.constants.request_boolean("regex")?;
        let pattern = args.constants.request_string("pattern")?.to_string();
        let rep = args.constants.request_string("rep")?.to_string();

        if is_regex{
            let s = args.inputs.request_string("s")?.to_string();


            let re = regex::Regex::new(&pattern).map_err(|x| ExecutionError::OtherError(format!("Regex error: {:?}",x).into()))?;
            let cow = re.replace_all(&s, rep);
            let c = cow.into_owned();

            args.outputs.set_value("s", c.into())?;
            Ok(())
        }
        else{
            let s = args.inputs.request_string("s")?.to_string();
            let c = s.replace(&pattern, &rep);

            args.outputs.set_value("s", c.into())?;
            Ok(())
        }


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

        )
    }

    fn identifier(&self,) -> RString where {
        "padamocore.strings.replace2".into()
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("s",ContentType::String)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!(
            ("pattern",""),
            ("rep",""),
            ("regex",false)
        )
    }

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}

pub fn nodes()->RVec<CalculationNodeBox>{
    nodes_vec![
        StringConcatNode,
        StringReplaceNode,
        //StringReplaceRegexNode
    ]
}
