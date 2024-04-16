use std::fs;

use super::PadamoTool;
use crate::messages::PadamoAppMessage;
use iced::widget;
pub mod messages;
use padamo_detectors::{Detector,DetectorChart,};
use padamo_detectors::polygon::DetectorContent;
use iced_aw::split::Split;

use messages::DetectorManagerMessage;

pub struct PadamoDetectorManager{
    chart:Detector<DetectorManagerMessage>,
    source:widget::text_editor::Content,
    is_dirty:bool,
    split_pos:Option<u16>,
}

impl PadamoDetectorManager {
    pub fn new()->Self{
        let source = widget::text_editor::Content::with_text(&DetectorContent::default_vtl().into_src(Some(1)));
        Self {
            source,
            chart:Detector::default_vtl(),
            is_dirty:false,
            split_pos:None
        }
    }
}

impl PadamoTool for PadamoDetectorManager{
    fn tab_name(&self)->String {
        "Detector manager".into()
    }

    fn view<'a>(&'a self)->iced::Element<'a, crate::messages::PadamoAppMessage> {
        let action:Option<fn(Vec<usize>)->DetectorManagerMessage> = None;
        let frame:iced::Element<DetectorManagerMessage> = widget::row![
                Split::new(
                    widget::container(self.chart.view(None,padamo_detectors::Scaling::Autoscale,action,action)).width(iced::Length::Fill),
                    widget::container(widget::text_editor(&self.source).on_action(DetectorManagerMessage::EditorActionPerformed)).width(iced::Length::Fill),
                    self.split_pos,
                    iced_aw::split::Axis::Vertical,
                    DetectorManagerMessage::SetSplitPosition
                ),
                widget::column![
                    widget::button("Rebuild").on_press(DetectorManagerMessage::Rebuild),
                    widget::button("Export").on_press(DetectorManagerMessage::Export),
                ],

        ].into();
        frame.map(PadamoAppMessage::DetectorManagerMessage)
    }

    fn update(&mut self, msg: std::rc::Rc<PadamoAppMessage>, padamo:crate::application::PadamoStateRef) {
        match msg.as_ref() {
            // PadamoAppMessage::SetDetector(v) => {
            //     self.chart = Detector::from_cells(v.clone());
            // }
            PadamoAppMessage::DetectorManagerMessage(emsg) => {
                match emsg {
                    DetectorManagerMessage::EditorActionPerformed(action)=>{
                        self.is_dirty = self.is_dirty || action.is_edit();
                        self.source.perform(action.clone());
                    },
                    DetectorManagerMessage::SetSplitPosition(v)=>{
                        self.split_pos = Some(*v);
                    },
                    DetectorManagerMessage::Rebuild=>{
                        match DetectorContent::from_specs(&self.source.text()) {
                            Ok(detector)=>{
                                self.chart = Detector::from_cells(detector);
                            }
                            Err(e)=>{
                                padamo.show_error(e.to_string());
                            }
                        }
                    },
                    DetectorManagerMessage::Export=>{
                        if let Some(path) = padamo.workspace.workspace("detectors").save_dialog(vec![("Detector",vec!["json"])]){
                            //if let nfd::Response::Okay(path) = v{
                            let s = serde_json::to_string_pretty(&self.chart.cells) ;
                            let s = match s {
                                Ok(v)=>v,
                                Err(e)=>{
                                    padamo.show_error(format!("{:?}",e));
                                    return;
                                }
                            };
                            match fs::write(path, &s){
                                Ok(_)=>{},
                                Err(e)=>padamo.show_error(format!("{:?}",e)),
                            }
                            //}
                        }
                    },
                }
            }

            _ => (),
        }
    }

    fn context_update(&mut self, msg: std::rc::Rc<crate::messages::PadamoAppMessage>, padamo:crate::application::PadamoStateRef){
        match msg.as_ref(){
            PadamoAppMessage::Save=>{
                if let Some(path) = padamo.workspace.workspace("detector_sources").save_dialog(vec![("Detector source",vec!["dsrc"])]){
                    //if let nfd::Response::Okay(path) = v{
                    match fs::write(path, &self.source.text()){
                        Ok(_)=>{},
                        Err(e)=>padamo.show_error(format!("{:?}",e)),
                    }
                    //}
                }
            },
            PadamoAppMessage::Open=>{
                if let Some(path) = padamo.workspace.workspace("detector_sources").open_dialog(vec![("Detector source",vec!["dsrc"])]){
                    match fs::read_to_string(path){
                        Ok(s)=>{
                            self.source = widget::text_editor::Content::with_text(&s);
                            self.is_dirty = false;
                        }
                        Err(e) => padamo.show_error(format!("{}", e))
                    }
                }

            },
            _=>()
        }
    }
}
