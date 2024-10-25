use iced::widget;
use padamo_iced_forms::double_entry_state::EntryState;

#[derive(Clone,Debug, Default)]
pub struct TransformState{
    // pub transform: crate::Transform,
    delta_x:EntryState<f64>,
    delta_y:EntryState<f64>,
    zoom:EntryState<f64>,

}

#[derive(Clone,Debug)]
pub enum TransformMessage{
    MoveX(String),
    MoveY(String),
    SetZoom(String),
}

impl TransformState{
    pub fn update(&mut self, msg:TransformMessage){
        match msg {
            TransformMessage::MoveX(x)=>self.delta_x.set_string(x),
            TransformMessage::MoveY(y)=>self.delta_y.set_string(y),
            TransformMessage::SetZoom(z)=>self.zoom.set_string(z),
        }
    }

    pub fn view(&self)->iced::Element<'_, TransformMessage>{
        widget::Container::new(
        widget::row![
            self.delta_x.view_row("X", "0.0", TransformMessage::MoveX),
            self.delta_y.view_row("Y", "0.0", TransformMessage::MoveX),
            self.zoom.view_row("Z", "1.0", TransformMessage::SetZoom)
        ]).width(400).into()
    }

    pub fn transform(&self)->padamo_detectors::Transform{
        padamo_detectors::Transform::new(self.zoom.parsed_value, self.delta_x.parsed_value, self.delta_y.parsed_value)
    }
}
