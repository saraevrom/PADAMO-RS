use padamo_api::prelude::CalculationNodeBox;
use ordered_hash_map::OrderedHashMap;
use padamo_api::prelude::*;

#[derive(Clone,Debug)]
pub struct NodeProxy(pub CalculationNodeBox);

impl NodeProxy{
    pub fn new(node:CalculationNodeBox)->Self{
        Self(node)
    }

    pub fn inputs(&self)->OrderedHashMap<String,ContentType>{
        self.0.inputs().iter().map(|x| (x.name.to_string(),x.port_type)).collect()
    }

    pub fn outputs(&self)->OrderedHashMap<String,ContentType>{
        self.0.outputs().iter().map(|x| (x.name.to_string(),x.port_type)).collect()
    }

    pub fn identifier(&self)->String{
        self.0.identifier().into()
    }

    pub fn title(&self)->String{
        self.0.name().into()
    }
}
