use std::{fmt::Debug, marker::PhantomData};

use crate::{IcedForm, IcedFormBuffer};
use abi_stable::std_types::{RString, ROption};

// pub trait ProxyMark<T:IcedForm>: abi_stable::StableAbi{}

pub trait SameAs{
    type Output;
}

impl<T> SameAs for T{
    type Output = T;
}

pub trait Convertable<T>{
    fn fwd(self)->T;
    fn inv(x:T)->Self;
}

impl<T,U> Convertable<T> for U where
T:From<U>,
U:From<T>,
{
    fn fwd(self)->T {
        self.into()
    }

    fn inv(x:T)->Self {
        x.into()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ProxyBuffer<Subject, Proxy> where
    Proxy:IcedForm
{
    buffer: <Proxy as IcedForm>::Buffer,
    _pd:PhantomData<Subject>
}

impl<Subject, Proxy> ProxyBuffer<Subject, Proxy>
where
    Proxy:IcedForm+Debug
{
    pub fn new(buffer: <Proxy as IcedForm>::Buffer) -> Self {
        Self { buffer, _pd: PhantomData }
    }
}

impl<Subject, Proxy> IcedFormBuffer for ProxyBuffer<Subject, Proxy> where
    Subject: IcedForm+Convertable<<Proxy::Buffer as IcedFormBuffer>::FormType>+Debug+Default+Clone,
    Proxy:IcedForm+Debug+Clone+SameAs<Output = <Proxy::Buffer as IcedFormBuffer>::FormType>,
{
    type Message = <<Proxy as IcedForm>::Buffer as IcedFormBuffer>::Message;
    type FormType = Subject;

    fn view<'a>(&'a self,title:Option<&'a str>)->iced::Element<'a,crate::ActionOrUpdate<Self::Message>> {
        self.buffer.view(title)
    }

    fn view_untitled<'a>(&'a self)->iced::Element<'a,crate::ActionOrUpdate<Self::Message>> {
        self.buffer.view_untitled()
    }

    fn update(&mut self, message:Self::Message) {
        self.buffer.update(message);
    }

    fn get(&self)->crate::errors::Result<Subject> {
        self.buffer.get().map(Convertable::inv)
    }

    fn set(&mut self, value:Self::FormType) {
        self.buffer.set(value.fwd());
    }

    fn from_value(value:Self::FormType) ->Self {
        let inner:<Proxy::Buffer as IcedFormBuffer>::FormType  = value.fwd();
        let subbuffer = <Proxy::Buffer as IcedFormBuffer>::from_value(inner);
        Self::new(subbuffer)
    }
}

// impl ProxyMark<String> for RString{}
impl IcedForm for RString{
    type Buffer = ProxyBuffer<RString, String>;
}

// impl<T:abi_stable::StableAbi+IcedForm> Convertable<Option<T>> for ROption<T>{
//     fn fwd(self)->Option<T> {
//         self.into_option()
//     }
//
//     fn inv(x:Option<T>)->Self {
//         Self::from(x)
//     }
// }

// impl<T:abi_stable::StableAbi+IcedForm> ProxyMark<Option<T>> for ROption<T> {}
impl<T:abi_stable::StableAbi+IcedForm+Debug+Clone> IcedForm for ROption<T> {
    type Buffer = ProxyBuffer<ROption<T>, Option<T>>;
}
