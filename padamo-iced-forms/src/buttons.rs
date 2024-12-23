use std::{fmt::Debug, marker::PhantomData};
use std::sync::Arc;

use crate::{IcedForm,IcedFormBuffer};

// #[derive(Clone,Debug)]
// pub struct ActionBuffer{
//     pub identifier:String
// }
//
// #[derive(Clone,Debug)]
// pub struct Action{
//     pub identifier:String
// }
//
// #[derive(Clone,Debug)]
// pub struct ActionMessage;
//
// impl

pub trait ActionType: std::any::Any+Sync+Send+Default+Clone+std::fmt::Debug{}

impl<T:std::any::Any+Sync+Send+Default+Clone+std::fmt::Debug> ActionType for T{

}

#[derive(Clone,Debug,Default)]
pub struct ActionBuffer<T:ActionType>{
    _pd:PhantomData<T>
}

#[derive(Clone,Debug)]
pub struct ActionMessage;

#[derive(Clone,Debug,Default)]
pub struct Action<T:std::any::Any+Sync+Send+Default>{
    _pd:PhantomData<T>
}

impl<T:ActionType> IcedForm for Action<T>{
    type Buffer = ActionBuffer<T>;
}

impl<T:ActionType> IcedFormBuffer for ActionBuffer<T>{
    type Message = ActionMessage;
    type FormType = Action<T>;

    fn get(&self)->Option<Self::FormType> {
        Default::default()
    }

    fn update(&mut self, _message:Self::Message) {

    }

    fn set(&mut self, _value:Self::FormType) {

    }

    fn from_value(_value:Self::FormType) ->Self {
        Default::default()
    }

    fn view<'a>(&'a self,title:Option<&'a str>)->iced::Element<'a,crate::ActionOrUpdate<Self::Message>,iced::Theme> {
        iced::widget::button(title.unwrap_or("---")).on_press(crate::ActionOrUpdate::Action(Arc::new(T::default()))).into()
    }
}
