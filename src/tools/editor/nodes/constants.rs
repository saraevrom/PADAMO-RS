use super::errors::NodeError;
use ordered_hash_map::OrderedHashMap;
use serde::{Serialize,Deserialize};


#[derive(Clone,Debug,PartialEq)]
pub enum NodeConstantBuffer{
    Check(bool),
    Text(String)
}


#[derive(Clone,Debug,PartialEq)]
pub enum NodeConstantMessageContent{
    Check(bool),
    Text(String),
    ToggleExternal(bool)
}

#[derive(Clone,Debug,PartialEq)]
pub struct NodeConstantMessage{
    key:String,
    value:NodeConstantMessageContent

}


impl NodeConstantMessage{
    pub fn check(key: String)->impl Fn(bool)->Self{
        move |x| {
            Self { key: key.clone(), value: NodeConstantMessageContent::Check(x) }
        }
    }

    pub fn text(key: String)->impl Fn(String)->Self{
        move |x| {
            Self { key: key.clone(), value: NodeConstantMessageContent::Text(x) }
        }
    }

    pub fn external_toggle(key:String)->impl Fn(bool)->Self{
        move |x| {
            Self { key: key.clone(), value: NodeConstantMessageContent::ToggleExternal(x) }
        }
    }
}



#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
pub enum NodeConstantContent{
    Boolean(bool),
    Text(String),
    Integer(i64),
    Real(f64)
}




impl NodeConstantContent{
    pub fn is_compatible(&self, other:&Self)->bool{
        match (self,other) {
            (Self::Boolean(_),Self::Boolean(_))=>true,
            (Self::Text(_),Self::Text(_))=>true,
            (Self::Integer(_),Self::Integer(_))=>true,
            (Self::Real(_),Self::Real(_))=>true,
            _=>false,
        }
    }

    pub fn commit_changes(&mut self, other:&Self)->Result<(),NodeError>{
        if self.is_compatible(other){
            *self = other.clone();
            Ok(())
        }
        else{
            Err(NodeError::IncompatibleConstants)
        }
    }
}

impl Into<NodeConstantContent> for padamo_api::prelude::ConstantContent{
    fn into(self) -> NodeConstantContent {
        match self {
            Self::Integer(x)=>NodeConstantContent::Integer(x),
            Self::Boolean(b)=>NodeConstantContent::Boolean(b),
            Self::Float(f)=>NodeConstantContent::Real(f),
            Self::String(s)=>NodeConstantContent::Text(s.into_string()),
        }
    }
}

impl Into<padamo_api::prelude::ConstantContent> for NodeConstantContent{
    fn into(self) -> padamo_api::prelude::ConstantContent {
        match self{
            Self::Boolean(b)=>padamo_api::prelude::ConstantContent::Boolean(b),
            Self::Real(f)=>padamo_api::prelude::ConstantContent::Float(f),
            Self::Integer(i)=>padamo_api::prelude::ConstantContent::Integer(i),
            Self::Text(s)=>padamo_api::prelude::ConstantContent::String(s.into()),
        }
    }
}


#[derive(Clone,Debug,PartialEq)]
pub struct NodeConstant{
    pub buffer:NodeConstantBuffer,
    pub ok:bool,
    pub content:NodeConstantContent,
    default_value:NodeConstantContent,
    pub use_external:bool
}


impl NodeConstant{
    pub fn new(content:NodeConstantContent)->Self{
        Self { buffer: content.clone().into(),default_value:content.clone(), content, ok:true, use_external:false  }
    }

    pub fn update_buffer(&mut self){
        self.buffer = self.content.clone().into();
    }

    pub fn commit_buffer(&mut self, other:NodeConstantMessage){
        match other.value{
            NodeConstantMessageContent::Check(x) => {self.buffer = NodeConstantBuffer::Check(x);},
            NodeConstantMessageContent::Text(x) => {self.buffer = NodeConstantBuffer::Text(x);},
            _=>()
        }
    }

    pub fn parse_buffer(&mut self){
        // if let NodeConstantMessageContent::ToggleExternal(v) = self.buffer{
        //     self.use_external = v;
        //     return;
        // }
        match &self.content {
            NodeConstantContent::Text(_)=>{
                if let NodeConstantBuffer::Text(txt) = &self.buffer{
                    self.content = NodeConstantContent::Text(txt.into());
                    self.ok = true;
                }
                else{
                    self.ok = false;
                }
            },
            NodeConstantContent::Boolean(_)=>{
                if let NodeConstantBuffer::Check(b) = &self.buffer{
                    self.content = NodeConstantContent::Boolean(*b);
                    self.ok = true;
                }
                else{
                    self.ok = false;
                }
            }
            NodeConstantContent::Integer(_)=>{
                if let NodeConstantBuffer::Text(txt) = &self.buffer{
                    if let Ok(num) = txt.parse::<i64>() {
                        self.content = NodeConstantContent::Integer(num);
                        self.ok = true;
                    }
                    else{
                        self.ok = false;
                    }
                }
                else{
                    self.ok = false;
                }
            }
            NodeConstantContent::Real(_)=>{
                if let NodeConstantBuffer::Text(txt) = &self.buffer{
                    if let Ok(num) = txt.parse::<f64>() {
                        self.content = NodeConstantContent::Real(num);
                        self.ok = true;
                    }
                    else{
                        self.ok = false;
                    }
                }
                else{
                    self.ok = false;
                }
            }
        }
        if !self.ok{
            self.content.commit_changes(&self.default_value).unwrap();
        }
    }
}

#[derive(Clone,Debug,PartialEq)]
pub struct NodeConstantStorage{
    pub constants:OrderedHashMap<String,NodeConstant>,
}

impl NodeConstantStorage{
    pub fn new()->Self{
        Self { constants: OrderedHashMap::new() }
    }

    pub fn add_constant(&mut self, key:&str, value:NodeConstantContent){
        self.constants.insert(key.into(),value.into());
    }

    pub fn modify_constant(&mut self, msg:NodeConstantMessage)->Result<(),NodeError>{
        if let Some(v) = self.constants.get_mut(&msg.key){
            if let NodeConstantMessageContent::ToggleExternal(ext) = msg.value{
                v.use_external = ext;
            }
            else{
                v.commit_buffer(msg);
                v.parse_buffer();
            }

            Ok(())
        }
        else{
            Err(NodeError::MissingConstant(msg.key.clone()))
        }
    }


}


impl Into<NodeConstantContent> for &str{
    fn into(self) -> NodeConstantContent {
        NodeConstantContent::Text(self.into())
    }
}

impl Into<NodeConstantContent> for String{
    fn into(self) -> NodeConstantContent {
        NodeConstantContent::Text(self)
    }
}

impl Into<NodeConstantContent> for f64{
    fn into(self) -> NodeConstantContent {
        NodeConstantContent::Real(self)
    }
}

impl Into<NodeConstantContent> for i64{
    fn into(self) -> NodeConstantContent {
        NodeConstantContent::Integer(self)
    }
}

impl Into<NodeConstantContent> for bool{
    fn into(self)->NodeConstantContent{
        NodeConstantContent::Boolean(self)
    }
}

impl Into<NodeConstant> for NodeConstantContent{
    fn into(self) -> NodeConstant {
        NodeConstant::new(self)
    }
}



impl Into<String> for NodeConstantContent{
    fn into(self) -> String {
        match self {
            NodeConstantContent::Boolean(x) => x.to_string(),
            NodeConstantContent::Text(x) => x,
            NodeConstantContent::Integer(x) => x.to_string(),
            NodeConstantContent::Real(x) => x.to_string(),
        }
    }
}


impl Into<NodeConstantMessageContent> for NodeConstantContent{
    fn into(self) -> NodeConstantMessageContent {
        match self {
            NodeConstantContent::Boolean(x) => NodeConstantMessageContent::Check(x),
            x => NodeConstantMessageContent::Text(x.into()),
        }
    }
}

impl Into<NodeConstantBuffer> for NodeConstantContent{
    fn into(self) -> NodeConstantBuffer {
        match self {
            NodeConstantContent::Boolean(x) => NodeConstantBuffer::Check(x),
            x => NodeConstantBuffer::Text(x.into()),
        }
    }
}

