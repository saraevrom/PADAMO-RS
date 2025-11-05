use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use crate::{IcedForm, IcedFormBuffer};



pub trait ActionType: std::any::Any+Sync+Send+Clone+std::fmt::Debug+std::default::Default{}

impl<T:std::any::Any+Sync+Send+Clone+std::fmt::Debug+std::default::Default> ActionType for T{

}


pub trait ActionTrait<T:ActionType>: Default+Clone+Debug{
    fn make()->T;
}

#[derive(Clone,Debug,Default)]
pub struct ActionBuffer<T:ActionType,A:ActionTrait<T>>{
    _pd1:PhantomData<T>,
    _pd2:PhantomData<A>,
}

#[derive(Clone,Debug,Default)]
pub struct ActionMessage;

#[derive(Clone,Debug,Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Action<T:ActionType,A:ActionTrait<T>>{
    _pd1:PhantomData<T>,
    _pd2:PhantomData<A>,
}


impl<T:ActionType,A:ActionTrait<T>> IcedForm for Action<T,A>{
    type Buffer = ActionBuffer<T,A>;
}

impl <T:ActionType, A:ActionTrait<T>> IcedFormBuffer for ActionBuffer<T,A>{
    type FormType = Action<T,A>;
    type Message = ActionMessage;
    fn get(&self)->crate::Result<Self::FormType> {
        Ok(Default::default())
    }

    fn update(&mut self, _message:Self::Message) {

    }

    fn set(&mut self, _value:Self::FormType) {

    }

    fn from_value(_value:Self::FormType) ->Self {
        Default::default()
    }

    fn view<'a>(&'a self,title:Option<&'a str>)->iced::Element<'a,crate::ActionOrUpdate<Self::Message>,iced::Theme> {
        iced::widget::button(title.unwrap_or("---")).on_press(crate::ActionOrUpdate::Action(Arc::new(A::make()))).into()
    }
}
