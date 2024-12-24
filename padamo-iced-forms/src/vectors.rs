
use crate::{IcedForm,IcedFormBuffer,ActionOrUpdate};


#[derive(Clone,Debug)]
pub struct VectorBuffer<T:IcedFormBuffer>{
    pub data:Vec<T>
}

#[derive(Debug,Clone)]
pub enum VectorMessage<T:IcedFormBuffer>{
    AddNew,
    Remove(usize),
    MoveUp(usize),
    MoveDown(usize),
    InsertAfter(usize),
    InsertBefore(usize),
    Update(usize, T::Message)
}

impl<T:IcedForm> IcedForm for Vec<T>{
    type Buffer = VectorBuffer<T::Buffer>;
}

impl <T:IcedFormBuffer> IcedFormBuffer for VectorBuffer<T>{
    type Message = VectorMessage<T>;
    type FormType = Vec<T::FormType>;

    fn update(&mut self, message:Self::Message) {
        match message {
            VectorMessage::AddNew=>{
                self.data.push(Default::default());
            },
            VectorMessage::Remove(i)=>{
                if i<self.data.len(){
                    self.data.remove(i);
                }
            },
            VectorMessage::MoveUp(i)=>{
                if 0<i && i<self.data.len(){
                    self.data.swap(i-1, i);
                }
            },
            VectorMessage::MoveDown(i)=>{
                if self.data.len()>1 && i<self.data.len()-1{
                    self.data.swap(i, i+1);
                }
            },
            VectorMessage::InsertBefore(i)=>{
                if i<=self.data.len(){
                    self.data.insert(i, Default::default());
                }
            }
            VectorMessage::InsertAfter(i)=>{
                if i+1<=self.data.len(){
                    self.data.insert(i+1, Default::default());
                }
            }
            VectorMessage::Update(i, msg)=>{
                if let Some(v) = self.data.get_mut(i){
                    v.update(msg);
                }
            }

        }
    }
    fn get(&self)->crate::Result<Self::FormType> {
        let mut res = Vec::new();
        for (i,x) in self.data.iter().enumerate(){
            res.push( x.get().map_err(|e| e.map(format!("element #{i}")))?);
        }
        Ok(res)
    }

    fn set(&mut self, value:Self::FormType) {
        let mut value = value;
        self.data.clear();
        for x in value.drain(..){
            self.data.push(T::from_value(x));
        }
    }

    fn from_value(value:Self::FormType) ->Self {
        let mut value = value;
        let mut data = Vec::new();
        for x in value.drain(..){
            data.push(T::from_value(x));
        }
        Self{data}
    }

    fn view<'a>(&'a self,title:Option<&'a str>)->iced::Element<'a,ActionOrUpdate<Self::Message>,iced::Theme> {
        let title = title.unwrap_or_default();
        let header = iced::widget::text(title);
        let mut content = iced::widget::Column::new();
        for (i,x) in self.data.iter().enumerate(){
            let bar = iced::widget::row![
                iced::widget::text(format!("#{}:",i)),
                iced::widget::button("V").on_press(ActionOrUpdate::Update(VectorMessage::MoveDown(i))),
                iced::widget::button("^").on_press(ActionOrUpdate::Update(VectorMessage::MoveUp(i))),
                iced::widget::button("+V").on_press(ActionOrUpdate::Update(VectorMessage::InsertAfter(i))),
                iced::widget::button("+^").on_press(ActionOrUpdate::Update(VectorMessage::InsertBefore(i))),
                iced::widget::button("-").on_press(ActionOrUpdate::Update(VectorMessage::Remove(i))),
            ];
            let pair = iced::widget::column![
                bar,
                x.view(None).map(move |outer| outer.map(move |x| VectorMessage::Update(i, x)))
            ];
            content = content.push(iced::widget::container(pair).style(iced::widget::container::bordered_box));
        }
        let container = iced::widget::container(content.spacing(5))
            .padding(5)
            .style(iced::widget::container::bordered_box);
        iced::widget::column![
            header,
            container,
            //iced::widget::button("+").on_press(VectorMessage::AddNew)
        ].into()
    }
}




impl <T:IcedFormBuffer> Default for VectorBuffer<T>{
    fn default()->Self{
        Self::from_value(Default::default())
    }
}
