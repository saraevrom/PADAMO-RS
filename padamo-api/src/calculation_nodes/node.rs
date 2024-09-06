use std::collections::HashMap;

use abi_stable::sabi_types::StaticRef;
use abi_stable::std_types::map::RefIterInterface;
use abi_stable::std_types::{RStr,RString,RVec,RBox,RHashMap,RResult, ROption, Tuple2};
use abi_stable::{StableAbi, DynTrait, rvec};
use super::content::{Content, ContentContainer, ContentType, ConstantContent, ConstantContentContainer};
use super::errors::ExecutionError;
use super::graph::PortKey;
use crate::rng::RandomState;



#[repr(C)]
#[derive(StableAbi,Clone,Debug)]
pub struct CalculationIO{
    pub name: RString,
    pub display_name: RString,
    pub port_type: ContentType,
}

impl CalculationIO{
    pub fn new(name:&str, port_type: ContentType)->Self{
        Self{
            name: name.into(),
            display_name: name.into(),
            port_type
        }
    }

    pub fn new_named(name:&str, display_name:&str, port_type: ContentType)->Self{
        Self{
            name: name.into(),
            display_name: display_name.into(),
            port_type
        }
    }
}

impl Into<CalculationIO> for (&str,ContentType){
    fn into(self) -> CalculationIO {
        CalculationIO::new(self.0, self.1)
    }
}

impl Into<CalculationIO> for (&str,&str,ContentType){
    fn into(self) -> CalculationIO {
        CalculationIO::new_named(self.0, self.1.into(), self.2)
    }
}

#[repr(C)]
#[derive(StableAbi,Clone,Debug)]
pub struct CalculationConstant{
    pub name: RString,
    pub default_value:ConstantContent,
    pub display_name:RString,
}

impl<T> Into<CalculationConstant> for (&str, T)
where
    T: Into<ConstantContent>
{
    fn into(self) -> CalculationConstant {
        CalculationConstant::new(self.0, self.1.into())
    }
}

impl<T> Into<CalculationConstant> for (&str, &str, T)
where
    T: Into<ConstantContent>
{
    fn into(self) -> CalculationConstant {
        CalculationConstant::new_named(self.0, self.1.into(), self.2.into())
    }
}


impl CalculationConstant{
    pub fn new(name:&str, default_value:ConstantContent)->Self{
        Self{
            name:name.into(),
            display_name:name.into(),
            default_value
        }
    }

    pub fn new_named(name:&str, display_name:&str, default_value:ConstantContent)->Self{
        Self{
            name:name.into(),
            display_name:display_name.into(),
            default_value
        }
    }
}

#[repr(C)]
#[derive(StableAbi,Clone,Debug)]
pub struct IOData{
    data: RHashMap<RString, ROption<Content>>
}

impl IOData{
    pub fn new(mut definitions:RVec<CalculationIO>)->Self{
        let mut data:RHashMap<RString, ROption<Content>> = RHashMap::new();
        for def in definitions.drain(..){
            data.insert(def.name.clone(), None.into());
        }
        Self { data }
    }

    pub fn is_valid(&self)->bool{
        for entry in self.data.iter(){
            let tup:Tuple2<&RString,&ROption<Content>> = entry;
            if let ROption::RNone = tup.1{
                return false;
            }
        }
        true
    }

    pub fn set_value(&mut self, key:&str, value:Content)->Result<(),ExecutionError>{
        if let Some(v) = self.data.get_mut(key.into()){
            *v = ROption::RSome(value);
            Ok(())
        }
        else{
            Err(ExecutionError::MissingPort(key.into()))
        }
    }

    pub fn clarify(mut self)->Result<RHashMap<RString, Content>,ExecutionError>{
        let mut res:RHashMap<RString, Content> = RHashMap::new();
        for entry in self.data.drain(){
            let tup:Tuple2<RString,ROption<Content>> = entry;
            if let ROption::RSome(x) = tup.1{
                res.insert(tup.0, x);
            }
            else{
                return Err(ExecutionError::UnfilledOutputs);
            }
        }
        Ok(res)
    }


}


/// Trait for calculation node
#[abi_stable::sabi_trait]
pub trait CalculationNode: Debug+Clone{

    /// Name of node displayed in graph editor or node list
    fn name(&self)->RString;

    /// Category to place node in node list
    fn category(&self)->RVec<RString>{
        rvec![]
    }

    fn identifier(&self)->RString{
        self.path().join("/").into()
    }

    fn old_identifier(&self)->ROption<RString>{
        ROption::RNone
    }

    /// If node requires calculation by anyway. Set it to true if node is outputting data in environment or somewhere else
    fn is_primary(&self)->bool{
        false
    }

    /// Input definitions of node
    fn inputs(&self)->RVec<CalculationIO>;

    /// Output definition of node
    fn outputs(&self)->RVec<CalculationIO>;

    /// Constants definition of node with default values.
    fn constants(&self)->RVec<CalculationConstant>;

    /// Main calculation
    fn calculate(&self, inputs:ContentContainer, outputs:&mut IOData, constants:ConstantContentContainer, environment:&mut ContentContainer, rng:&mut RandomState)->RResult<(),ExecutionError>;


    fn path(&self)->RVec<RString>{
        let mut category = self.category();
        let name = self.name();
        category.push(name);
        category
    }
}

//pub type CalculationNodeStatic = CalculationNode_TO<'static, StaticRef<()>>;
pub type CalculationNodeBox = CalculationNode_TO<'static, RBox<()>>;


#[repr(C)]
#[derive(StableAbi,Debug)]
pub struct CalculationNodeObject{
    pub calculator:CalculationNodeBox,
    pub constants:ConstantContentContainer,
    pub constants_external_flags: RHashMap<RString,bool>,
    pub input_links: RHashMap<RString, ROption<PortKey>>,
}




impl CalculationNodeObject{
    pub fn new(calculator:CalculationNodeBox, constants_override:Option<ConstantContentContainer>, external_override:Option<RHashMap<RString,bool>>)->Self{
        let mut input_links: RHashMap<RString, ROption<PortKey>> = RHashMap::new();
        for link in calculator.inputs().drain(..){
            input_links.insert(link.name, ROption::RNone);
        }

        let constants = if let Some(c) = constants_override{c}
        else{
            ConstantContentContainer::from_rvec( calculator.constants())
        };

        let constants_external_flags = if let Some(f) = external_override{
            f
        }
        else{
            constants.0.iter().map(|x| (x.0.clone(),false)).collect()
        };

        for (const_value, const_external) in constants_external_flags.iter().map(|x| (x.0, x.1)){
            if *const_external{
                input_links.insert(format!("constant_{}",const_value).into(), ROption::RNone);
            }
        }

        Self {input_links, constants,calculator, constants_external_flags}
    }


    pub fn get_connections(&self)->RResult<RHashMap<RString, PortKey>,ExecutionError>{
        let mut res:RHashMap<RString, PortKey> = RHashMap::new();
        for link in self.input_links.iter(){
            let tup:Tuple2<&RString, &ROption<PortKey>> = link.into();
            //println!("{:?}",tup);
            if let ROption::RSome(c) = tup.1{
                res.insert(tup.0.clone(), c.clone());
            }
            else{
                return RResult::RErr(ExecutionError::NotConnected(tup.0.clone()));
            }
        }
        RResult::ROk(res)
    }

    pub fn connect_from(&mut self, port:&str, link:PortKey){
        if let Some(x) = self.input_links.get_mut(port.into()){
            *x = ROption::RSome(link);
        }
    }
}
