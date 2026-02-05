use plotters::prelude::*;

use crate::polygon::Detector;

pub fn display_pixel_id<DB:DrawingBackend>(
    detector: &Detector,
    root: &DrawingArea<DB, plotters::coord::Shift>,
    value_source:&dyn super::traits::ColorValueSource,
    state: &super::modular_diagram::PadamoDetectorDiagramState,
    rotation_angle:f64,
) {
    if let Some(index) = detector.position_index(crate::rotate(state.pos,-rotation_angle)){
        let mut unmapped_pos = state.unmapped;
        unmapped_pos.1 -= 20;
        let txt = if let Some(val) = value_source.get_value(index){
            format!("{:?} {:.3}",index,val)
        }
        else{
            format!("{:?}",index)
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
        let matching = if let Some(s) = self.state{
            s.0==mouse_button
        }
        else{
            true
        };

        if matching{
            self.state = Some((mouse_button, position));
            // println!("CLICK {:?}", mouse_button);
        }
    }

    pub fn release(&mut self, mouse_button:iced::mouse::Button)->Option<(f64,f64)>{
        if let Some((btn, pos)) = self.state.take(){
            if btn==mouse_button{

                // println!("CLICK END {:?}", mouse_button);
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

    pub fn is_active(&self)->bool{
        self.state.is_some()
    }

    pub fn reset(&mut self){
        // println!("CLICK RESET");
        self.state = None;
    }
}
