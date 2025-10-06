use self::{data_state::DataState, messages::PlotterMessage};
use std::{cell::RefCell, thread};

use super::PadamoTool;
use form::PlotterActions;
//use iced::advanced::Widget;
//use padamo_api::lazy_array_operations::ndim_array::ArrayND;
use iced::{widget::{self}, Length};
use padamo_detectors::{DetectorAndMask, DetectorPlotter};
use plotters::coord::{cartesian::Cartesian2d, types::RangedCoordf64};
use plotters_iced::Chart;
use crate::{application::PadamoState, messages::PadamoAppMessage};
//use super::viewer::ViewerMessage;
pub mod messages;
pub mod diagram;
//pub mod selection_diagram;
use padamo_api::lazy_array_operations::ndim_array::ArrayND;
use iced::widget::canvas::Cache;
use super::viewer::ViewerMessage;
// use super::viewer::make_player_pad;
use super::viewer::cross_progress::{CrossProgressMessage, make_player_pad};
// use padamo_workspace::PadamoWorkspace;
use padamo_api::lazy_array_operations::{LazyDetectorSignal,LazyTimeSignal};

use padamo_iced_forms::{ActionOrUpdate, IcedFormBuffer};

//static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);
pub mod data_state;
pub mod form;

pub use form::{TimeAxisFormat,LCMode};




#[allow(dead_code)]
#[derive(Clone,Debug,Copy,Eq,PartialEq)]
pub enum TimeAxisRangeFormat{
    Seconds,
    GTU
}

impl TimeAxisFormat{
    pub fn format(&self,x:f64, x0:f64)->String{
        match self {
            Self::AsIs=> format!("{:.4}", x),
            Self::GTU=> format!("{}", x as usize),
            Self::Offset=>{
                let off = x-x0;
                format!("{:.4}", off)
            },
            Self::Time=>{
                use chrono::DateTime;
                let secs = x.floor();
                let nsecs = (x - x.floor())*1e9;
                if let Some(dt) = DateTime::from_timestamp(secs as i64, nsecs as u32){
                    let fmt = dt.format("%F %T%.3f");
                    format!("{}",fmt)
                }
                else{
                    "<!!!>".into()
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn range_format(&self)->TimeAxisRangeFormat{
        match self {
            Self::AsIs | Self::Offset | Self::Time => TimeAxisRangeFormat::Seconds,
            Self::GTU => TimeAxisRangeFormat::GTU
        }
    }

}

//type DataCache = (Vec<f64>,ArrayND<f64>, Vec<f64>, usize);



pub struct Plotter{
    detector_plotter:padamo_detectors::DetectorPlotter<PlotterMessage>,

    plot_spec:RefCell<Option<Cartesian2d<RangedCoordf64, RangedCoordf64>>>,

    data: data_state::DataState,
    pixels:Vec<Vec<usize>>,
    pixels_show:Vec<bool>,


    //safeguard_str:String,

    //last_indices:Option<(usize,usize)>,
    last_y_limits:(f64,f64),
    last_x_limits:(f64,f64),
    last_y_limits_live:(f64,f64),
    last_x_limits_live:(f64,f64),
    //lazy_data_load:Option<(usize,usize)>,
    //select_threshold_string:String,

    cache:Cache,
    view_index:usize,
    view_pivot:usize,


    min_x_string:String,
    max_x_string:String,
    min_y_string:String,
    max_y_string:String,
    //out_shape_str:(String,String),
    detector_pixels:usize,

    is_selecting_pixels:bool,

    form_buffer:form::PlotterFormBuffer,
    form_instance:form::PlotterForm,
    //loader:Option<thread::JoinHandle<DataCache>>
}


pub fn spawn_loader(lazy_spatial:LazyDetectorSignal,lazy_temporal:LazyTimeSignal,start:usize,end:usize)->thread::JoinHandle<data_state::DataCache>{
    thread::spawn( move || {
        let spatial = lazy_spatial.request_range(start,end);
        let temporal:Vec<f64> = lazy_temporal.request_range(start,end).into();

        let spatial_out = spatial;
        let minv = spatial_out.flat_data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let maxv = spatial_out.flat_data.iter().fold(-f64::INFINITY, |a, &b| a.max(b));
        let pixel_count_u64 = spatial_out.flat_data.len()/temporal.len();
        let mut lc:Vec<f64> = Vec::with_capacity(temporal.len());
        lc.resize(temporal.len(), 0.0);

        for index in spatial_out.enumerate(){
            lc[index[0]] += spatial_out[&index];
        }

        //self.data = Some((temporal,spatial_out,lc, pixel_count_u64));
        let time_probe_size = lazy_temporal.length().min(1000);
        let time_probe:Vec<f64> = lazy_temporal.request_range(0,time_probe_size).into();

        let mut t_last = time_probe[0];
        let mut time_step = f64::MAX;
        for rt in time_probe.iter(){
            let t = *rt;
            let dt = t-t_last;
            if dt > 0.0{
                if dt<time_step{
                    time_step = dt;
                }
            }
            t_last = t;
        }

        data_state::DataCache {
            primary:(spatial_out, temporal),
            // time:temporal,
            time_step,
            // signal:spatial_out,
            lc,
            pixel_count:pixel_count_u64,
            start,
            end,
            minv,
            maxv,
            last_indices:(start,end)
        }
    })
}

pub fn get_maxes(pix_data:&ArrayND<f64>)->ArrayND<f64>{
    let res_shape:Vec<usize> = pix_data.shape.iter().skip(1).cloned().collect();
    let mut maxes:ArrayND<f64> = ArrayND::<f64>::defaults(res_shape);
    for (i,v) in maxes.flat_data.iter_mut().enumerate(){
        *v = pix_data.flat_data[i];
    }
    for i in pix_data.enumerate(){
        let tgt_index:Vec<usize> = i.iter().skip(1).cloned().collect();
        if pix_data[&i]>maxes[&tgt_index]{
            maxes.set(&tgt_index, pix_data[&i]);
        }
    }
    maxes
}

impl Plotter{
    pub fn new()->Self{
        let mut res = Self {  data:DataState::NoData,
                plot_spec: RefCell::new(None),
                detector_plotter:DetectorPlotter::new(),
                pixels:Vec::new(),
                //safeguard_str:"30000".into(),
                //last_indices:None,
                pixels_show: Vec::new(),
                last_y_limits:(0.0,0.0),
                last_x_limits:(0.0,0.0),
                last_y_limits_live:(0.0,0.0),
                last_x_limits_live:(0.0,0.0),
                //select_threshold_string:"".into(),
                cache:Cache::new(),
                view_index:0,
                view_pivot:0,
                min_x_string:"".into(),
                max_x_string:"".into(),
                min_y_string:"".into(),
                max_y_string:"".into(),
                //out_shape_str:("".into(),"".into()),
                detector_pixels:1,
                is_selecting_pixels:false,
                form_buffer:Default::default(),
                form_instance:Default::default(),
                //loader:None,
                //lazy_data_load:None,
        };
        res.sync_entries();
        res
    }

    pub fn clear(&mut self){
        self.data = DataState::NoData;
        //self.last_indices = None;
        self.pixels.clear();
        self.pixels_show.clear();
        self.cache.clear();
    }

    pub fn add_pixel_with_visibility(&mut self, pix:&Vec<usize>, visible:bool){
        if let Some(i) = self.pixels.iter().position(|r| r==pix){
            self.pixels_show[i] = visible;
        }
        else{
            self.pixels.push(pix.clone());
            self.pixels_show.push(visible);
            println!("Selected pixel {:?}", pix);
            self.place_last_pixel();
        }
    }

    pub fn toggle_pixel(&mut self, pix:&Vec<usize>){
        if let Some(i) = self.pixels.iter().position(|r| r==pix){
            self.pixels_show[i] = !self.pixels_show[i];
        }
        else{
            self.pixels.push(pix.clone());
            self.pixels_show.push(true);
            println!("Selected pixel {:?}", pix);
            self.place_last_pixel();
        }
    }


    pub fn select_pixel(&mut self, pix:&Vec<usize>){
        // if !self.pixels.contains(pix){
        //     self.pixels.push(pix.clone());
        //     self.pixels_show.push(true);
        //     println!("Selected pixel {:?}", pix);
        //     self.place_last_pixel();
        // }
        self.add_pixel_with_visibility(pix, true);
    }


    fn place_last_pixel(&mut self){
        let mut i = self.pixels.len()-1;
        while i>0{
            if self.pixels[i]<self.pixels[i-1]{
                self.swap_pixels(i, i-1);
                i-=1;
            }
            else{
                return;
            }
        }
    }

    pub fn clear_pixels(&mut self){
        self.pixels.clear();
        self.pixels_show.clear();
    }


    fn swap_pixels(&mut self, i:usize, j:usize){
        self.pixels.swap(i, j);
        self.pixels_show.swap(i,j);
    }

    fn sync_entries(&mut self){
        let (xmin,xmax) = self.last_x_limits_live;
        let (ymin,ymax) = self.last_y_limits_live;
        let x_pivot = self.last_x_limits.0;

        self.min_x_string = (xmin-x_pivot).to_string();
        self.max_x_string = (xmax-x_pivot).to_string();

        self.min_y_string = ymin.to_string();
        self.max_y_string = ymax.to_string();

        //self.out_shape_str = (self.out_shape.0.to_string(), self.out_shape.1.to_string());
        //self.select_threshold_string = self.select_threshold.to_string();
        // self.safeguard_str = self.safeguard.to_string();
    }

    fn ensure_data_async(&mut self, padamo:&mut PadamoState){
        // let indices = (start,end);
        // if let Some(ind) = self.last_indices{
        //     if indices != ind{
        //         self.clear();
        //     }
        // }
        if let DataState::PendingLoad(start, end) = self.data{
            let safeguard = self.form_instance.display_settings.safeguard;
            if end-start>safeguard{
                padamo.show_warning(format!("Cannot show signal of length {} (Safeguard is {})",end-start,safeguard));
                self.data = DataState::NoData;
                return;
            }

            if let Some(padamo_api::prelude::Content::DetectorFullData(signal_in)) = padamo.compute_graph.environment.0.get(crate::builtin_nodes::viewer::VIEWER_SIGNAL_VAR){
                //let signal = (*signal_in).clone();
                let lazy_spatial = signal_in.0.clone();
                let lazy_temporal = signal_in.1.clone();

                println!("Getting data");
                let length = lazy_spatial.length();
                // No need to compare with zero.
                if !(start<end && end<=length){
                    return;
                }

                //let start = start;
                //let end = end;

                let loader = spawn_loader(lazy_spatial, lazy_temporal, start, end);
                // self.clear_pixels();

                //self.loader = Some(loader);
                //self.last_indices = Some(indices);
                self.data = DataState::Loading(loader);
            }
        }

    }

    fn update_data_state(&mut self){
        if let DataState::Loaded(v) = &self.data{
            self.detector_pixels = v.pixel_count;
            self.view_pivot = v.start;
            let axis_formatter = self.form_instance.display_settings.time_format;
            self.last_x_limits = if let TimeAxisFormat::GTU=axis_formatter{
                (v.start as f64, v.end as f64)
            }
            else{
                let time = &v.primary.1;
                (time[0],time[time.len()-1])
            };
            self.last_x_limits_live = self.last_x_limits;

            self.last_y_limits = (v.minv,v.maxv);
            self.last_y_limits_live = self.last_y_limits;
            self.sync_entries();
            self.cache.clear();
        }
    }

    fn save_chart<T:plotters::backend::DrawingBackend>(&self, plotter:T, padamo:&PadamoState){
        use plotters::prelude::*;
        let root_area = plotter.into_drawing_area();
        root_area.fill(&WHITE).unwrap();
        //let cc = ChartBuilder::on(&root_area);
        if let Some(detector) = self.get_detector(padamo){
            let charter = diagram::PlotterChart::new(&self, detector);
            charter.draw_chart(&(), root_area);
        }

    }

    fn get_detector<'a>(&'a self, padamo:&'a PadamoState)->Option<&'a DetectorAndMask>{
        padamo.detectors.get_primary()
    }

}

impl PadamoTool for Plotter{
    fn tab_name(&self)->String {
        "Signal plotter".into()
    }

    fn view<'a>(&'a self, padamo: &'a PadamoState)->iced::Element<'a, PadamoAppMessage> {
        if let Some(detector) = self.get_detector(padamo){
            let mut pixlist = widget::Column::new();
            for i in 0..self.pixels.len(){
                let check = widget::checkbox(format!("{:?}",self.pixels[i]),self.pixels_show[i]).on_toggle(PlotterMessage::toggle_pixel(i));
                pixlist = pixlist.push(check);
            }
            let pixlist_element:iced::Element<'a, PlotterMessage> = pixlist.into();

            //let diag = chart.view();


            let chart_view:iced::Element<'a, PlotterMessage> = if self.is_selecting_pixels{
                let action:Option<fn(Vec<usize>)->PlotterMessage> = None;
                let body = self.detector_plotter.view_map(Some(detector),&self.pixels, &self.pixels_show, Some(PlotterMessage::TogglePixelByName), action);
                let body:iced::Element<'_,PlotterMessage> = iced::widget::container(body).width(Length::Fill).height(Length::Fill).into();
                iced::widget::column![
                    widget::container(
                        body,
                    ).width(iced::Length::Fill).align_x(iced::alignment::Horizontal::Center),
                    widget::container(
                        widget::button("Done").on_press(PlotterMessage::HidePixelSelector).width(100)
                    ).width(iced::Length::Fill).align_x(iced::alignment::Horizontal::Center)

                ].width(iced::Length::Fill).height(iced::Length::Fill).into()
            }
            else{
                match self.data{
                    DataState::NoData => widget::text("No data").into(),
                    DataState::PendingLoad(_, _) => widget::text("Pending load").into(),
                    DataState::Loading(_) => widget::text("Loading...").into(),
                    DataState::Loaded(_) => diagram::PlotterChart::new(&self,detector).view().into(),
                }
            };

            let main_row:iced::Element<'a, PlotterMessage> = iced::widget::container(widget::row![
                widget::column![
                    widget::container(chart_view).width(iced::Length::Fill).height(iced::Length::Fill),
                                                                                     widget::row![
                                                                                         widget::text("X"),
                                                                                     iced::widget::TextInput::new("x min",&self.min_x_string)
                                                                                     .on_input(PlotterMessage::SetXMin)
                                                                                     .on_submit(PlotterMessage::SubmitLimits)
                                                                                     .width(100),
                                                                                     widget::text("-"),
                                                                                     iced::widget::TextInput::new("x max",&self.max_x_string)
                                                                                     .on_input(PlotterMessage::SetXMax)
                                                                                     .on_submit(PlotterMessage::SubmitLimits)
                                                                                     .width(100),
                                                                                     //widget::rule::Rule::vertical(10),
                                                                                     widget::text(";     Y"),
                                                                                     iced::widget::TextInput::new("y min",&self.min_y_string)
                                                                                     .on_input(PlotterMessage::SetYMin)
                                                                                     .on_submit(PlotterMessage::SubmitLimits)
                                                                                     .width(100),
                                                                                     widget::text("-"),
                                                                                     iced::widget::TextInput::new("y max",&self.max_y_string)
                                                                                     .on_input(PlotterMessage::SetYMax)
                                                                                     .on_submit(PlotterMessage::SubmitLimits)
                                                                                     .width(100),
                                                                                     ].width(iced::Length::Fill).height(iced::Length::Shrink),
                ].width(iced::Length::Fill).height(iced::Length::Fill),

                                                                                     widget::rule::Rule::vertical(10),
                                                                                     widget::scrollable(widget::column![
                                                                                         self.form_buffer.view(None).map(PlotterMessage::FormMessage),
                                                                                                        pixlist_element
                                                                                     ]).width(300).height(iced::Length::Fill)
            ]).width(iced::Length::Shrink).height(iced::Length::Fill)
            .into();


            let main_row = main_row.map(PadamoAppMessage::PlotterMessage);
            let pad:iced::Element<'a, CrossProgressMessage> = make_player_pad().height(40).into();
            let pad:iced::Element<'a, ViewerMessage> = pad.map(ViewerMessage::TimeLine);

            let underlay:iced::Element<'a, PadamoAppMessage> = widget::column![
                main_row,
                pad.map(PadamoAppMessage::ViewerMessage)
            ].into();


            underlay
        }
        else{
            iced::widget::text("No detector").into()
        }
    }

    fn update(&mut self, msg: std::rc::Rc<PadamoAppMessage>, padamo:crate::application::PadamoStateRef) {
        match msg.as_ref() {
            PadamoAppMessage::Tick => {
                if let Some(worker) = self.data.take_worker(){
                    if worker.is_finished(){
                        if let Ok(v) = worker.join(){
                            // self.detector_pixels = v.pixel_count;
                            // self.view_pivot = v.start;
                            // self.last_x_limits = (v.time[0],v.time[v.time.len()-1]);
                            // self.last_x_limits_live = self.last_x_limits;
                            //
                            // self.last_y_limits = (v.minv,v.maxv);
                            // self.last_y_limits_live = self.last_y_limits;
                            self.data = DataState::Loaded(v);
                            self.update_data_state();
                            // self.sync_entries();
                            println!("Loaded interval");
                            // self.cache.clear();
                        }
                    }
                    else{
                        self.data = DataState::Loading(worker);
                    }
                }
            }
            PadamoAppMessage::SetDetector(v)=>{
                //self.detector = DetectorAndMask::from_cells(v.clone())
            }
            PadamoAppMessage::PlotterMessage(plot_msg) => {
                let mut will_replot = true;
                match plot_msg {
                    PlotterMessage::FormMessage(form_msg)=>{
                        match form_msg{
                            ActionOrUpdate::Update(update_msg)=>{
                                self.form_buffer.update(update_msg.clone());
                                // println!("Form update");
                                match self.form_buffer.get(){
                                    Ok(new_v)=> {
                                        // println!("{:?}",new_v);
                                        self.form_instance = new_v;
                                        self.update_data_state();
                                    },
                                    Err(e)=>eprintln!("Form get error: {}",e),
                                    // println!("Get OK");
                                }
                            },
                            ActionOrUpdate::Action(action)=>{
                                if let Some(plotter_action) = action.downcast_ref::<PlotterActions>(){
                                    match plotter_action {
                                        PlotterActions::Noop=>(),
                                        PlotterActions::Save=>{
                                            //diagram::PlotterChart::new(&self).
                                            let result = padamo.workspace.workspace("plots").save_dialog(vec![
                                                ("Portable net graphics", vec!["png"]),
                                                ("Lossy compressed JPEG", vec!["jpg"]),
                                                ("Scalar vector graphics", vec!["svg"])
                                            ]);
                                            if let Some(v) = result{
                                                use plotters::prelude::*;
                                                let out_shape = (self.form_instance.output_shape.width, self.form_instance.output_shape.height);
                                                let filename = std::path::Path::new(&v);
                                                let ext:Option<&str> = if let Some(e) = filename.extension(){
                                                    e.to_str()
                                                }
                                                else{
                                                    None
                                                };

                                                match ext{
                                                    Some("svg")=>{self.save_chart(SVGBackend::new(&v, out_shape),padamo)},
                                                    Some("png")=>{self.save_chart(BitMapBackend::new(&v, out_shape),padamo)},
                                                    Some("jpg")=>{self.save_chart(BitMapBackend::new(&v, out_shape),padamo)},
                                                    _=>{
                                                        println!("Cannot determine backend for {}", v);
                                                    }
                                                }
                                            }
                                            will_replot=false;
                                        },
                                        PlotterActions::ClearSelection=>{
                                            self.clear_pixels();
                                        },
                                        PlotterActions::ThresholdSelect=>{
                                            if let DataState::Loaded(data) = &self.data{
                                                let pix_data = data.primary.0.clone();
                                                let maxes = get_maxes(&pix_data);
                                                for i in maxes.enumerate(){
                                                    if maxes[&i]>self.form_instance.selector.threshold{
                                                        self.select_pixel(&i);
                                                    }
                                                }
                                            }
                                        },
                                        PlotterActions::ManualSelect=>{
                                            self.is_selecting_pixels = !self.is_selecting_pixels;
                                        }
                                    }
                                }
                            },
                        }
                    }



                    PlotterMessage::PlotPixel(start, end, index) => {
                        println!("Engaged plot {}..{} ({}) for {:?}", start,end, end-start, index);

                        if let DataState::NoData = self.data{
                            self.data.apply_start_end(*start, *end);
                        }

                        self.ensure_data_async(padamo);

                        self.select_pixel(index);
                    },
                    PlotterMessage::TogglePixel(index, value)=>{
                        self.pixels_show[*index] = *value;
                    }

                    PlotterMessage::SyncData { start, end, pointer, force_clear }=>{
                        self.view_index = *pointer;
                        if *force_clear{
                            self.clear();
                        }
                        self.data.apply_start_end(*start, *end);
                        //will_replot = self.display_pointer;
                        //self.lazy_data_load = Some((*start,*end));
                    }
                    // PlotterMessage::SyncPointer(ptr)=>{
                    //     self.view_index = *ptr;
                    //     will_replot = self.display_pointer;
                    // }


                    PlotterMessage::SetXMin(v)=>{
                        self.min_x_string = v.clone();
                    }
                    PlotterMessage::SetXMax(v)=>{
                        self.max_x_string = v.clone();
                    }

                    PlotterMessage::SetYMin(v)=>{
                        self.min_y_string = v.clone();
                    }
                    PlotterMessage::SetYMax(v)=>{
                        self.max_y_string = v.clone();
                    }

                    PlotterMessage::SubmitLimits=>{
                        if let Ok(min_x_unshifted) = self.min_x_string.parse::<f64>(){
                            if let Ok(max_x_unshifted) = self.max_x_string.parse::<f64>(){
                                let min_x = min_x_unshifted + self.last_x_limits.0;
                                let max_x = max_x_unshifted + self.last_x_limits.0;
                                let (bottom_x, top_x) = self.last_x_limits;
                                if bottom_x<=min_x && min_x<=max_x && max_x<=top_x{
                                    self.last_x_limits_live = (min_x,max_x);
                                }
                            }
                        }
                        if let Ok(min_y) = self.min_y_string.parse(){
                            if let Ok(max_y) = self.max_y_string.parse(){
                                // let (bottom_y, top_y) = self.last_y_limits;
                                // let top_y_scaled = if self.form_instance.display_settings.lc_mean {top_y} else {top_y*(self.detector_pixels as f64)};
                                // if bottom_y<=min_y && min_y<=max_y && max_y<=top_y_scaled{
                                if min_y<=max_y{
                                    self.last_y_limits_live = (min_y,max_y);
                                }
                            }
                        }
                        self.sync_entries();
                    }


                    // PlotterMessage::LazySelectData(start, end)=>{
                    //     self.lazy_data_load = Some((*start,*end));
                    // }
                    PlotterMessage::PlotXClicked(_)=>(),

                    PlotterMessage::HidePixelSelector=>{
                        self.is_selecting_pixels = false;
                    }

                    PlotterMessage::TogglePixelByName(v)=>{
                        println!("Toggled pixel {:?}",v);
                        self.toggle_pixel(v);
                    }
                }
                if will_replot{
                    self.cache.clear();
                }
            }
            _ => (),
        }

    }


    fn late_update(&mut self, msg: std::rc::Rc<crate::messages::PadamoAppMessage>, _padamo:crate::application::PadamoStateRef)->Option<crate::messages::PadamoAppMessage> {
        if let PadamoAppMessage::PlotterMessage(PlotterMessage::PlotXClicked(f)) = msg.as_ref(){
            if let TimeAxisFormat::GTU = self.form_instance.display_settings.time_format{
                let i = *f as usize;
                return Some(PadamoAppMessage::ViewerMessage(ViewerMessage::TimeLine(CrossProgressMessage::SetViewPosition(i))));
            }
            else{
                return Some(PadamoAppMessage::ViewerMessage(ViewerMessage::TimeLine(CrossProgressMessage::SetViewPositionUnixTime(*f))));
            }
        }
        None
    }

    fn context_update(&mut self, _msg: std::rc::Rc<crate::messages::PadamoAppMessage>, padamo:crate::application::PadamoStateRef) {
        // if let Some((start,end)) = self.lazy_data_load.take(){
        //     println!("Eager load {},{}",start,end);
        //     self.data.apply_start_end(start, end);
        //     self.ensure_data_async(padamo);
        // }
        if let DataState::PendingLoad(_, _) = self.data{
            self.ensure_data_async(padamo);
        }
    }
}
