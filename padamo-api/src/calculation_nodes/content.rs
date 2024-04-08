use abi_stable::{StableAbi, std_types::{RString, RStr, RVec, Tuple3}};
use std::array::IntoIter;
use serde::{Serialize,Deserialize};

use super::errors::ExecutionError;
use padamo_api_macros_internal::impl_content;
use abi_stable::std_types::RHashMap;
use super::node::CalculationConstant;
use crate::lazy_array_operations::{LazyDetectorSignal, LazyTriSignal};

#[repr(C)]
#[derive(StableAbi,Clone,Debug)]
#[impl_content]
pub enum Content{
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(RString),
    DetectorSignal(LazyDetectorSignal),
    DetectorFullData(LazyTriSignal),
}


impl Into<Content> for String{
    fn into(self) -> Content {
        Content::String(self.into())
    }
}

impl Into<Content> for &str{
    fn into(self) -> Content {
        Content::String(self.into())
    }
}

impl Into<ConstantContent> for String{
    fn into(self) -> ConstantContent {
        ConstantContent::String(self.into())
    }
}

impl Into<ConstantContent> for &str{
    fn into(self) -> ConstantContent {
        ConstantContent::String(self.into())
    }
}

//Macro impl_content makes enum ContentType with stripped data
//ContentContainer is tuple struct


#[repr(C)]
#[derive(abi_stable::StableAbi,Clone,Debug)]
pub struct ContentContainer(pub RHashMap<RString,Content>);

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Color{
    pub r:f32,
    pub g:f32,
    pub b:f32,
    pub a:f32,
}

impl ContentType{
    pub fn get_color(&self)->Color{
        match self {
            ContentType::Boolean => Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 },
            ContentType::String => Color { r: 1.0, g: 0., b: 0.0, a: 1.0 },
            ContentType::Integer => Color { r: 0.0, g: 2./3., b: 0.733333, a: 1.0 },
            ContentType::Float => Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 },
            ContentType::DetectorSignal => Color { r: 1.0, g: 0.33333, b: 0.0, a: 1.0 },
            ContentType::DetectorFullData => Color { r: 0.3333333, g: 0.5, b: 0.0, a: 1.0 }
            //ContentType::Array => iced::Color { r: 1.0, g: 1./3., b: 0.0, a: 1.0 },
        }
    }

    // pub fn default_constant(&self)->Content{
    //     match self {
    //         Self::Boolean => Content::Boolean(Default::default()),
    //         Self::String => Content::String(Default::default()),
    //         Self::Integer => Content::Integer(Default::default()),
    //         Self::Float => Content::Float(Default::default()),
    //         _=>panic!("Unsupported constant type"),
    //     }
    // }
}

#[repr(C)]
#[derive(StableAbi,Clone,Debug,Serialize,Deserialize)]
#[impl_content]
pub enum ConstantContent{
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(RString),
}

impl ConstantContentType{
    pub fn default_constant_value(&self)->Content{
        match self {
            Self::Boolean => Content::Boolean(Default::default()),
            Self::String => Content::String(Default::default()),
            Self::Integer => Content::Integer(Default::default()),
            Self::Float => Content::Float(Default::default()),
        }
    }

    pub fn default_constant(&self)->ConstantContent{
        match self {
            Self::Boolean => ConstantContent::Boolean(Default::default()),
            Self::String => ConstantContent::String(Default::default()),
            Self::Integer => ConstantContent::Integer(Default::default()),
            Self::Float => ConstantContent::Float(Default::default()),
        }
    }
}

impl Into<Content> for ConstantContent{
    fn into(self) -> Content {
        match self{
            ConstantContent::Integer(x) => Content::Integer(x),
            ConstantContent::Float(x) => Content::Float(x),
            ConstantContent::Boolean(x) => Content::Boolean(x),
            ConstantContent::String(x) => Content::String(x),
        }
    }
}

impl Into<ContentType> for ConstantContentType{
    fn into(self) -> ContentType {
        match self{
            ConstantContentType::Integer => ContentType::Integer,
            ConstantContentType::Float => ContentType::Float,
            ConstantContentType::Boolean => ContentType::Boolean,
            ConstantContentType::String => ContentType::String,
        }
    }
}

#[repr(C)]
#[derive(abi_stable::StableAbi,Clone,Debug,Serialize,Deserialize)]
pub struct ConstantContentContainer(pub RHashMap<RString,ConstantContent>);

impl ConstantContentContainer{
    pub fn from_rvec(rvec:RVec<CalculationConstant>)->Self{
        let mut mapping:RHashMap<RString, ConstantContent> = RHashMap::new();
        for item in rvec.into_vec().drain(..){
            mapping.insert(item.name, item.default_value);
        }
        Self(mapping)
    }
}
