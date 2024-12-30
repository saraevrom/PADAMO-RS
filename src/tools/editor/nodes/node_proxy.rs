use padamo_api::prelude::CalculationNodeBox;
use ordered_hash_map::OrderedHashMap;
use super::PortData;

#[derive(Clone,Debug)]
pub struct NodeProxy(pub CalculationNodeBox);

impl NodeProxy{
    #[allow(dead_code)]
    pub fn new(node:CalculationNodeBox)->Self{
        Self(node)
    }

    pub fn inputs(&self)->OrderedHashMap<String,PortData>{
        self.0.inputs().iter().map(|x| (x.name.to_string(),PortData{port_type:x.port_type,
                                                                    display_name:x.display_name.clone().into()})).collect()
    }

    pub fn outputs(&self)->OrderedHashMap<String,PortData>{
        self.0.outputs().iter().map(|x| (x.name.to_string(),PortData{port_type:x.port_type,
                                                                     display_name:x.display_name.clone().into()})).collect()
    }

    pub fn identifier(&self)->String{
        self.0.identifier().into()
    }

    pub fn title(&self)->String{
        self.0.name().into()
    }
}
