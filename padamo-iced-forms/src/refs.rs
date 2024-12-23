
use crate::{IcedForm,IcedFormBuffer,ActionOrUpdate};

#[derive(Debug,Clone)]
pub struct BoxBuffer<T:IcedFormBuffer>{
    pub inner:Box<T>
}

#[derive(Debug,Clone)]
pub struct BoxMessage<T:IcedFormBuffer>{
    pub inner:Box<T::Message>
}

impl<T:IcedForm> IcedForm for Box<T>{
    type Buffer = BoxBuffer<T::Buffer>;
}

fn unbox<T>(value: Box<T>) -> T {
    *value
}

impl<T:IcedFormBuffer> IcedFormBuffer for BoxBuffer<T>{
    type FormType = Box<T::FormType>;
    type Message = BoxMessage<T>;

    fn view<'a>(&'a self,title:Option<&'a str>)->iced::Element<'a,ActionOrUpdate<Self::Message>,iced::Theme> {
        self.inner.view(title).map(|outer| outer.map (|x| BoxMessage { inner: Box::new(x) }))
    }

    fn update(&mut self, message:Self::Message) {
        self.inner.update(unbox(message.inner));
    }

    fn get(&self)->Option<Self::FormType> {
        self.inner.get().map(|x| Box::new(x))
    }

    fn set(&mut self, value:Self::FormType) {
        self.inner.set(unbox(value));
    }

    fn from_value(value:Self::FormType) ->Self {
        Self{inner: Box::new(T::from_value(unbox(value)))}
    }
}

impl <T:IcedFormBuffer> Default for BoxBuffer<T>{
    fn default()->Self{
        Self::from_value(Default::default())
    }
}
