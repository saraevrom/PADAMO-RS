use std::str::FromStr;
use iced::widget::row;

pub fn errored_text(disp:&str,is_ok:bool)->iced::widget::Text<'_,iced::theme::Theme>{
    let mut txt:iced::widget::Text<'_,iced::theme::Theme> = iced::widget::text(disp);
    if !is_ok{
        txt = txt.style(|_| iced::widget::text::Style{color: Some(iced::Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0})});
    }
    txt
}

pub fn errored_entry<'a,M:Clone,F:Fn(String)->M+'a>(placeholder:&'a str,value:&'a str,is_ok:bool, msg:F)->iced::widget::TextInput<'a,M>{
    use iced::widget::text_input;
    let mut entry = text_input(placeholder, value).on_input(msg);
    if !is_ok{
        entry = entry.style(|t,s| text_input::Style{
            value: iced::Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0},..text_input::default(t,s)});
    }
    entry
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

    pub fn view<'a,Message:Clone+'a, F:Fn(String)->Message+'a> (&'a self,placehoder:&'a str, msg:F)->iced::widget::TextInput<'a,Message>{
        errored_entry(placehoder, &self.string_value, self.is_valid, msg)
        //iced::widget::text_input(placehoder,&self.string_value).on_input(msg)
    }

    pub fn view_label<'a>(&'a self, label:&'a str)->iced::widget::Text<'_,iced::theme::Theme>{
        errored_text(label,self.is_valid)
    }

    pub fn view_row<'a,Message:Clone+ 'a, F:Fn(String)->Message+'a>(&'a self,label:&'a str, placehoder:&'a str, msg:F)->iced::widget::Row<'a,Message>{
        row![
            self.view_label(label),
            self.view(placehoder, msg)
        ]
    }
}

impl<T:ToString+FromStr+Default> Default for EntryState<T>{
    fn default()->Self{
        Self::new(T::default())
    }
}
