use plotters::prelude::*;

use crate::polygon::DetectorContent;

pub fn display_pixel_id<DB:DrawingBackend>(
    detector: &DetectorContent,
    root: &DrawingArea<DB, plotters::coord::Shift>,
    value_source:&dyn super::traits::ColorValueSource,
    state: &super::modular_diagram::PadamoDetectorDiagramState
) {
    if let Some(index) = detector.position_index(state.pos){
        let mut unmapped_pos = state.unmapped;
        unmapped_pos.1 -= 20;
        let txt = if let Some(val) = value_source.get_value(index){
            format!("{:?} {:.3}",index,val)
        }
        else{
            format!("{:?} MAPPING INVALID",index)
        };
        root.draw_text(&txt, &(("sans-serif", 15).into()), unmapped_pos).unwrap();
    }
}

#[derive(Default)]
pub struct ClickTracker{
    state:Option<(iced::mouse::Button, (f64,f64))>,
}

impl ClickTracker{
    pub fn new()->Self{
        Self { state: None }
    }

    pub fn click(&mut self, mouse_button:iced::mouse::Button, position:(f64,f64)){
        if self.state.is_none(){
            self.state = Some((mouse_button, position));
        }
    }

    pub fn release(&mut self, mouse_button:iced::mouse::Button)->Option<(f64,f64)>{
        if let Some((btn, pos)) = self.state.take(){
            if btn==mouse_button{
                Some(pos)
            }
            else{
                self.state = Some((btn, pos));
                None
            }
        }
        else{
            None
        }
    }

    pub fn get_state(&self)->Option<(iced::mouse::Button, (f64,f64))>{
        self.state
    }

    pub fn reset(&mut self){
        self.state = None;
    }
}
