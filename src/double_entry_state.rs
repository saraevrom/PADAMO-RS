use std::str::FromStr;
use iced::widget::row;

pub fn errored_text(disp:&str,is_ok:bool)->iced::widget::Text<'_,iced::theme::Theme>{
    let mut txt:iced::widget::Text<'_,iced::theme::Theme> = iced::widget::text(disp);
    if !is_ok{
        txt = txt.style(iced::theme::Text::Color(iced::Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0}));
    }
    txt
}

#[derive(Clone,Debug)]
pub struct EntryState<T:ToString+FromStr>{
    pub parsed_value:T,
    pub string_value:String,
    pub is_valid:bool
}

impl<T:ToString+FromStr> EntryState<T>{
    pub fn new(v:T)->Self{
        Self {string_value: v.to_string(),parsed_value: v, is_valid: true }
    }

    pub fn set_string(&mut self, s:String){
        if let Ok(v) = s.parse(){
            self.parsed_value = v;
            self.is_valid = true;
        }
        else{
            self.is_valid = false;
        }
        self.string_value = s;
    }

    pub fn set_value(&mut self, v:T){
        self.string_value = v.to_string();
        self.parsed_value = v;
        self.is_valid = true;
    }

    pub fn view<'a,Message:Clone+'a, F:Fn(String)->Message+'a> (&'a self,placehoder:&str, msg:F)->iced::widget::TextInput<'a,Message>{
        iced::widget::text_input(placehoder,&self.string_value).on_input(msg)
    }

    pub fn view_label<'a>(&'a self, label:&'a str)->iced::widget::Text<'_,iced::theme::Theme>{
        errored_text(label,self.is_valid)
    }

    pub fn view_row<'a,Message:Clone+ 'a, F:Fn(String)->Message+'a>(&'a self,label:&'a str, placehoder:&str, msg:F)->iced::widget::Row<'a,Message>{
        row![
            self.view_label(label),
            self.view(placehoder, msg)
        ]
    }
}

