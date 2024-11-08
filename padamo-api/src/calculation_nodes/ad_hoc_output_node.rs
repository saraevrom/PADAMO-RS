use std::collections::HashMap;

use abi_stable::rvec;
use abi_stable::std_types::RVec;
use crate::prelude::*;

use crate::{constants, ports, CalculationNode, ConstantContent, ContentType};

#[derive(Clone,Debug)]
pub struct AdHocOutputNode{
    pub values:HashMap<String,ConstantContent>,
    pub placeholder_identifier:String
}

impl AdHocOutputNode{
    pub fn new<T:Into<String>>(identifier:T)->Self{
        Self { values: HashMap::new() ,placeholder_identifier:identifier.into()}
    }

    pub fn single_value<T:Into<String>,U:Into<String>,V:Into<ConstantContent>>(identifier:T, port:U, value:V)->Self{
        Self::new(identifier.into()).with_value(port.into(), value)
    }

    pub fn with_value<T:Into<ConstantContent>>(mut self, key:String, value:T)->Self{
        self.values.insert(key,value.into());
        self
    }

    fn calculate(&self,inputs:crate::ContentContainer,outputs: &mut crate::IOData,constants:crate::ConstantContentContainer,environment: &mut crate::ContentContainer,rng: &mut crate::RandomState,) -> Result<(),crate::ExecutionError>where {
        for (k,v) in self.values.iter(){
            outputs.set_value(k.as_str(), v.clone().into())?;
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

    fn inputs(&self,) -> abi_stable::std_types::RVec<crate::CalculationIO>where {
        ports!()
    }

    fn outputs(&self,) -> abi_stable::std_types::RVec<crate::CalculationIO>where {
        let imm:Vec<crate::CalculationIO> = self.values.iter().map(|(k,v)| {
            let ty = match v{
                ConstantContent::Integer(_)=>ContentType::Integer,
                ConstantContent::Float(_)=>ContentType::Float,
                ConstantContent::Boolean(_)=>ContentType::Boolean,
                ConstantContent::String(_)=>ContentType::String,
            };
            crate::CalculationIO::new(k.as_str(), ty)
        }).collect();
        imm.into()
    }

    fn constants(&self,) -> RVec<crate::CalculationConstant>where {
        constants!()
    }

    fn calculate(&self,inputs:crate::ContentContainer,outputs: &mut crate::IOData,constants:crate::ConstantContentContainer,environment: &mut crate::ContentContainer,rng: &mut crate::RandomState,) -> abi_stable::std_types::RResult<(),crate::ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment, rng).into()
    }
}
