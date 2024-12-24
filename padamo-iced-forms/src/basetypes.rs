
use crate::traits::{IcedForm,IcedFormBuffer,ActionOrUpdate};
use crate::double_entry_state::EntryState;

#[derive(Clone,Debug)]
pub struct SetBasicData(pub String);



macro_rules! impl_entry_base {
    ($ty:ident) => {
        impl IcedFormBuffer for EntryState<$ty> {
            type Message=SetBasicData;

            type FormType = $ty;

            fn view<'a>(&'a self,title:Option<&'a str>)->iced::Element<'a,ActionOrUpdate<SetBasicData>> {
                let res:iced::Element<'a,SetBasicData> = if let Some(v)=title{
                    self.view_row(v,"Value", SetBasicData).into()
                }
                else{
                    self.view("Value", SetBasicData).into()
                };
                res.map(|x| x.into())
            }

            fn update(&mut self, message:SetBasicData) {
                self.set_string(message.0);
            }

            fn get(&self)->crate::Result<$ty> {
                Ok(self.parsed_value.clone())
            }

            fn set(&mut self, value: $ty){
                self.set_value(value);
            }

            fn from_value(value: $ty) -> Self{
                Self::new(value)
            }
        }

        impl IcedForm for $ty{
            type Buffer = EntryState<$ty>;
        }

    };
}

impl_entry_base!(i8);
impl_entry_base!(u8);
impl_entry_base!(char);

impl_entry_base!(u16);
impl_entry_base!(i16);

impl_entry_base!(u32);
impl_entry_base!(i32);
impl_entry_base!(f32);


impl_entry_base!(u64);
impl_entry_base!(i64);
impl_entry_base!(f64);


impl_entry_base!(usize);
impl_entry_base!(isize);

impl_entry_base!(String);

#[derive(Clone,Debug,Default)]
pub struct BoolBuffer{
    pub inner:bool
}

#[derive(Clone,Debug)]
pub struct SetFlag(pub bool);

impl IcedFormBuffer for BoolBuffer{
    type Message = SetFlag;
    type FormType = bool;

    fn view<'a>(&'a self,title:Option<&'a str>)->iced::Element<'a,ActionOrUpdate<Self::Message>> {
        let t = title.unwrap_or("");
        iced::widget::checkbox(t,self.inner).on_toggle(|x| ActionOrUpdate::Update(SetFlag(x))).into()
    }

    fn update(&mut self, message:SetFlag) {
        self.inner = message.0;
    }

    fn get(&self)->crate::Result<bool> {
        Ok(self.inner)
    }

    fn set(&mut self, value:Self::FormType) {
        self.inner = value;
    }

    fn from_value(value:bool) ->Self {
        Self { inner: value }
    }
}

impl IcedForm for bool{
    type Buffer = BoolBuffer;
}
