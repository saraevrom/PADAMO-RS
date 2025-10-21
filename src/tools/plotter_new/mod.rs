use std::thread::JoinHandle;

use padamo_api::lazy_array_operations::ArrayND;
use padamo_iced_forms::{ActionOrUpdate, IcedFormBuffer};

use crate::{application::PadamoState, messages::PadamoAppMessage};

use super::PadamoTool;
pub mod messages;
pub mod form;
pub mod loader;
pub mod sub_plotter;
pub mod bulk_selector;

#[derive(Copy, PartialEq, Eq, Clone, Debug)]
pub struct SyncDataRequest{
    pub start:usize,
    pub end:usize,
    pub aux_detector_id:Option<usize>
}

pub struct PlotterNew{
    pub state: loader::CurrentData,
    // pub current_view: CurrentView,
    pub form_buffer:form::PlotterFormBuffer,
    pub form: form::PlotterForm,
    pub primary_plotter:sub_plotter::Subplotter,
    pub secondary_plotter:sub_plotter::Subplotter,
    pub last_request:Option<SyncDataRequest>,
}

impl PlotterNew{
    pub fn new()->Self{
        Self {
            state: loader::CurrentData::Idle,
            // current_view: CurrentView::Plots,
            last_request:None,
            form:Default::default(),
            form_buffer:Default::default(),
            primary_plotter: sub_plotter::Subplotter::new(),
            secondary_plotter: sub_plotter::Subplotter::new(),
        }
    }

    fn start_data_reload(&mut self, padamo: &crate::application::PadamoState, order:SyncDataRequest){
        // let aux_detector = self.form.get_aux_id();
        // if (order.end - order.start)<self.form.frames_safeguard+1{
        self.state.pin_to_load(&padamo, order);
        // }
    }

    fn start_data_reload_if_changed(&mut self, padamo: &crate::application::PadamoState, order:SyncDataRequest){
        if Some(order) != self.last_request{
            self.start_data_reload(&padamo, order);
        }
    }

    fn propagate_form(&mut self, padamo:&PadamoState) {
        self.primary_plotter.update_settings(self.form.display_settings.get_primary_settings());
        if let Some(sec) = self.form.display_settings.get_secondary_settings(){
            self.secondary_plotter.update_settings(sec);
        }
        self.secondary_plotter.set_override_range(self.primary_plotter.get_override_range_master());
        padamo.persistent_state.serialize("plotter_form", &self.form);
    }

    fn action_select(&mut self){
        println!("Selecting pixels");
        let workon = match self.form.selector.detector{
            bulk_selector::DetectorSelection::Primary=> &mut self.primary_plotter,
            bulk_selector::DetectorSelection::Secondary=> &mut self.secondary_plotter,
        };
        if let (Some(s), Some(m)) = workon.get_mutable_mask_info(){
            self.form.selector.modify_mask(m, &s.signals);
        }
        workon.clear_cache();
    }
}


impl PadamoTool for PlotterNew{
    fn tab_name(&self)->String {
        "Signal plotter".into()
    }

    fn view<'a>(&'a self, padamo:&'a crate::application::PadamoState)->iced::Element<'a, PadamoAppMessage> {
        let form_view: iced::Element<'a, _>= self.form_buffer.view(None).into();

        let mut plots_column = iced::widget::column![];

        match self.form.current_view{
            form::CurrentView::Plots=>{
                if self.state.is_loading(){
                    plots_column = plots_column.push(iced::widget::container(iced::widget::text("Loading...")).center(iced::Length::Fill));
                }
                else if self.state.is_too_long(){
                    plots_column = plots_column.push(iced::widget::container(iced::widget::text("Too long to load")).center(iced::Length::Fill));
                }
                else{
                    plots_column =  plots_column.push(self.primary_plotter.view().map(messages::NewPlotterMessage::PrimarySubplotterMessage));
                    if self.secondary_plotter.has_data(){
                        plots_column = plots_column.push(self.secondary_plotter.view().map(messages::NewPlotterMessage::SecondarySubplotterMessage))
                    }
                }
            },
            form::CurrentView::ChoosingPrimary=>{
                plots_column = plots_column.push(self.primary_plotter.view_mask().map(messages::NewPlotterMessage::PrimarySubplotterMessage));
            },
            form::CurrentView::ChoosingSecondary=>{
                plots_column = plots_column.push(self.secondary_plotter.view_mask().map(messages::NewPlotterMessage::SecondarySubplotterMessage));
            },
        }



        let plots_column = iced::widget::container(plots_column).width(iced::Length::Fill).height(iced::Length::Fill);

        let mainframe:  iced::Element<'a, messages::NewPlotterMessage> = iced::widget::row!(
            plots_column,
            iced::widget::scrollable(
                form_view.map(messages::NewPlotterMessage::FormMessage)
            ).width(300).height(iced::Length::Fill)
        ).into();

        mainframe.map(PadamoAppMessage::NewPlotterMessage)
    }

    fn update(&mut self, msg: std::rc::Rc<crate::messages::PadamoAppMessage>, padamo:crate::application::PadamoStateRef) {
        if let PadamoAppMessage::Run = msg.as_ref(){
            // self.primary_plotter.set_data(None, None);
            // self.secondary_plotter.set_data(None, None);
            self.last_request = None;
            self.primary_plotter.set_pointer(None);
        }
        else if let PadamoAppMessage::NewPlotterMessage(msg_inner) = msg.as_ref(){
            match msg_inner {
                messages::NewPlotterMessage::FormMessage(form_msg)=>{
                    match form_msg{
                        ActionOrUpdate::Update(upd)=>{
                            self.form_buffer.update(upd.clone());
                            // println!("Form update");
                            match self.form_buffer.get(){
                                Ok(new_v)=> {
                                    // println!("{:?}",new_v);
                                    self.form = new_v;
                                    // self.update_data_state();
                                    self.propagate_form(padamo);
                                },
                                Err(e)=>eprintln!("Form get error: {}",e),
                                // println!("Get OK");
                            }
                        },
                        ActionOrUpdate::Action(action)=>{
                            if let Some(selector_action) = action.downcast_ref::<bulk_selector::SelectorActions>(){
                                match selector_action {
                                    bulk_selector::SelectorActions::Select=>{
                                        self.action_select();
                                    },
                                    // There may be more actions in future
                                }
                            }
                        }
                    }
                    if let Some(mut last_req) = self.last_request{
                        last_req.aux_detector_id = self.form.display_settings.get_aux_id();
                        self.start_data_reload_if_changed(&padamo, last_req);
                    }
                },
                messages::NewPlotterMessage::PrimarySubplotterMessage(subplotter_message) => {
                    self.primary_plotter.update(subplotter_message.clone(), padamo);
                    padamo.persistent_state.serialize("plotter_primary", &self.primary_plotter.get_mutable_detector_info());
                },
                messages::NewPlotterMessage::SecondarySubplotterMessage(subplotter_message) => {
                    self.secondary_plotter.update(subplotter_message.clone(), padamo);
                    padamo.persistent_state.serialize("plotter_secondary", &self.secondary_plotter.get_mutable_detector_info());
                },
                messages::NewPlotterMessage::SyncData{start, end, pointer, poked_pixel} => {
                    let order = SyncDataRequest {
                        start: *start,
                        end: *end,
                        aux_detector_id: self.form.display_settings.get_aux_id()
                    };
                    // println!("{:?} {:?}", self.last_request, order);
                    self.start_data_reload_if_changed(&padamo, order);

                    if let Some(pix) = poked_pixel{
                        if pix.detector_id==0{
                            self.primary_plotter.set_pixel(&pix.pixel_id, true);
                        }
                        if let Some(aux) = self.form.display_settings.get_aux_id(){
                            if pix.detector_id==aux{
                                self.secondary_plotter.set_pixel(&pix.pixel_id, true);
                            }
                        }


                    }
                    if let Some(p) = pointer{
                        //self.primary_plotter.set_pointer();
                        //let x = if self.primary_plotter.
                        self.primary_plotter.set_pointer(Some(*p));
                    }
                },
            }
        }

        self.state.update_state();
        if let Some((data, request)) = self.state.get_data_if_loaded(){
            self.primary_plotter.set_data(Some(data.primary), padamo.detectors.get_primary().map(Clone::clone));
            if let Some(aux) = request.aux_detector_id{
                self.secondary_plotter.set_data(data.secondary, padamo.detectors.get(aux).map(Clone::clone));
            }
            else{
                self.secondary_plotter.set_data(None, None);
            }

            self.secondary_plotter.set_override_range(self.primary_plotter.get_override_range_master());

            self.last_request = Some(request);
            padamo.persistent_state.serialize("plotter_primary", &self.primary_plotter.get_mutable_detector_info());
            padamo.persistent_state.serialize("plotter_secondary", &self.secondary_plotter.get_mutable_detector_info());
        }
    }

    fn late_update(&mut self, msg: std::rc::Rc<crate::messages::PadamoAppMessage>, padamo:crate::application::PadamoStateRef)->Option<crate::messages::PadamoAppMessage> {
        if let PadamoAppMessage::NewPlotterMessage(msg_inner) = msg.as_ref(){
            match msg_inner{
                messages::NewPlotterMessage::PrimarySubplotterMessage(subplotter_message) => {
                    self.primary_plotter.get_late_update_message(subplotter_message)
                },
                messages::NewPlotterMessage::SecondarySubplotterMessage(subplotter_message) => {
                    self.secondary_plotter.get_late_update_message(subplotter_message)
                },
                _ => None,
            }
        }
        else{
            None
        }
    }

    fn context_update(&mut self, _msg: std::rc::Rc<crate::messages::PadamoAppMessage>, padamo:crate::application::PadamoStateRef) {
        self.state.update_state_context(padamo,self.form.frames_safeguard);
    }

    fn initialize(&mut self, padamo:crate::application::PadamoStateRef) {
        if let Some(form) = padamo.persistent_state.deserialize("plotter_form"){
            self.form = form;
            self.propagate_form(&padamo);
            self.form_buffer.set(self.form.clone());
            if let Some((new_det,new_mask)) = padamo.persistent_state.deserialize("plotter_primary"){
                let (a,b) = self.primary_plotter.get_mutable_detector_info();
                *a = new_det;
                *b = new_mask;
            }
            if let Some((new_det,new_mask)) = padamo.persistent_state.deserialize("plotter_secondary"){
                let (a,b) = self.secondary_plotter.get_mutable_detector_info();
                *a = new_det;
                *b = new_mask;
            }
        }
    }
}
