use std::fs;

use super::PadamoTool;
use crate::application::PadamoState;
use crate::messages::PadamoAppMessage;
use iced::widget;
pub mod messages;
use padamo_detectors::{DetectorPlotter, DetectorAndMask};
use padamo_detectors::polygon::DetectorContent;
use iced::widget::pane_grid;

use messages::DetectorManagerMessage;

#[derive(Clone, Copy)]
pub enum Pane{
    DetectorView,
    SourceCode
}

pub struct PadamoDetectorManager{
    chart:DetectorPlotter<DetectorManagerMessage>,
    detector:DetectorAndMask,
    source:widget::text_editor::Content,
    is_dirty:bool,
    //split_pos:Option<u16>,
    panes: pane_grid::State<Pane>,
    viewer_transform:crate::transform_widget::TransformState
}

impl PadamoDetectorManager {
    pub fn new()->Self{
        //let source = widget::text_editor::Content::with_text(&DetectorContent::default_vtl().into_src(Some(1)));
        let source = widget::text_editor::Content::with_text(include_str!("tuloma_source.dsrc"));
        let (mut panes, start) = pane_grid::State::new(Pane::DetectorView);
        panes.split(pane_grid::Axis::Vertical, start, Pane::SourceCode);
        Self {
            source,
            chart:DetectorPlotter::new(),
            detector:DetectorAndMask::default_vtl(),
            is_dirty:false,
            //split_pos:None,
            panes,
            viewer_transform:Default::default(),
        }
    }
}

impl PadamoTool for PadamoDetectorManager{
    fn tab_name(&self)->String {
        "Detector manager".into()
    }

    fn view<'a>(&'a self, padamo:&PadamoState)->iced::Element<'a, crate::messages::PadamoAppMessage> {
        let action:Option<fn(Vec<usize>)->DetectorManagerMessage> = None;
        // let frame:iced::Element<DetectorManagerMessage> = widget::row![
        //         Split::new(
        //             widget::container(self.chart.view(None,padamo_detectors::Scaling::Autoscale,action,action)).width(iced::Length::Fill),
        //             widget::container(widget::text_editor(&self.source).on_action(DetectorManagerMessage::EditorActionPerformed)).width(iced::Length::Fill),
        //             self.split_pos,
        //             iced_aw::split::Axis::Vertical,
        //             DetectorManagerMessage::SetSplitPosition
        //         ),
        //         widget::column![
        //             widget::button("Rebuild").on_press(DetectorManagerMessage::Rebuild),
        //             widget::button("Export").on_press(DetectorManagerMessage::Export),
        //         ],
        //
        // ].into();
        //todo!()
        let subframe = iced::widget::PaneGrid::new(&self.panes,|_id, pane, _is_maximized| {
            match pane{
                Pane::DetectorView=>{
                    let controls:iced::Element<'_,_> = self.viewer_transform.view().into();
                    widget::container(iced::widget::column![
                        self.chart.view(Some(&self.detector),None, self.viewer_transform.transform(),padamo_detectors::Scaling::Autoscale,action,action,None),
                        controls.map(DetectorManagerMessage::PlotZoomMessage),
                    ]).width(iced::Length::Fill).into()
                }
                Pane::SourceCode=>{
                    widget::container(widget::text_editor(&self.source).on_action(DetectorManagerMessage::EditorActionPerformed)).width(iced::Length::Fill).into()
                }
            }
        }).on_drag(DetectorManagerMessage::PaneDrag).on_resize(10, DetectorManagerMessage::PaneResize);
        let frame:iced::Element<DetectorManagerMessage> = widget::row![
                subframe,
                widget::column![
                    widget::button("Rebuild").on_press(DetectorManagerMessage::RebuildMarkup),
                    widget::button("Rebuild (Rhai)").on_press(DetectorManagerMessage::RebuildScript),
                    widget::button("Export").on_press(DetectorManagerMessage::Export),
                ],

        ].into();
        //todo!()
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
                    // DetectorManagerMessage::SetSplitPosition(v)=>{
                    //     self.split_pos = Some(*v);
                    // },
                    DetectorManagerMessage::PaneDrag(pane_grid::DragEvent::Dropped {pane, target})=>{
                        self.panes.drop(*pane, *target);
                    }
                    DetectorManagerMessage::PaneResize(pane_grid::ResizeEvent { split, ratio }) => {
                        self.panes.resize(*split, *ratio);
                    }
                    DetectorManagerMessage::RebuildMarkup=>{
                        match DetectorContent::from_specs(&self.source.text()) {
                            Ok(detector)=>{
                                self.detector= DetectorAndMask::from_cells(detector);
                            }
                            Err(e)=>{
                                padamo.show_error(e.to_string());
                            }
                        }
                    },
                    DetectorManagerMessage::RebuildScript=>{
                        match DetectorContent::from_script(&self.source.text()) {
                            Ok(detector)=>{
                                self.detector = DetectorAndMask::from_cells(detector);
                            }
                            Err(e)=>{
                                padamo.show_error(e.to_string());
                            }
                        }
                    }
                    DetectorManagerMessage::Export=>{
                        if let Some(path) = padamo.workspace.workspace("detectors").save_dialog(vec![("Detector",vec!["json"])]){
                            //if let nfd::Response::Okay(path) = v{
                            let s = serde_json::to_string_pretty(&self.detector.cells) ;
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
                    DetectorManagerMessage::PlotZoomMessage(msg)=>{
                        self.viewer_transform.update(msg.to_owned());
                    }
                    _=>(),
                }
            }

            _ => (),
        }
    }

    fn context_update(&mut self, msg: std::rc::Rc<crate::messages::PadamoAppMessage>, padamo:crate::application::PadamoStateRef){
        match msg.as_ref(){
            PadamoAppMessage::Save=>{
                if let Some(path) = padamo.workspace.workspace("detector_sources").save_dialog(vec![("Detector source",vec!["dsrc","rhai"])]){
                    //if let nfd::Response::Okay(path) = v{
                    match fs::write(path, &self.source.text()){
                        Ok(_)=>{},
                        Err(e)=>padamo.show_error(format!("{:?}",e)),
                    }
                    //}
                }
            },
            PadamoAppMessage::Open=>{
                if let Some(path) = padamo.workspace.workspace("detector_sources").open_dialog(vec![("Detector source",vec!["dsrc","rhai"])]){
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
