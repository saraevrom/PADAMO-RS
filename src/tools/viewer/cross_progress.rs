use chrono::{DateTime, Utc};
use iced::widget::{row, column};
use padamo_api::lazy_array_operations::LazyTriSignal;

use crate::{application::PadamoState, messages::PadamoAppMessage};
use crate::custom_widgets::timeline::TimeLine;
use crate::detector_muxer::get_signal_var;

use iced::widget::text::Catalog;
use iced_font_awesome::{fa_icon_solid, FaIcon};

use super::PlayState;

#[derive(Clone, Copy, Debug)]
pub struct IntervalReport{
    pub start:usize,
    pub end:usize,
    pub pointer:usize
}

#[derive(Clone, Debug)]
pub struct CrossProgress{
    detector_id:usize,
    pointer:usize,
    pointer_str:String,
    datetime_entry:String,
    start:usize,
    end:usize,
    start_str:String,
    end_str:String,
    length:usize,
    playstate:PlayState,
}

#[derive(Clone, Debug)]
pub enum CrossProgressMessage{
    SetStart,
    SetEnd,
    Reset,
    JumpToStart,
    JumpToEnd,
    Stop,
    StepBack,
    StepFwd,
    Forward,
    Backward,
    SetViewPositionText(String),
    SetViewStartText(String),
    SetViewEndText(String),
    EditDatetime(String),
    SetViewPositionUnixTime(f64),
    SetViewPosition(usize),
    SubmitTimeline,
    SubmitDatetime,
    FocusOn(usize,usize),
}

pub fn get_icon<Theme:Catalog>(icon:&'static str)->FaIcon<'static,Theme>{
    fa_icon_solid(icon).size(20.0).color(iced::color![255,255,255])
}

pub fn make_player_pad<'a>()->iced::widget::Container<'a, CrossProgressMessage>{
    iced::widget::container(
        row![
            //iced::widget::button("<<").width(40),
            iced::widget::button(get_icon("backward-fast")).on_press(CrossProgressMessage::JumpToStart).width(40),
                            iced::widget::button(get_icon("backward")).on_press(CrossProgressMessage::Backward).width(40),
                            iced::widget::button(get_icon("backward-step")).on_press(CrossProgressMessage::StepBack).width(40),
                            iced::widget::button(get_icon("pause").size(20.0)).on_press(CrossProgressMessage::Stop).width(40),
                            iced::widget::button(get_icon("forward-step")).on_press(CrossProgressMessage::StepFwd).width(40),
                            iced::widget::button(get_icon("play")).on_press(CrossProgressMessage::Forward).width(40),

                            // iced::widget::button("Reset").on_press(ViewerMessage::Reset).width(225),
                            iced::widget::button(get_icon("forward-fast")).on_press(CrossProgressMessage::JumpToEnd).width(40),
                            //iced::widget::button(">>").width(40),
        ],


    ).center_x(iced::Length::Shrink).center_y(iced::Length::Shrink).width(iced::Length::Fill)
}

impl CrossProgress{
    pub fn new(detector_id: usize) -> Self {
        Self { detector_id,
            pointer:0,
            start:0, end:100,
            length:100,
            pointer_str:"0".into(),
            start_str:"0".into(),
            end_str:"100".into(),
            datetime_entry:"".into(),
            playstate: PlayState::Stop,
        }
    }

    pub fn view(&self, padamo:&PadamoState) -> iced::Element<'_,CrossProgressMessage>{
        let frame_num = &self.pointer_str;
        let start_frame = &self.start_str;
        let end_frame = &self.end_str;

        let res = row![
            iced::widget::Container::new(
                iced::Element::new(TimeLine::new(self.length,self.pointer, self.start, self.end,Some(CrossProgressMessage::SetViewPosition))),
            ).center_x(iced::Length::Fill).center_y(iced::Length::Shrink).width(iced::Length::Fill).height(iced::Length::Fill),

            iced::widget::container(column![
                row![
                    iced::widget::TextInput::new("",start_frame.as_str())
                    .on_input(CrossProgressMessage::SetViewStartText)
                    .on_submit(CrossProgressMessage::SubmitTimeline)
                    .width(100),

                    iced::widget::TextInput::new("",frame_num.as_str())
                    .on_input(CrossProgressMessage::SetViewPositionText)
                    .on_submit(CrossProgressMessage::SubmitTimeline)
                    .width(100),

                    iced::widget::TextInput::new("",end_frame.as_str())
                    .on_input(CrossProgressMessage::SetViewEndText)
                    .on_submit(CrossProgressMessage::SubmitTimeline)
                    .width(100),
                ],
                row![
                    //iced::widget::button("To start").on_press(ViewerMessage::JumpToStart).width(100),
                    iced::widget::button("Set start").on_press(CrossProgressMessage::SetStart).width(100),
                                    iced::widget::button("Reset").on_press(CrossProgressMessage::Reset).width(100),
                                    iced::widget::button("Set end").on_press(CrossProgressMessage::SetEnd).width(100),
                                    //iced::widget::button("To end").on_press(ViewerMessage::JumpToEnd).width(100),
                ],
                // row![
                iced::widget::TextInput::new("Enter datetime",self.datetime_entry.as_str())
                .on_input(CrossProgressMessage::EditDatetime)
                .on_submit(CrossProgressMessage::SubmitDatetime)
                .width(300),
                                    make_player_pad().center_x(iced::Length::Shrink)
                                    // ],
            ]).style(iced::widget::container::bordered_box),

        ].height(iced::Length::Shrink);
        res.into()
    }

    fn clamp(&mut self){
        if self.pointer>self.length{
            self.pointer = self.length;
        }
        if self.end>self.length{
            self.end = self.length;
        }
        if self.start>self.end {
            self.start = self.end;
        }

        if self.start>self.pointer{
            self.start = self.pointer;
        }
        if self.end<self.pointer{
            self.end = self.pointer;
        }
    }

    pub fn fill_strings(&mut self){
        self.pointer_str = self.pointer.to_string();
        self.start_str = self.start.to_string();
        self.end_str = self.end.to_string();
    }

    pub fn try_get_signal<'a>(&'a self, padamo: &'a PadamoState)->Option<&'a LazyTriSignal>{
        if let Some(padamo_api::prelude::Content::DetectorFullData(signal)) = padamo.compute_graph.environment.0.get(get_signal_var(self.detector_id).as_str()){
            Some(signal)
        }
        else{
            None
        }
    }

    fn get_frame_any(&self, padamo:&PadamoState, target_detector_id:usize, source_frame:usize)->Option<usize>{
        if let Some(my_signal) = self.try_get_signal(padamo){
            if target_detector_id==self.detector_id{
                Some(source_frame)
            }
            else{
                let target_ut = my_signal.1.request_range(source_frame, source_frame+1)[0];
                if let Some(padamo_api::prelude::Content::DetectorFullData(signal)) = padamo.compute_graph.environment.0.get(get_signal_var(target_detector_id).as_str()){
                    Some(signal.1.find_unixtime(target_ut))
                }
                else{
                    None
                }
            }
        }
        else{
            None
        }

    }

    pub fn get_frame(&self, padamo:&PadamoState, target_detector_id:usize)->Option<usize>{
        self.get_frame_any(padamo, target_detector_id, self.pointer)

    }

    pub fn update_signal_info(&mut self, padamo: &mut PadamoState)->Option<IntervalReport>{
        if let Some(padamo_api::prelude::Content::DetectorFullData(signal)) = padamo.compute_graph.environment.0.get(crate::builtin_nodes::viewer::VIEWER_SIGNAL_VAR){
            //let signal_w = signal.clone();
            // self.signal = Some(signal.clone());
            //self.signal = Some((signal_w.0,signal_w.1,signal_w.2.into()));
            self.length = signal.0.length()-1;
            // if self.file_changed{
            //     self.end = self.length;
            //     self.file_changed = false;
            // }
            self.clamp();
            // self.update_buffer(Some(padamo));
            // self.fill_strings();

            // return Some(PadamoAppMessage::PlotterMessage(super::plotter::messages::PlotterMessage::SyncData {
            //     start: self.start,
            //     end: self.end+1,
            //     pointer: self.pointer,
            //     force_clear:true,
            // }));
            if self.detector_id==0{
                Some(IntervalReport { start: self.start, end: self.end, pointer: self.pointer })
            }
            else{
                None
            }
        }
        else {
            None
        }
    }

    pub fn update(&mut self, message:CrossProgressMessage, padamo:&mut PadamoState){
        let mut request_buffer_fill = true;
        match message {
            CrossProgressMessage::SetStart => {
                self.start= self.pointer;
            },
            CrossProgressMessage::SetEnd => {
                self.end = self.pointer;
            },
            CrossProgressMessage::Reset => {
                self.start = 0;
                self.end = self.length;
            },
            CrossProgressMessage::JumpToStart => {
                self.pointer = self.start;
            },
            CrossProgressMessage::JumpToEnd => {
                self.pointer = self.end;
            },
            CrossProgressMessage::Stop => {
                self.playstate = PlayState::Stop;
            },
            CrossProgressMessage::StepBack => {
                self.playstate = PlayState::Stop;
                if self.pointer>0{
                    self.pointer-=1;
                }
            },
            CrossProgressMessage::StepFwd => {
                self.playstate = PlayState::Stop;
                if self.pointer<self.length{
                    self.pointer+= 1;
                }
            },
            CrossProgressMessage::Forward => {
                self.playstate = PlayState::Forward;
            },
            CrossProgressMessage::Backward => {
                self.playstate = PlayState::Backward;
            },
            CrossProgressMessage::SetViewPositionText(txt) => {
                self.pointer_str = txt;
                // self.fill_strings();
                request_buffer_fill = false;
            },
            CrossProgressMessage::SetViewStartText(txt) => {
                self.start_str = txt;
                // self.fill_strings();
                request_buffer_fill = false;
            },
            CrossProgressMessage::SetViewEndText(txt) => {
                self.end_str = txt;
                request_buffer_fill = false;
                // self.fill_strings();
            },
            CrossProgressMessage::EditDatetime(txt) => {
                self.datetime_entry = txt;
            },
            CrossProgressMessage::SubmitTimeline => {
                if let Ok(newpos) = self.pointer_str.parse(){
                    self.pointer = newpos;
                }
                if let Ok(newstart) = self.start_str.parse(){
                    self.start = newstart;
                }
                if let Ok(newend) = self.end_str.parse(){
                    self.end = newend;
                }
                self.clamp();
            },
            CrossProgressMessage::SubmitDatetime => {
                println!("Submitted DT {}",self.datetime_entry);



                if let Some(signal) = &self.try_get_signal(&padamo){

                    let ut:f64 = signal.1.request_range(self.pointer,self.pointer+1)[0];
                    let ut_s = ut as i64;
                    //let ut_ns = ((ut-ut_s as f64)*1e9) as u32;
                    let ut_ns = ((ut-(ut_s as f64))*1e9) as u32;


                    //println!("Datetime from {}=>{}; {}",ut,ut_s,ut_ns);
                    // let naive = NaiveDateTime::from_timestamp_opt(ut_s,ut_ns).unwrap();
                    //
                    // // Create a normal DateTime from the NaiveDateTime
                    // let start_dt:DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
                    let start_dt:DateTime<Utc> = DateTime::from_timestamp(ut_s, ut_ns).unwrap();

                    if let Some(end_dt) = datetime_parser::parse_datetimes(&self.datetime_entry, start_dt){
                        let index = crate::time_search::find_time(&signal.1, end_dt);
                        self.pointer = index;
                        self.clamp();
                    }
                }

                self.datetime_entry.clear();
            },
            CrossProgressMessage::SetViewPositionUnixTime(f)=>{
                if let Some(signal) = self.try_get_signal(&padamo){
                    let pos = crate::time_search::find_unixtime(&signal.1, f);

                    self.pointer = pos;
                    if self.pointer<self.start{
                        self.start = self.pointer;
                    }
                    if self.pointer>self.end{
                        self.end = self.pointer;
                    }
                }
            },
            CrossProgressMessage::SetViewPosition(pos)=>{
                self.pointer = pos;
                if self.pointer<self.start{
                    self.start = self.pointer;
                }
                if self.pointer>self.end{
                    self.end = self.pointer;
                }
            }
            CrossProgressMessage::FocusOn(start, end)=>{
                if self.detector_id==0{
                    self.pointer = start;
                    self.start = start;
                    padamo.current_page = 0;
                    self.end = end;
                }

            }
        }
        if request_buffer_fill{
            self.fill_strings();
        }
    }

    pub fn get_interval(&self, padamo:&PadamoState, target_detector_id:usize)->Option<(usize,usize)>{
        let start = self.get_frame_any(padamo, target_detector_id, self.start);
        let end = self.get_frame_any(padamo, target_detector_id, self.end);
        if let (Some(a), Some(b)) = (start,end){
            Some((a,b))
        }
        else{
            None
        }
    }

    pub fn run_tick(&mut self)->bool{
        match self.playstate {
            PlayState::Stop=>false,
            PlayState::Forward=>{
                if self.pointer<self.end{
                    self.pointer += 1;
                    // self.update_buffer(Some(padamo));
                    // self.fill_strings();
                    true
                }
                else{
                    self.playstate = PlayState::Stop;
                    false
                }
            }
            PlayState::Backward=>{
                if self.pointer>self.start{
                    self.pointer-=1;
                    // self.update_buffer(Some(padamo));
                    // self.fill_strings();
                    true
                }
                else{
                    self.playstate = PlayState::Stop;
                    false
                }
            }
        }
    }

    pub fn get_sync_message(&self, check_playstate:bool)->Option<PadamoAppMessage>{
        if self.detector_id !=0{
            None
        }
        else if check_playstate{
            if let PlayState::Stop = self.playstate{
                None
            }
            else{
                self.form_sync_message()
                //Some(PadamoAppMessage::PlotterMessage(super::plotter::messages::PlotterMessage::SyncPointer(self.pointer)))
            }
        }
        else {
            self.form_sync_message()
        }
    }

    fn form_sync_message(&self)->Option<PadamoAppMessage>{
        Some(PadamoAppMessage::PlotterMessage(crate::tools::plotter::messages::PlotterMessage::SyncData { start: self.start, end: self.end+1, pointer: self.pointer, force_clear:false }))
    }
}
