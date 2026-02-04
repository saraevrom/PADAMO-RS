use iced::widget::{column, row};
use padamo_api::{lazy_array_operations::LazyTriSignal, prelude::{make_lao_box, Content}};
use padamo_detectors::mesh::Mesh;
use plotters::style::Color;

use crate::{application::PadamoState, detector_muxer::get_signal_var, transform_widget::{TransformMessage, TransformState}};
use super::norm_entry::{MultiEntry, MultiEntryMessage};

#[derive(Clone, Debug)]
pub enum SingleDetectorDisplayMessage{
    SetDetectorID(usize),
    // SetMinSignal(String),
    // SetMaxSignal(String),
    // SetAutoscale(bool),
    NormEntryMessage(MultiEntryMessage),
    PlotZoomMessage(TransformMessage),
    TogglePixel(Vec<usize>),
    ResetMask,
}

pub struct SingleDetectorDisplay
{
    detector_id: usize,
    // is_autoscale:bool,
    //
    // min_signal_entry:String,
    // max_signal_entry:String,
    scale_state:MultiEntry,

    buffer:Option<(padamo_api::lazy_array_operations::ndim_array::ArrayND<f64>,f64)>,
    view_transform: TransformState,
}

impl SingleDetectorDisplay{
    pub fn new(detector_id:usize)->Self{
        Self {
            detector_id,
            buffer:None,
            scale_state: MultiEntry::new(),
            view_transform: TransformState::new()
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
            self.scale_state.get_entry_mut(self.detector_id).update_scale();
        }
    }

    pub fn update_pixels(&self, padamo :&mut PadamoState, save:bool){
        // let detector = if let Some(det) = self.get_detector(padamo){det} else {return;};
        let detector_entry = if let Some(det) = padamo.detectors.get(self.get_id()){det} else {return;};
        let mask = detector_entry.detector.alive_pixels_mask();
        if save{
            padamo.save_detectors();
        }
        padamo.compute_graph.environment.0.insert("alive_pixels".into(),Content::DetectorSignal(make_lao_box(mask)));
        //let arr = self.chart.alive_pixels.clone().to_ndarray();

    }

    pub fn update(&mut self, msg:SingleDetectorDisplayMessage, padamo: &mut PadamoState)->bool{
        match msg{
            SingleDetectorDisplayMessage::SetDetectorID(id)=> {
                self.detector_id = id;
                self.buffer = None;
                return true;
            },
            SingleDetectorDisplayMessage::NormEntryMessage(msg)=>{
                self.scale_state.update(msg, self.detector_id);
                padamo.persistent_state.serialize("viewer_color_norm", &self.scale_state);
            },
            SingleDetectorDisplayMessage::PlotZoomMessage(msg)=>{
                self.view_transform.update(msg.clone());
            },
            SingleDetectorDisplayMessage::TogglePixel(pix_id)=>{
                if let Some(detector_info) = padamo.detectors.get_mut(self.detector_id){
                    detector_info.detector.toggle_pixel(&pix_id);
                    self.update_pixels(padamo, true);
                }
            },
            SingleDetectorDisplayMessage::ResetMask=>{
                if let Some(detector_info) = padamo.detectors.get_mut(self.detector_id){
                    detector_info.detector.reset_mask();
                    self.update_pixels(padamo, true);
                }
            }
        }
        false
    }

    pub fn view<'a, T:Clone+'a, F,F1>(&'a self, padamo:&'a PadamoState, wrapper:F, a1:Option<F1>, mesh_data:Option<(&'a Mesh, nalgebra::Matrix4<f64>)>)->iced::Element<'a, T>
    where
        F: 'static+Copy+Fn(SingleDetectorDisplayMessage)->T,
        F1:'static+Fn(Vec<usize>)->T,
    {
        let detector = padamo.detectors.get(self.detector_id).map(|x| &x.detector);

        let a2 = move |x| wrapper(SingleDetectorDisplayMessage::TogglePixel(x));

        let color_source = padamo_detectors::diagrams::autoselect_source(detector, self.buffer.as_ref().map(|x| &x.0), self.get_scale());
        let mut detector_frame = padamo_detectors::diagrams::PadamoDetectorDiagram::new(detector.map(|x| &x.cells), color_source)
            .transformed(self.view_transform.transform())
            .on_right_click(a2);

        if let Some(a) = a1{
            detector_frame = detector_frame.on_left_click(a);
        }

        if let Some(m) = mesh_data{
            detector_frame = detector_frame.with_mesh(m.0, m.1, plotters::prelude::RED.filled());
        }

        if let Some(time) = self.buffer.as_ref().map(|x| &x.1){
            detector_frame = detector_frame.with_title_unixtime(*time);
        }

        let detector_frame = detector_frame.view();

        let transform:iced::Element<'_, _> = self.view_transform.view().into();
        let footer:iced::Element<'_, _> = row![
            self.scale_state.view(self.detector_id).map(SingleDetectorDisplayMessage::NormEntryMessage),
            iced::widget::Space::new(),
            transform.map(SingleDetectorDisplayMessage::PlotZoomMessage),
            // iced::widget::Space::new(10,10).width(iced::Length::Fill),
        ].height(iced::Length::Shrink).into();

        let detector_view = if let Some(det) = padamo.detectors.get(self.detector_id){
            if self.detector_id==0{
                iced::widget::text(format!("Primary detector: {}", det.detector.cells.name))
            }
            else{
                iced::widget::text(format!("Aux detector {}: {}", self.detector_id, det.detector.cells.name))
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
            iced::widget::Space::new().width(iced::Length::Fill),
            left_btn,
            right_btn,
            iced::widget::button("Reset mask").on_press(SingleDetectorDisplayMessage::ResetMask)
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
        // if self.is_autoscale{
        //     let detector = if let Some(det) = padamo.detectors.get(self.detector_id){det} else {return;};
        //
        //     if let Some(frame) = &self.buffer{
        //         let (min,max) = self.plot_scale.get_bounds(&frame.0,&detector.alive_pixels);
        //         self.min_signal_entry = min.to_string();
        //         self.max_signal_entry = max.to_string();
        //     }
        // }
        let detector = if let Some(det) = padamo.detectors.get(self.detector_id){&det.detector} else {return;};
        if let Some(frame) = &self.buffer{
            self.scale_state.get_entry_mut(self.detector_id).fill_strings(padamo, &frame.0, &detector.alive_pixels);
        }


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
        self.scale_state.get_entry(self.detector_id).map(|x| x.get_scale()).unwrap_or(padamo_detectors::Scaling::Autoscale)
    }

    pub fn initialize(&mut self, padamo:&PadamoState){
        if let Some(v) = padamo.persistent_state.deserialize("viewer_color_norm"){
            self.scale_state = v;
        }
    }
}
