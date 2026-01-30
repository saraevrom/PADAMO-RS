use std::collections::HashMap;
use padamo_api::lazy_array_operations::ArrayND;

use crate::application::PadamoState;

#[derive(Clone, Debug,serde::Serialize,serde::Deserialize)]
pub struct NormEntryState{
    min_signal_entry:String,
    max_signal_entry:String,
    is_autoscale: bool,
    plot_scale:padamo_detectors::Scaling,
}

#[derive(Clone,Debug)]
pub enum NormEntryMessage{
    SetMinSignal(String),
    SetMaxSignal(String),
    SetAutoscale(bool),
}

impl NormEntryState{
    pub fn new()->Self{
        Self {
            min_signal_entry: String::new(),
            max_signal_entry: String::new(),
            is_autoscale: true,
            plot_scale:padamo_detectors::Scaling::Autoscale,
        }
    }

    pub fn fill_strings(&mut self, _padamo:&PadamoState, ref_frame:&ArrayND<f64>, ref_mask:&ArrayND<bool>){
        if self.is_autoscale{
            // let detector = if let Some(det) = padamo.detectors.get(self.detector_id){det} else {return;};
            // if let Some(frame) = &self.buffer{
            let (min,max) = self.plot_scale.get_bounds(ref_frame, ref_mask);
            self.min_signal_entry = min.to_string();
            self.max_signal_entry = max.to_string();
            // }
        }
    }

    pub fn update_scale(&mut self){
        if self.is_autoscale{
            self.plot_scale = padamo_detectors::Scaling::Autoscale;
        }
        else{
            let min = if let Ok(v) = self.min_signal_entry.parse::<f64>() {v} else {return;};
            let max = if let Ok(v) = self.max_signal_entry.parse::<f64>() {v} else {return;};
            self.plot_scale = padamo_detectors::Scaling::Fixed(min, max);
        }
    }

    pub fn update(&mut self, message:NormEntryMessage){
        match message{
            NormEntryMessage::SetAutoscale(v)=>{
                self.is_autoscale = v;
                self.update_scale();
                //request_buffer_fill = false;
            }
            NormEntryMessage::SetMinSignal(s)=>{
                self.min_signal_entry = s.clone();
                self.update_scale();
                // request_buffer_fill = false;
            }
            NormEntryMessage::SetMaxSignal(s)=>{
                self.max_signal_entry = s.clone();
                self.update_scale();
                // request_buffer_fill = false;
            }
        }
    }

    pub fn view(&self)->iced::Element<'_,NormEntryMessage>{
        iced::widget::row![
            iced::widget::checkbox(self.is_autoscale).label("Autoscale").on_toggle(NormEntryMessage::SetAutoscale),
            iced::widget::TextInput::new("Min signal", &self.min_signal_entry).width(100).on_input(NormEntryMessage::SetMinSignal),
            iced::widget::text("-").align_x(iced::alignment::Horizontal::Center).width(20),
            iced::widget::TextInput::new("Max signal", &self.max_signal_entry).width(100).on_input(NormEntryMessage::SetMaxSignal),
        ].into()
    }

    pub fn get_scale(&self)->padamo_detectors::Scaling{
        self.plot_scale
    }
}

#[derive(Clone, Debug,serde::Serialize,serde::Deserialize)]
pub struct MultiEntry{
    pub stash: HashMap<usize, NormEntryState>,
    pub lock: Option<usize>,
}

#[derive(Clone,Debug)]
pub enum MultiEntryMessage{
    NormEntryMessage(NormEntryMessage),
    SetLock(bool)
}


impl MultiEntry{
    pub fn new()->Self{
        Self { stash: HashMap::new(), lock:None }
    }

    pub fn get_entry_mut(&mut self, id:usize) -> &mut NormEntryState{
        let id = if let Some(o) = self.lock {o} else {id};
        self.stash.entry(id).or_insert(NormEntryState::new())
    }

    pub fn get_entry(&self, id:usize) -> Option<&NormEntryState>{
        let id = if let Some(o) = self.lock {o} else {id};
        self.stash.get(&id)
    }

    pub fn view(&self, id:usize) -> iced::Element<'_, MultiEntryMessage>{
        let norm: iced::Element<'_, MultiEntryMessage> = if let Some(elem) = self.get_entry(id){
            elem.view().map(MultiEntryMessage::NormEntryMessage)
        }
        else{
            iced::widget::text("---No normalization---").into()
        };
        iced::widget::row![
            iced::widget::checkbox(self.lock.is_some()).label("Lock").on_toggle(MultiEntryMessage::SetLock),
            iced::widget::Space::new(),
            norm,
        ].into()
    }

    pub fn update(&mut self, msg:MultiEntryMessage, id:usize){
        match msg{
            MultiEntryMessage::NormEntryMessage(msg1)=>{
                self.get_entry_mut(id).update(msg1);
            }
            MultiEntryMessage::SetLock(v)=>{
                if v{
                    self.lock = Some(id);
                }
                else{
                    self.lock = None;
                }
            }
        }
    }

}
