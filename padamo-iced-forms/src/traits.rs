use iced::Element;
use std::{any::Any, sync::Arc};

#[derive(Clone,Debug)]
pub enum ActionOrUpdate<T:std::fmt::Debug+std::clone::Clone>{
    Update(T),
    Action(Arc<dyn Any + Send + Sync>)
}

impl<T:std::fmt::Debug+std::clone::Clone> ActionOrUpdate<T>{
    pub fn map<U:std::fmt::Debug+std::clone::Clone, F: FnOnce(T) -> U>(self, f:F)->ActionOrUpdate<U>{
        match self {
            Self::Action(a)=>ActionOrUpdate::Action(a),
            Self::Update(t) => ActionOrUpdate::Update(f(t)),
        }
    }
}

impl <T:std::fmt::Debug+std::clone::Clone> From<T> for ActionOrUpdate<T>{
    fn from(value: T) -> Self {
        Self::Update(value)
    }
}


pub trait IcedFormBuffer<Theme=iced::Theme,Renderer=iced::Renderer>:std::fmt::Debug+std::default::Default+std::clone::Clone{
    type Message:std::fmt::Debug+std::clone::Clone;
    type FormType:IcedForm;
    fn view<'a>(&'a self,title:Option<&'a str>)->Element<'a,ActionOrUpdate<Self::Message>,Theme,Renderer>;

    fn view_untitled<'a>(&'a self)->Element<'a,ActionOrUpdate<Self::Message>,Theme,Renderer>{
        self.view(None)
    }

    fn update(&mut self, message:Self::Message);
    fn get(&self)->crate::errors::Result<Self::FormType>;
    fn set(&mut self, value:Self::FormType);
    fn from_value(value:Self::FormType) ->Self;
}

pub trait IcedForm:Default {
    type Buffer: IcedFormBuffer<FormType = Self>;
}

// impl<T:IcedFormBuffer> LocalDefault for T{
//     fn default() -> Self {
//         T::from_value(T::FormType::default())
//     }
// }


