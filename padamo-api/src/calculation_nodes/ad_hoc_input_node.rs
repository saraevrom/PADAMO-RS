use std::collections::HashMap;

use abi_stable::rvec;
use abi_stable::std_types::RVec;
use crate::prelude::*;

use crate::{constants, ports, CalculationNode, ContentType};

#[derive(Clone,Debug)]
pub struct AdHocInputNode{
    pub inputs_mapping:HashMap<String,ContentType>,
    pub placeholder_identifier:String
}


impl AdHocInputNode{
    pub fn new<T:Into<String>>(identifier:T)->Self{
        Self { inputs_mapping: HashMap::new() ,placeholder_identifier:identifier.into()}
    }

    pub fn single_value<T:Into<String>,U:Into<String>>(identifier:T, port:U, value:ContentType)->Self{
        Self::new(identifier.into()).with_value(port.into(), value)
    }

    pub fn with_value(mut self, key:String, value:ContentType)->Self{
        self.inputs_mapping.insert(key,value.into());
        self
    }

    fn calculate(&self,args:CalculationNodeArguments) -> Result<(),crate::ExecutionError>where {
        for (k,v) in self.inputs_mapping.iter(){
            let indata = args.inputs.request_type(v, k)?;
            args.environment.0.insert(format!("out_{}",k).into(), indata);
        }
        Ok(())
    }
}

impl CalculationNode for AdHocInputNode{
    fn name(&self,) -> abi_stable::std_types::RString where {
        self.placeholder_identifier.clone().into()
    }

    fn identifier(&self,) -> abi_stable::std_types::RString where {
        self.placeholder_identifier.clone().into()
    }

    fn category(&self,) -> abi_stable::std_types::RVec<abi_stable::std_types::RString>where {
        rvec!["Ad Hoc".into()]
    }

    fn is_primary(&self,) -> bool where {
        true
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports!()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        self.inputs_mapping.iter().map(|(k,v)|{
            CalculationIO::new(k, v.clone())
        }).collect()
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!()
    }

    fn calculate(&self,args:CalculationNodeArguments) -> abi_stable::std_types::RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}
