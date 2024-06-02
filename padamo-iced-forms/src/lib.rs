pub mod double_entry_state;

pub trait IcedForm {
    type Interface: IcedFormInterface;
}

pub trait IcedFormInterface {
    type ParentType:IcedForm;
    type MessageType;

    fn new(parent:&Self::ParentType)->Self;
    fn sync_fields(&mut self, parent:&Self::ParentType);
    fn commit_fields(&self, target: &mut Self::ParentType);
    fn submit(&mut self, target:&mut Self::ParentType);
    fn update(&mut self, msg:Self::MessageType, target:&mut Self::ParentType);
    fn view(&self)-> iced::Element<'_,Self::MessageType>;
}
