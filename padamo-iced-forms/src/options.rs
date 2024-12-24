use iced::futures::stream::Empty;

use crate::{IcedForm,IcedFormBuffer,ActionOrUpdate};

#[derive(Debug,Clone)]
pub enum OptionStash<T:IcedFormBuffer>{
    Empty,
    Stowed(T),
    Some(T)
}

impl <T:IcedFormBuffer> OptionStash<T>{
    pub fn stow(&mut self){
        let taken = std::mem::replace(self, Self::Empty);
        *self = match taken {
            Self::Empty => Self::Empty ,
            Self::Stowed(v) => Self::Stowed(v),
            Self::Some(v) => Self::Stowed(v),
        };
    }

    pub fn unstow(&mut self){
        let taken = std::mem::replace(self, Self::Empty);
        *self = match taken {
            Self::Empty => Self::Some(Default::default()) ,
            Self::Stowed(v) => Self::Some(v),
            Self::Some(v) => Self::Some(v),
        };
    }

    pub fn is_active(&self)->bool{
        if let Self::Some(_)= self{
            true
        }
        else{
            false
        }
    }
}

#[derive(Debug,Clone)]
pub struct OptionBuffer<T:IcedFormBuffer>{
    pub stash:OptionStash<T>,
}

#[derive(Debug,Clone)]
pub enum OptionMessage<T:IcedFormBuffer>{
    SetEnable(bool),
    Update(T::Message)
}


impl<T:IcedForm> IcedForm for Option<T>{
    type Buffer = OptionBuffer<T::Buffer>;
}

impl <T:IcedFormBuffer> IcedFormBuffer for OptionBuffer<T>{
    type FormType = Option<T::FormType>;
    type Message = OptionMessage<T>;

    fn view<'a>(&'a self,title:Option<&'a str>)->iced::Element<'a,ActionOrUpdate<Self::Message>> {
        let title = title.unwrap_or_default();
        let header_check = iced::widget::checkbox(title, self.stash.is_active())
            .on_toggle(|x| ActionOrUpdate::Update(OptionMessage::SetEnable(x)));
        let bottom_view = if let OptionStash::Some(v) = &self.stash{
            v.view_untitled()
        }
        else{
            iced::widget::column![].into()
        };
        let merged = iced::widget::column![
            //header_check,bottom_view.map(OptionMessage::Update)
            header_check,
            bottom_view.map(|outer_msg| outer_msg.map(OptionMessage::Update))
        ];
        merged.into()
        //let merged:iced::Element<'a,Self::Message> = merged.into();

    }

    fn update(&mut self, message: OptionMessage<T>) {
        match message {
            OptionMessage::SetEnable(v)=>{
                if v{
                    self.stash.unstow();
                }
                else{
                    self.stash.stow();
                }
            }
            OptionMessage::Update(v)=>{
                if let OptionStash::Some(stash) = &mut self.stash{
                    IcedFormBuffer::update(stash, v);
                }

            }
        }
    }

    fn get(&self)->crate::Result< Option<T::FormType>> {
        match &self.stash{
            OptionStash::Empty=>Ok(None),
            OptionStash::Stowed(_)=>Ok(None),
            OptionStash::Some(v) => {
                v.get().map_err(|x| x.map("Inner of")).map(Some)
                // if let Some(res) = v.get(){
                //     Some(Some(res))
                // }
                // else{
                //     None
                // }
            }
        }
    }

    fn set(&mut self, value:Self::FormType) {
        match value {
            Some(v)=>{
                self.stash = OptionStash::Some(T::from_value(v));
            },
            None=>{
                self.stash = OptionStash::Empty;
            }
        }
    }

    fn from_value(value:Self::FormType) ->Self {
        let mut res = Self{stash:OptionStash::Empty};
        res.set(value);
        res
    }
}

impl<T:IcedFormBuffer> Default for OptionBuffer<T>{
    fn default() -> Self {
        Self::from_value(Default::default())
    }
}
