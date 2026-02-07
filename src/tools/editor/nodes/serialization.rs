use std::collections::HashMap;

use crate::tools::editor::nodes::constants::NodeConstantContent;

#[derive(serde::Serialize, serde::Deserialize, Clone,Copy)]
pub struct SerdePoint{
    x:f32,
    y:f32
}


impl From<iced::Point> for SerdePoint{
    fn from(val: iced::Point) -> Self {
        SerdePoint { x: val.x, y: val.y }
    }
}

impl From<SerdePoint> for iced::Point{
    fn from(val: SerdePoint)-> Self{
        iced::Point { x: val.x, y: val.y }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SerdeConnection{
    pub node_index:usize,              //source node
    pub port:String                    //output port
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct SerializationEntry{
    pub position:SerdePoint,
    pub identifier:String,
    pub connections:HashMap<String,Option<SerdeConnection>>,
    pub constants:HashMap<String,NodeConstantContent>,
    pub constants_external_flags:HashMap<String,bool>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct SerializedNodes(pub Vec<SerializationEntry>);
