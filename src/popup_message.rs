use std::collections::VecDeque;

use iced::widget;
use iced_aw::card;
use crate::messages::PadamoAppMessage;

#[derive(Clone,Copy,Debug)]
pub enum PadamoPopupMessageType{
    Info,
    Warning,
    Error,
}

impl PadamoPopupMessageType{
    pub fn title(&self)->&'static str{
        match self {
            PadamoPopupMessageType::Info => "Info",
            PadamoPopupMessageType::Warning => "Warning",
            PadamoPopupMessageType::Error => "Error",
        }
    }
}

#[derive(Clone,Debug)]
pub struct PadamoPopupMessage{
    message_type:PadamoPopupMessageType,
    message:String
}

impl PadamoPopupMessage{
    pub fn new(message_type: PadamoPopupMessageType, message: String) -> Self {
        Self { message_type, message }
    }

    pub fn view<'a>(&'a self)->iced::Element<'a, PadamoAppMessage>{
        card::Card::new(widget::text(self.message_type.title()), widget::text(&self.message))
            .foot(
                widget::container(
                    widget::button("OK").width(100).on_press(PadamoAppMessage::PopupMessageClick)
                ).width(iced::Length::Fill).align_x(iced::alignment::Horizontal::Center)
            )
            .on_close(PadamoAppMessage::PopupMessageClick)
            .max_width(500.0)
            .max_height(250.0)
            .into()
    }
}

pub struct MessageList{
    container:VecDeque<PadamoPopupMessage>
}


impl MessageList{
    pub fn new()->Self{
        Self { container: VecDeque::new() }
    }

    pub fn oldest_message<'a>(&'a self)->Option<&'a PadamoPopupMessage>{
        if self.container.len()>0{
            Some(&self.container[0])
        }
        else{
            None
        }
    }

    pub fn add_message(&mut self, msg:String, level:PadamoPopupMessageType){
        self.container.push_back(PadamoPopupMessage::new(level, msg));
    }

    pub fn pop_oldest_message(&mut self){
        self.container.pop_front();
    }
}
