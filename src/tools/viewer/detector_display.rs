use iced::widget::{column, row};
use padamo_api::lazy_array_operations::LazyTriSignal;
use padamo_detectors::DetectorPlotter;

use crate::{application::PadamoState, detector_muxer::get_signal_var, transform_widget::{TransformMessage, TransformState}};
use super::cross_progress::get_icon;

#[derive(Clone, Debug)]
pub enum SingleDetectorDisplayMessage{
    SetDetectorID(usize),
    SetMinSignal(String),
    SetMaxSignal(String),
    SetAutoscale(bool),
    PlotZoomMessage(TransformMessage)
}

pub struct SingleDetectorDisplay<T:Clone>
{
    detector_id: usize,
    is_autoscale:bool,

    min_signal_entry:String,
    max_signal_entry:String,

    buffer:Option<(padamo_api::lazy_array_operations::ndim_array::ArrayND<f64>,f64)>,
    plotter:DetectorPlotter<T>,
    view_transform: TransformState,
    plot_scale:padamo_detectors::Scaling,
}

impl<T:Clone> SingleDetectorDisplay<T>{
    pub fn new(detector_id:usize)->Self{
        Self {
            detector_id,
            plotter: DetectorPlotter::new(),
            buffer:None,
            is_autoscale:true,
            min_signal_entry: String::new(),
            max_signal_entry: String::new(),
            view_transform: TransformState::new(),
            plot_scale: padamo_detectors::Scaling::Autoscale
        }
    }

    pub fn try_get_signal<'a>(&'a self, padamo: &'a PadamoState)->Option<&'a LazyTriSignal>{
        if let Some(padamo_api::prelude::Content::DetectorFullData(signal)) = padamo.compute_graph.environment.0.get(get_signal_var(self.detector_id).as_str()){
            Some(signal)
        }
        else{
            None
        }
    }

    pub fn set_frame(&mut self, frame:usize, padamo: &PadamoState){
        if let Some(signal_tri) = self.try_get_signal(padamo){
            let mut signal = signal_tri.0.request_range(frame, frame+1);
            signal.shape.drain(0..1);
            let time = signal_tri.1.request_range(frame, frame+1)[0];
            self.buffer = Some((signal,time));
            self.fill_strings(padamo);
            self.update_scale();
        }
    }

    fn update_scale(&mut self){
        if self.is_autoscale{
            self.plot_scale = padamo_detectors::Scaling::Autoscale;
        }
        else{
            let min = if let Ok(v) = self.min_signal_entry.parse::<f64>() {v} else {return;};
            let max = if let Ok(v) = self.max_signal_entry.parse::<f64>() {v} else {return;};
            self.plot_scale = padamo_detectors::Scaling::Fixed(min, max);
        }
    }


    pub fn update(&mut self, msg:SingleDetectorDisplayMessage, padamo: &mut PadamoState)->bool{
        match msg{
            SingleDetectorDisplayMessage::SetDetectorID(id)=> {
                self.detector_id = id;
                self.buffer = None;
                return true;
            },
            SingleDetectorDisplayMessage::SetAutoscale(v)=>{
                self.is_autoscale = v;
                self.update_scale();
                //request_buffer_fill = false;
            }
            SingleDetectorDisplayMessage::SetMinSignal(s)=>{
                self.min_signal_entry = s.clone();
                self.update_scale();
                // request_buffer_fill = false;
            }
            SingleDetectorDisplayMessage::SetMaxSignal(s)=>{
                self.max_signal_entry = s.clone();
                self.update_scale();
                // request_buffer_fill = false;
            }
            SingleDetectorDisplayMessage::PlotZoomMessage(msg)=>{
                self.view_transform.update(msg.clone());
            }
        }
        false
    }

    pub fn view<'a, F,F1,F2>(&'a self, padamo:&'a PadamoState, wrapper:F, a1:Option<F1>, a2:Option<F2>)->iced::Element<'a, T>
    where
        F: 'static+Copy+Fn(SingleDetectorDisplayMessage)->T,
        F1:'static+Fn(Vec<usize>)->T,
        F2:'static+Fn(Vec<usize>)->T
    {
        let detector = padamo.detectors.get(self.detector_id);

        let frame = if let Some(buf) = &self.buffer{
            Some((&buf.0, buf.1))
        }
        else{
            None
        };

        let detector_frame = self.plotter.view(detector,frame,self.view_transform.transform(),self.plot_scale,
                        a1,
                        a2,
        );

        let transform:iced::Element<'_, _> = self.view_transform.view().into();
        let footer:iced::Element<'_, _> = row![
            iced::widget::checkbox("Autoscale",self.is_autoscale).on_toggle(SingleDetectorDisplayMessage::SetAutoscale),
            iced::widget::TextInput::new("Min signal", &self.min_signal_entry).width(100).on_input(SingleDetectorDisplayMessage::SetMinSignal),
            iced::widget::text("-").align_x(iced::alignment::Horizontal::Center).width(100),
            iced::widget::TextInput::new("Max signal", &self.max_signal_entry).width(100).on_input(SingleDetectorDisplayMessage::SetMaxSignal),
            iced::widget::Space::new(10,10).width(iced::Length::Fill),
            transform.map(SingleDetectorDisplayMessage::PlotZoomMessage),
            // iced::widget::Space::new(10,10).width(iced::Length::Fill),
        ].into();

        let detector_view = if let Some(det) = padamo.detectors.get(self.detector_id){
            if self.detector_id==0{
                iced::widget::text(format!("Primary detector: {}", det.cells.name))
            }
            else{
                iced::widget::text(format!("Aux detector {}: {}", self.detector_id, det.cells.name))
            }
        }
        else{
            iced::widget::text("No detector")
        };

        let mut left_btn = iced::widget::button("Prev");
        let mut right_btn = iced::widget::button("Next");

        if self.detector_id>0{
            left_btn = left_btn.on_press(SingleDetectorDisplayMessage::SetDetectorID(self.detector_id-1));
        };

        if self.detector_id+1 < padamo.detectors.len(){
            right_btn = right_btn.on_press(SingleDetectorDisplayMessage::SetDetectorID(self.detector_id+1))
        };

        let top = row![
            detector_view,
            iced::widget::Space::new(10,10).width(iced::Length::Fill),
            left_btn,
            right_btn,
        ];
        let top:iced::Element<'_, SingleDetectorDisplayMessage> = top.into();

        let view = column![
            top.map(wrapper),
            detector_frame,
            footer.map(wrapper)
        ];

        view.into()
    }

    pub fn fill_strings(&mut self, padamo:&PadamoState){

        //TODO: Make proper multidetector
        if self.is_autoscale{
            let detector = if let Some(det) = padamo.detectors.get(self.detector_id){det} else {return;};

            if let Some(frame) = &self.buffer{
                let (min,max) = self.plot_scale.get_bounds(&frame.0,&detector.alive_pixels);
                self.min_signal_entry = min.to_string();
                self.max_signal_entry = max.to_string();
            }
        }

    }

    pub fn is_primary(&self)->bool{
        self.detector_id==0
    }

    pub fn pump_frame(&mut self, padamo: &PadamoState, timeline:&super::cross_progress::CrossProgress){
        if let Some(frame) = timeline.get_frame(padamo, self.detector_id){
            self.set_frame(frame, padamo);
        }
        // self.set_frame(, );
    }

    pub fn get_id(&self)->usize{
        self.detector_id
    }

    pub fn get_scale(&self)->padamo_detectors::Scaling{
        self.plot_scale
    }
}
