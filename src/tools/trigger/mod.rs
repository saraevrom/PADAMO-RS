use std::{thread, sync::mpsc};

use crate::messages::PadamoAppMessage;

use self::sparse_intervals::{IntervalStorage, Interval, BinaryUnixIntervalStorage};

use super::PadamoTool;
use chrono::{Datelike, Timelike};
use iced::{widget, Font};
use padamo_api::lazy_array_operations::ArrayND;
use padamo_detectors::Detector;
pub mod messages;
use messages::TriggerMessage;
//use padamo_iced_forms::IcedForm;

pub mod sparse_intervals;
pub mod interval_selector;
use interval_selector::IntervalSelectionDialog;
// use padamo_iced_forms_derive::IcedForm;
// use padamo_iced_forms::IcedFormInterface;
use padamo_iced_forms::{IcedForm,IcedFormBuffer};
use crate::tools::viewer::Worker;
use sparse_intervals::split_intervals;
use iced_aw::selection_list;
use super::plotter::{spawn_loader,data_state::DataCache, get_maxes};

use std::fs;

pub enum TriggerProcessMessage{
    Status(String),
    MarkPositive(sparse_intervals::Interval),
    MarkNegative(sparse_intervals::Interval),
}

pub enum ExportProcessMessage{
    Status(String)
}

pub struct PadamoTrigger{
    chart:Detector<TriggerMessage>,
    signal:Option<padamo_api::lazy_array_operations::LazyTriSignal>,
    //buffer:Option<(padamo_api::lazy_array_operations::ndim_array::ArrayND<f64>,f64)>,
    unmarked_intervals:sparse_intervals::IntervalStorage,
    positive_intervals:sparse_intervals::IntervalStorage,
    negative_intervals:sparse_intervals::IntervalStorage,

    positive_strings:Vec<String>,
    negative_strings:Vec<String>,

    selection:Option<usize>,
    selected_interval:Option<Interval>,
    selection_positive:bool,
    //negative_select:Option<usize>,

    //interval_storage: Arc<Mutex<IntervalsContainer>>,
    trigger_interval_selector:Option<IntervalSelectionDialog>,
    // trigger_form:TriggerSettingsForm,
    trigger_form_buffer:TriggerSettingsFormBuffer,
    trigger_form_instance:TriggerSettingsForm,
    trigger_process:Option<Worker<TriggerProcessMessage>>,
    trigger_status:String,

    export_process:Option<Worker<ExportProcessMessage>>,
    export_status:String,

    loader:Option<thread::JoinHandle<DataCache>>,
    data:Option<(DataCache,ArrayND<f64>)>,
    view_transform: crate::transform_widget::TransformState,
}



#[derive(Clone,Debug,IcedForm)]
pub struct TriggerSettingsForm{
    #[field_name("Chunk size")] pub chunksize:usize,
    #[field_name("Safeguard [frames]")] pub safeguard:usize,
}

impl Default for TriggerSettingsForm{
    fn default()->Self{
        Self { chunksize: 10000, safeguard:3000 }
    }

}


impl PadamoTrigger{
    pub fn new()->Self{
        // let trigger_form = TriggerSettingsForm::new();
        Self {
            chart:Detector::default_vtl(),
            signal:None,
            view_transform:Default::default(),
            //buffer:None,
            //interval_storage:Arc::new(Mutex::new(IntervalsContainer::new())),
            unmarked_intervals:IntervalStorage::new_full(100),
            positive_intervals:IntervalStorage::new_empty(),
            negative_intervals:IntervalStorage::new_empty(),
            positive_strings:vec![],
            negative_strings:vec![],

            selection:None,
            selected_interval:None,
            selection_positive:true,

            trigger_interval_selector:None,
            trigger_form_buffer: Default::default(),
            trigger_form_instance: Default::default(),
            // trigger_form,
            trigger_process:None,
            export_process:None,
            trigger_status:"IDLE".into(),
            export_status:"IDLE".into(),
            loader:None,
            data:None

        }
    }

    pub fn reset_intervals(&mut self, length:usize){
        self.unmarked_intervals = IntervalStorage::new_full(length);
        self.positive_intervals = IntervalStorage::new_empty();
        self.negative_intervals = IntervalStorage::new_empty();

        self.selection = None;
        self.selected_interval = None;
        self.selection_positive = true;
        self.data = None;
        self.update_interval_strings();
    }

    pub fn stop_worker(&mut self){
        if let Some(mut a) = self.trigger_process.take(){
            a.stop();
        }
        self.trigger_status = "IDLE".into();
    }

    pub fn stop_export(&mut self){
        if let Some(mut a) = self.export_process.take(){
            a.stop();
        }
        self.export_status = "IDLE".into();
    }

    pub fn mark_positive(&mut self,interval:Interval){
        //println!("Trying to insert positive {}",interval);
        if self.unmarked_intervals.take_interval(interval){
            //println!("TRIGGERED {}",interval);
            self.positive_intervals.insert_interval(interval);
        }
        //self.unmarked_intervals.print_contents();
    }


    pub fn mark_negative(&mut self,interval:Interval){
        //println!("Trying to insert negative {}",interval);
        if self.unmarked_intervals.take_interval(interval){
            //println!("NOT TRIGGERED {}",interval);
            self.negative_intervals.insert_interval(interval);
        }
        //self.unmarked_intervals.print_contents();
    }

    pub fn update_interval_strings(&mut self){
        self.positive_strings = self.positive_intervals.container.iter().map(|x| format!("{}",x)).collect();
        self.negative_strings = self.negative_intervals.container.iter().map(|x| format!("{}",x)).collect();
    }

    fn select_event(&mut self){
        let trigger_form = &self.trigger_form_instance;
        if let Some(signal) = &self.signal{
            if let Some(sel) = self.selection{

                let interval = if self.selection_positive{
                    println!("Selected positive {}",sel);
                    self.positive_intervals.container[sel]
                }
                else{
                    println!("Selected negative {}",sel);
                    self.negative_intervals.container[sel]
                };
                self.selected_interval = Some(interval);
                if interval.length()>trigger_form.safeguard{
                    return;
                }
                if self.loader.is_some(){
                    return;
                }
                let spatial = signal.0.clone();
                let temporal = signal.1.clone();
                self.loader = Some(spawn_loader(spatial, temporal, interval.start, interval.end));
            }
        }
    }
}


impl PadamoTool for PadamoTrigger{
    fn tab_name(&self)->String {
        "Trigger".into()
    }

    fn view<'a>(&'a self)->iced::Element<'a, crate::messages::PadamoAppMessage> {
        let action:Option<fn(Vec<usize>)->TriggerMessage> = None;
        let positive_select = if self.selection_positive{
            self.selection
        }
        else{
            None
        };

        let negative_select = if !self.selection_positive{
            self.selection
        }
        else{
            None
        };

        let view_content = if let Some(v) = &self.data{
            Some((&v.1,v.0.time[0]))
        }
        else{
            None
        };

        let view_transform:iced::Element<'_,_> = self.view_transform.view().width(iced::Length::Fill).into();
        let underlay:iced::Element<'_, TriggerMessage> = widget::row![
            widget::column![
                widget::container(selection_list::SelectionList::new_with(
                    &self.positive_strings,
                    TriggerMessage::SelectPositive,
                    12.0,
                    5.0,
                    iced_aw::style::selection_list::primary,
                    positive_select,
                    Font::default(),
                )).height(iced::Length::FillPortion(2)).width(iced::Length::Fill),
                widget::container(selection_list::SelectionList::new_with(
                    &self.negative_strings,
                    TriggerMessage::SelectNegative,
                    12.0,
                    5.0,
                    iced_aw::style::selection_list::primary,
                    negative_select,
                    Font::default(),
                )).height(iced::Length::FillPortion(2)).width(iced::Length::Fill)
            ].width(250),

            //widget::container(
            widget::column![
                self.chart.view(view_content,self.view_transform.transform(),padamo_detectors::Scaling::Autoscale,action,action),
                view_transform.map(TriggerMessage::PlotZoomMessage)
            ].width(iced::Length::Fill),

            //).width(iced::Length::Fill),

            widget::scrollable(widget::column![
                widget::row![
                    widget::button("Trigger interval").on_press(TriggerMessage::ChooseTrigger),
                    widget::button("Stop").on_press(TriggerMessage::Stop),
                ],
                widget::button("Focus on event").on_press(TriggerMessage::ExamineEvent),
                widget::row![
                    widget::button("Export").on_press(TriggerMessage::Export),
                    widget::button("Export stop").on_press(TriggerMessage::ExportStop)
                ],
                widget::text(&self.trigger_status),
                widget::text(&self.export_status),

                widget::column![
                    widget::rule::Rule::horizontal(10),
                    widget::text("Settings"),
                    self.trigger_form_buffer.view(None).map(TriggerMessage::SettingsMessage),
                ]
            ].width(250)),
        ].into();

        //let underlay:iced::Element<'a, TriggerMessage> = ;
        // let overlay =  if let Some(modal_data) = &self.trigger_interval_selector{
        //     Some(modal_data.overlay())
        // }
        // else{
        //     None
        // };
        // let res:iced::Element<'a, TriggerMessage> = modal(underlay, overlay)
        //     .backdrop(TriggerMessage::CancelChoseTrigger)
        //     .on_esc(TriggerMessage::CancelChoseTrigger)
        //     .align_y(alignment::Vertical::Center)
        //     .into();
        let res = if let Some(modal_data) = &self.trigger_interval_selector{
            modal_data.overlay()
        }
        else{
            underlay
        };


        res.map(PadamoAppMessage::TriggerMessage)
    }


    fn update(&mut self, msg: std::rc::Rc<PadamoAppMessage>, padamo:crate::application::PadamoStateRef){
        match msg.as_ref() {
            PadamoAppMessage::SetDetector(v) => {
                self.chart = Detector::from_cells(v.clone());
            }
            PadamoAppMessage::TriggerMessage(msg) => {
                match msg {
                    TriggerMessage::ChooseTrigger=>{
                        if let Some(s) = &self.signal{
                            if s.2.is_some(){ // Initiate trigger *only* if there is one.
                                self.trigger_interval_selector = Some(IntervalSelectionDialog::new(self.unmarked_intervals.clone()));
                            }
                            else{
                                padamo.show_error("No trigger is attached to signal");
                            }
                        }
                        else{
                            padamo.show_error("No signal");
                        }

                    }
                    TriggerMessage::CancelChoseTrigger=>{
                        self.trigger_interval_selector = None;
                    }
                    TriggerMessage::SelectionMessage(smsg)=>{
                        if let Some(sel) = &mut self.trigger_interval_selector{
                            sel.update(smsg.clone());
                        }
                    }
                    TriggerMessage::SelectPositive(i, _)=>{
                        self.selection = Some(*i);
                        self.selection_positive = true;
                        self.select_event();
                    }
                    TriggerMessage::SelectNegative(i, _)=>{
                        self.selection = Some(*i);
                        self.selection_positive = false;
                        self.select_event();
                    }
                    TriggerMessage::PlotZoomMessage(msg)=>{
                        self.view_transform.update(msg.to_owned());
                    }
                    TriggerMessage::ConfirmTrigger=>{
                        if let Some(v) = self.trigger_interval_selector.take(){
                            if let Some(signal) = &self.signal{
                                if let abi_stable::std_types::ROption::RSome(trigger) = &signal.2{
                                    let interval = v.get_interval();
                                    if !self.unmarked_intervals.is_available(interval){
                                        padamo.show_error(format!("Interval {} is not available", interval));
                                        return;
                                    }
                                    let trigger_source = (*trigger).clone();
                                    let settings = self.trigger_form_instance.clone();
                                    // let settings = trigger_form;
                                    self.stop_worker();
                                    println!("TRIGGER START {}", interval);

                                    let (tx,rx) = mpsc::channel::<bool>();

                                    let (tx_status,rx_status) = mpsc::channel::<TriggerProcessMessage>();

                                    let handle = thread::spawn(move || {
                                        let mut start = interval.start;
                                        let max_step = settings.chunksize;
                                        let length = interval.length();
                                        while start<interval.end{
                                            if let Ok(v) = rx.try_recv(){
                                                if v{
                                                    println!("Interrupt requested");
                                                    break;
                                                }
                                            }
                                            let step = max_step.min(interval.end-start);
                                            let data = trigger_source.request_range(start,start+step);
                                            println!("Triggering {}-{}",start,start+step);
                                            let mut res:Vec<bool> = Vec::with_capacity(data.shape[0]);
                                            res.resize(data.shape[0], false);
                                            for i in data.enumerate(){
                                                res[i[0]] |= data[&i];
                                            }
                                            let (positives, negatives) = split_intervals(&res);

                                            for pos in positives.iter(){
                                                tx_status.send(TriggerProcessMessage::MarkPositive(pos.offset(start))).unwrap();
                                            }


                                            for neg in negatives.iter(){
                                                tx_status.send(TriggerProcessMessage::MarkNegative(neg.offset(start))).unwrap();
                                            }

                                            start += step;
                                            tx_status.send(TriggerProcessMessage::Status(format!("{}/{}",start,length))).unwrap();
                                        }
                                        tx_status.send(TriggerProcessMessage::Status("DONE".into())).unwrap();
                                    });

                                    self.trigger_process = Some(Worker::new(handle, tx, rx_status));

                                    return;
                                }
                            }
                            println!("How did you managed to take interval without trigger?");
                        }
                    }
                    TriggerMessage::SettingsMessage(msg)=>{
                        // self.trigger_form_buffer.update(msg.clone(), &mut self.trigger_form);
                        match msg{
                            padamo_iced_forms::ActionOrUpdate::Action(_)=>(),
                            padamo_iced_forms::ActionOrUpdate::Update(u)=>{
                                self.trigger_form_buffer.update(u.to_owned());
                                match self.trigger_form_buffer.get(){
                                    Ok(v)=>self.trigger_form_instance = v,
                                    Err(e)=>eprintln!("Form get error: {}",e),
                                }
                            },
                        }
                    }
                    TriggerMessage::Stop=>{
                        if let Some(v) = &mut self.trigger_process{
                            v.request_stop();
                        }
                        //self.stop_animator();
                    }
                    TriggerMessage::ExamineEvent=>(),
                    TriggerMessage::Export=>{
                        if let Some(signal_ref) = &self.signal{
                            if let Some(path) = padamo.workspace.workspace("events_export").choose_dir_dialog(vec![]){
                                let (tx,rx) = mpsc::channel::<bool>();
                                let spatial:padamo_api::lazy_array_operations::LazyDetectorSignal = signal_ref.0.clone();
                                let temporal:padamo_api::lazy_array_operations::LazyTimeSignal = signal_ref.1.clone();
                                self.stop_export();

                                let (tx_status,rx_status) = mpsc::channel::<ExportProcessMessage>();
                                let intervals = self.positive_intervals.clone();

                                let handle = thread::spawn(move || {
                                    let total_len = intervals.container.len();
                                    for (i,interval) in intervals.container.iter().enumerate(){
                                        if let Ok(v) = rx.try_recv(){
                                            if v{
                                                println!("Interrupt requested");
                                                break;
                                            }
                                        }

                                        let frame = crate::compat::arraynd_to_ndarray(spatial.request_range(interval.start,interval.end));
                                        let tim:Vec<f64> = temporal.request_range(interval.start,interval.end).into();
                                        let time_of_event = tim[0];
                                        let time_of_event_secs:i64 = time_of_event as i64;
                                        let time_of_event_nsecs = ((time_of_event-time_of_event_secs as f64)*1e9) as u32;
                                        let tim1 = if let Some(v) = chrono::DateTime::from_timestamp(time_of_event_secs, time_of_event_nsecs){
                                            format!("T{:04}{:02}{:02}-{:02}{:02}{:02}.{:09}",v.year(),v.month(),v.day(),v.hour(), v.minute(), v.second(),v.nanosecond())
                                        }
                                        else{
                                            format!("I{}", interval.start)
                                        };

                                        let tgt_path = std::path::Path::new(&path);
                                        let file_path = tgt_path.join(format!("event_{}.h5",tim1));
                                        if let Ok(file) = hdf5::File::create(file_path){
                                            let space_ds = file.new_dataset::<f64>()
                                                .deflate(3)
                                                .shape(frame.shape());
                                            let space_ds = space_ds.create("pdm_2d_rot_global").unwrap();
                                            let time_ds = file.new_dataset::<f64>()
                                                .deflate(3)
                                                .shape(vec![tim.len()]);
                                            let time_ds = time_ds.create("unixtime_dbl_global").unwrap();

                                            space_ds.write(&frame).unwrap();
                                            time_ds.write(&tim).unwrap();
                                        }


                                        tx_status.send(ExportProcessMessage::Status(format!("{}/{}",i,total_len))).unwrap();
                                    }
                                });
                                self.export_process = Some(Worker::new(handle, tx, rx_status));
                            }
                        }
                    }
                    TriggerMessage::ExportStop=>{
                        if let Some(v) = &mut self.export_process{
                             v.request_stop();
                        }
                    }
                }
            }
            _ => (),
        }
    }

    fn late_update(&mut self, msg: std::rc::Rc<crate::messages::PadamoAppMessage>, padamo:crate::application::PadamoStateRef)->Option<crate::messages::PadamoAppMessage>{
        match msg.as_ref() {
            PadamoAppMessage::Run => {
                if let Some(padamo_api::prelude::Content::DetectorFullData(signal)) = padamo.compute_graph.environment.0.get(crate::builtin_nodes::viewer::VIEWER_SIGNAL_VAR){
                    //let signal_w = signal.clone();
                    let signal = signal.clone();
                    let length = signal.0.length();
                    self.signal = Some(signal);
                    self.reset_intervals(length);
                }
                None
            }
            PadamoAppMessage::Tick => {
                let mut will_stop = false;
                if let Some(anim) = self.trigger_process.take(){
                    if anim.is_finished(){
                        will_stop = true;
                    }
                    //let pip = &anim.feedback;
                    let mut recv_res = anim.feedback.try_recv();
                    let mut need_update = false;
                    while let Ok(v) = recv_res{
                        need_update = true;
                        match v{
                            TriggerProcessMessage::Status(status) => {self.trigger_status = status;},
                            TriggerProcessMessage::MarkPositive(pos) => {
                                self.mark_positive(pos);
                            },
                            TriggerProcessMessage::MarkNegative(neg) => {
                                self.mark_negative(neg);
                            },
                        }
                        recv_res = anim.feedback.try_recv();
                    }
                    if need_update{
                        self.update_interval_strings();
                    }
                    self.trigger_process = Some(anim);
                }

                if will_stop{
                    self.stop_worker();
                }

                will_stop = false;
                if let Some(exporter) = self.export_process.take(){
                    if exporter.is_finished(){
                        will_stop = true;
                    }
                    let mut recv_res = exporter.feedback.try_recv();
                    while let Ok(v) = recv_res{
                        match v{
                            ExportProcessMessage::Status(status)=>{self.export_status = status;}
                        }
                        recv_res = exporter.feedback.try_recv();
                    }

                    self.export_process = Some(exporter);
                }

                if will_stop{
                    self.stop_export();
                }


                if let Some(worker) = self.loader.take(){
                    if worker.is_finished(){
                        match worker.join() {
                            Ok(v)=>{
                                let maxes = get_maxes(&v.signal);
                                self.data = Some((v,maxes));
                            }
                            Err(_)=>{
                                padamo.show_error(format!("Unknown error ocurred while loading data"));
                            }
                        }
                    }
                    else{
                        self.loader = Some(worker);
                    }
                }
                None
            }
            PadamoAppMessage::TriggerMessage(msg)=>{
                if let TriggerMessage::ExamineEvent = msg{
                    if let Some(interval) = &self.selected_interval{
                        Some(PadamoAppMessage::ViewerMessage(super::viewer::ViewerMessage::FocusOn(interval.start, interval.end)))
                    }
                    else{
                        padamo.show_info("Select event to examine");
                        None
                    }
                }
                else{
                    None
                }
            }
            _ => {None},
        }
    }

    fn context_update(&mut self, msg: std::rc::Rc<crate::messages::PadamoAppMessage>, padamo:crate::application::PadamoStateRef){
        match msg.as_ref() {
            PadamoAppMessage::Save=>{
                if let Some(path) = padamo.workspace.workspace("marked_up_events_rs").save_dialog(vec![("Marked up tracks",vec!["json"])]){
                    //if let nfd::Response::Okay(path) = v{
                    if let Some(data) = &self.signal{
                        let positives = self.positive_intervals.to_unixtime_storage(&data.1);
                        let negatives = self.negative_intervals.to_unixtime_storage(&data.1);

                        let total = BinaryUnixIntervalStorage{positives,negatives};

                        match serde_json::to_string(&total) {
                            Ok(s) => {
                                match fs::write(path, s){
                                    Ok(_)=>(),
                                    Err(e)=>{
                                        padamo.show_error(format!("{}", e))
                                    }
                                }
                            }
                            Err(e) => padamo.show_error(format!("{}", e)),
                        }
                    }
                    //}
                }
            }
            PadamoAppMessage::Open=>{
                if let Some(signal) = self.signal.clone(){
                    if let Some(path) = padamo.workspace.workspace("marked_up_events_rs").open_dialog(vec![("Marked up tracks",vec!["json"])]){
                        match fs::read_to_string(path){
                            Ok(s)=>{
                                let deserialized: Result<BinaryUnixIntervalStorage,serde_json::Error> = serde_json::from_str(&s);
                                match deserialized {
                                    Ok(obj)=>{
                                        self.reset_intervals(signal.1.length());
                                        let positives = IntervalStorage::from_unixtime_storage(&obj.positives,&signal.1);
                                        let negatives = IntervalStorage::from_unixtime_storage(&obj.negatives,&signal.1);
                                        //println!("{:?}",positives);
                                        for i in positives.container.iter(){
                                            self.mark_positive(*i);
                                        }
                                        for i in negatives.container.iter(){
                                            self.mark_negative(*i);
                                        }
                                        self.update_interval_strings();
                                    }
                                    Err(e) => padamo.show_error(format!("{}", e))
                                }
                            }
                            Err(e) => padamo.show_error(format!("{}", e))
                        }
                    }
                }

            }
            _=>()
        }
    }
}
