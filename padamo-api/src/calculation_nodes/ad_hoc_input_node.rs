use std::collections::HashMap;

use abi_stable::rvec;
use abi_stable::std_types::RVec;
use crate::prelude::*;

use crate::{constants, ports, CalculationNode, ConstantContent, ContentType};

#[derive(Clone,Debug)]
pub struct AdHocOutputNode{
    pub inputs_mapping:HashMap<String,ContentType>,
    pub placeholder_identifier:String
}


impl AdHocOutputNode{
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

    fn calculate(&self,inputs:crate::ContentContainer,outputs: &mut crate::IOData,constants:crate::ConstantContentContainer,environment: &mut crate::ContentContainer,rng: &mut crate::RandomState,) -> Result<(),crate::ExecutionError>where {
        for (k,v) in self.inputs_mapping.iter(){
            let indata = inputs.request_type(v, k)?;
            environment.0.insert(format!("out_{}",k).into(), indata);
        }
        Ok(())
    }
}

impl CalculationNode for AdHocOutputNode{
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

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> abi_stable::std_types::RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment, rng).into()
    }
}
