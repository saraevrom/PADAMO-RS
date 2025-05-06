mod messages;
mod animator;
mod form;

use super::PadamoTool;
use abi_stable::std_types::ROption;
use iced::widget::text::Catalog;
use iced_font_awesome::FaIcon;
use padamo_api::calculation_nodes::content::Content;
use padamo_api::lazy_array_operations::make_lao_box;
use padamo_detectors::Detector;
use plotters_video::VideoBackend;
use crate::application::PadamoState;
use crate::custom_widgets::timeline::TimeLine;
use chrono::{DateTime, Utc};
use iced::widget::{column,row};
use crate::messages::PadamoAppMessage;
use std::time::Instant;
// use padamo_iced_forms_derive::IcedForm;
// use padamo_iced_forms::IcedFormInterface;
use padamo_iced_forms::{ActionOrUpdate, IcedFormBuffer};

use std::thread;
use std::sync::mpsc;
use sysinfo::{System,RefreshKind,MemoryRefreshKind};

pub use messages::ViewerMessage;
use form::AnimationParameters;

use form::ViewerForm;

use iced_font_awesome::fa_icon_solid;

fn get_icon<Theme:Catalog>(icon:&'static str)->FaIcon<'static,Theme>{
    fa_icon_solid(icon).size(20.0).color(iced::color![255,255,255])
}

pub fn make_player_pad<'a>()->iced::widget::Container<'a, ViewerMessage>{
    iced::widget::container(
        row![
            //iced::widget::button("<<").width(40),
            iced::widget::button(get_icon("backward-fast")).on_press(ViewerMessage::JumpToStart).width(40),
            iced::widget::button(get_icon("backward")).on_press(ViewerMessage::Backward).width(40),
            iced::widget::button(get_icon("backward-step")).on_press(ViewerMessage::StepBack).width(40),
            iced::widget::button(get_icon("pause").size(20.0)).on_press(ViewerMessage::Stop).width(40),
            iced::widget::button(get_icon("forward-step")).on_press(ViewerMessage::StepFwd).width(40),
            iced::widget::button(get_icon("play")).on_press(ViewerMessage::Forward).width(40),

                            // iced::widget::button("Reset").on_press(ViewerMessage::Reset).width(225),
            iced::widget::button(get_icon("forward-fast")).on_press(ViewerMessage::JumpToEnd).width(40),
            //iced::widget::button(">>").width(40),
        ],


    ).center_x(iced::Length::Shrink).center_y(iced::Length::Shrink).width(iced::Length::Fill)
}

#[derive(Clone,Copy,Debug)]
pub enum PlayState{
    Stop,
    Forward,
    Backward
}



pub struct Worker<T>{
    pub worker: Option<thread::JoinHandle<()>>,
    controller:mpsc::Sender<bool>,
    pub feedback: mpsc::Receiver<T>,
    is_stopping:bool
}

impl<T> Worker<T>{
    pub fn new(worker: thread::JoinHandle<()>, controller:mpsc::Sender<bool>, feedback: mpsc::Receiver<T>)->Self{
        Self{
            worker:Some(worker),controller,feedback,is_stopping:false
        }
    }
    pub fn request_stop(&mut self){
        if let Some(worker) = &self.worker{
            if !worker.is_finished() && !self.is_stopping{
                if let Err(e) = self.controller.send(true){
                    println!("{:?}",e);
                }
                self.is_stopping = true;
            }
        }
    }
    pub fn is_finished(&self)->bool{
        if let Some(worker) = &self.worker{
            worker.is_finished()
        }
        else {
            true
        }
    }

    pub fn stop(&mut self){
        if let Some(worker) = self.worker.take(){
            if let Err(e) = worker.join(){
                println!("{:?}",e);
            }
            else{
                println!("Process successfully stopped");
            }
        }
    }
}

pub struct PadamoViewer{
    chart:Detector<PadamoAppMessage>,
    length:usize,
    pointer:usize,
    start:usize,
    end:usize,
    start_str:String,
    end_str:String,
    pointer_str:String,
    datetime_entry:String,

    min_signal_entry:String,
    max_signal_entry:String,
    is_autoscale:bool,

    signal:Option<padamo_api::lazy_array_operations::LazyTriSignal>,
    //signal:Option<(padamo_api::lazy_array_operations::LazyDetectorSignal,padamo_api::lazy_array_operations::LazyTimeSignal,Option<padamo_api::lazy_array_operations::LazyTrigger>)>,
    buffer:Option<(padamo_api::lazy_array_operations::ndim_array::ArrayND<f64>,f64)>,
    playstate:PlayState,
    plot_scale:padamo_detectors::Scaling,

    form:form::ViewerFormBuffer,
    form_instance:ViewerForm,


    animator:Option<Worker<String>>,
    exporter:Option<Worker<String>>,
    animation_status: String,
    export_status:String,

    file_changed:bool,
    view_transform: crate::transform_widget::TransformState,

}

impl PadamoViewer{
    fn run_export(&mut self, padamo:crate::application::PadamoStateRef){
        if let Some(filename) = padamo.workspace.workspace("viewed_hdf5_data").save_dialog(vec![("HDF5 data", vec!["h5"])]){
            //if let nfd::Response::Okay(filename) = res{
            self.stop_exporter();
            if let Some(signal_ref) = &self.signal{
                let spatial:padamo_api::lazy_array_operations::LazyDetectorSignal = signal_ref.0.clone();
                let temporal:padamo_api::lazy_array_operations::LazyTimeSignal = signal_ref.1.clone();
                let start = self.start;
                let end = self.end+1;
                let mut testframe = spatial.request_range(0,1);
                testframe.shape.drain(0..1); //Remove time axis
                let frame_shape = testframe.shape;
                let settings = self.form_instance.export.clone();
                println!("{:?}",settings);
                if settings.spatialfield.is_empty(){
                    padamo.show_error("Signal field is not specified");
                    return;
                }
                if settings.temporalfield.is_empty(){
                    padamo.show_error("Time field is not specified");
                    return;
                }

                if settings.temporalfield==settings.spatialfield{
                    padamo.show_error("Signal and time must be different");
                    return;
                }

                if settings.rampart>=1.0 || settings.rampart<=0.0{
                    padamo.show_error("Ram part must be in (0,1) interval");
                    return;
                }


                let mut sys = System::new_with_specifics(RefreshKind::new().with_memory(MemoryRefreshKind::new().with_ram()));
                sys.refresh_memory();
                let allowed_memory = ((sys.total_memory() as f64)*settings.rampart) as usize;
                println!("Allowed usage of {} bytes",allowed_memory);
                if allowed_memory==0{
                    padamo.show_error("No memory available");
                    return;
                }

                let sample = spatial.request_range(0,1);
                let sample_size = sample.flat_data.len()*8; // (Flat buffer of f64 (8 bytes each))

                let mut chunk_size:Vec<usize> = sample.shape.clone().into();
                chunk_size[0] = settings.chunk;

                println!("Frame size: {} bytes",sample_size);

                let quota = allowed_memory/sample_size;

                println!("Quota: {} samples",quota);


                let (tx,rx) = mpsc::channel::<bool>();

                let (tx_status,rx_status) = mpsc::channel::<String>();

                let handle = thread::spawn(move || {
                        tx_status.send("Estimating frame size".into()).unwrap();

                        let mut size_up = end-start;
                        let mut size_down = 0;
                        let mut size_mid = (size_up+size_down)/2;
                        while size_mid != size_down && size_mid != size_up {

                            let overhead = spatial.calculate_overhead(start,start+size_mid);
                            if overhead>=quota{
                                size_up = size_mid;
                            }
                            else if overhead<=quota{
                                size_down = size_mid;
                            }
                            size_mid = (size_up+size_down)/2;
                        }
                        let step = usize::max(size_mid,1);
                        let overhead = spatial.calculate_overhead(start,start+step);
                        println!("Estimated step: {} (overhead {})",step, overhead);

                        if let Ok(file) = hdf5::File::create(filename){
                            let mut ds_shape = vec![end-start];
                            ds_shape.extend(frame_shape.clone());

                            let mut space_ds = file.new_dataset::<f64>()
                                .chunk(chunk_size)
                                .shape(ds_shape);
                            if settings.deflate{
                                space_ds = space_ds.deflate(settings.deflatelevel);
                            }

                            let space_ds = space_ds.create(settings.spatialfield.as_str()).unwrap();

                            let mut time_ds = file.new_dataset::<f64>()
                                .chunk((settings.chunk,))
                                //.chunk((1, 16, 16))
                                .shape((end-start,));

                            if settings.deflate{
                                time_ds = time_ds.deflate(settings.deflatelevel);
                            }

                            let time_ds = time_ds.create(settings.temporalfield.as_str()).unwrap();

                            let mut i = start;
                            //let step = settings.framesstep;
                            while i<end{
                                let size = usize::min(step, end-i);

                                tx_status.send(format!("{}/{}",i-start,end-start)).unwrap();

                                let frame = spatial.request_range(i,i+size).to_ndarray();
                                let tim = temporal.request_range(i,i+size);

                                let mut slabs:Vec<hdf5::SliceOrIndex> = Vec::with_capacity(frame_shape.len());
                                for j in 0..frame_shape.len()+1{
                                    if j==0{
                                        slabs.push((i-start..i-start+size).into());
                                    }
                                    else {
                                        slabs.push((0..frame_shape[j-1]).into());
                                    }
                                }
                                let slicer: hdf5::Hyperslab = slabs.into();

                                space_ds.write_slice(&frame, slicer).unwrap();
                                time_ds.write_slice(&tim, (i-start..i-start+size, )).unwrap();
                                if let Ok(v) = rx.try_recv(){
                                    if v{
                                        println!("Interrupt requested");
                                        break;
                                    }
                                }
                                i+=size;
                            }
                        }

                });
                self.exporter = Some(Worker::new(handle, tx, rx_status));
                //self.exporter = Some((handle,tx,rx_status));
            }
            //}
        }
    }


    fn run_animation(&mut self, padamo:crate::application::PadamoStateRef){
        if let Some(filename) = padamo.workspace.workspace("animations").save_dialog(vec![
            ("MP4 animation", vec!["mp4"]),
            ("GIF animation", vec!["gif"]),
        ]){

            self.stop_animator();
            use plotters::prelude::*;

            // let form_data = if let Some(f) = self.form.get() {f} else {return;};
            let animation_parameters = self.form_instance.animation.clone();

            println!("Animation parameters: {:?}",animation_parameters);
            let plot_scale = self.plot_scale.clone();
            let chart = self.chart.clone();
            if let Some(signal_ref) = &self.signal{
                //let signal = signal_ref.clone();
                let spatial:padamo_api::lazy_array_operations::LazyDetectorSignal = signal_ref.0.clone();
                let temporal:padamo_api::lazy_array_operations::LazyTimeSignal = signal_ref.1.clone();
                let start = self.start;
                let end = self.end+1;
                let height = if animation_parameters.displaylc {animation_parameters.height+animation_parameters.lcheight} else {animation_parameters.height};
                let f:&std::path::Path = filename.as_ref();
                self.animator = if let Some(ext) = f.extension(){
                    if let Some(ext_str) = ext.to_str(){
                        match ext_str {
                            "gif"=>{
                                let backend = BitMapBackend::gif(filename,(animation_parameters.width+80, height), animation_parameters.framedelay);
                                match backend{
                                    Ok(back)=>{Some(animator::animate(back, spatial, temporal, start, end, animation_parameters, chart, plot_scale))}
                                    Err(e)=>{
                                        eprintln!("{}",e);
                                        padamo.show_error(format!("{}",e));
                                        None
                                    }
                                }
                            }
                            "mp4"=>{
                                let backend = VideoBackend::new(filename, animation_parameters.width+80, height,
                                                                plotters_video::FrameDelay::DelayMS(animation_parameters.framedelay as usize));
                                match backend{
                                    Ok(back)=>{Some(animator::animate(back, spatial, temporal, start, end, animation_parameters, chart, plot_scale))}
                                    Err(e)=>{
                                        eprintln!("{}",e);
                                        padamo.show_error(format!("{}",e));
                                        None
                                    }
                                }
                            }
                            ue=>{
                                padamo.show_error(format!("Unsupported extension {}",ue));
                                // eprintln!("Unsupported extension {}",ue);
                                None
                            }
                        }
                    }
                    else{
                        padamo.show_error(format!("Invalid extension {:?}", ext));
                        // eprintln!("Invalid extension {:?}", ext);
                        None
                    }
                }
                else{
                    padamo.show_error("No extension");
                    // eprintln!("No extension");
                    None
                };

            }

        }
        //
    }

    fn stop_animation(&mut self){
        if let Some(v) = &mut self.animator{
            v.request_stop();
        }
    }

    fn stop_export(&mut self){
        if let Some(v) = &mut self.exporter{
            v.request_stop();
        }
    }

    fn make_frame(&mut self, padamo:crate::application::PadamoStateRef){
        let result = padamo.workspace.workspace("plots").save_dialog(vec![
                            ("Portable net graphics", vec!["png"]),
                            ("Lossy compressed JPEG", vec!["jpg"]),
                            ("Scalar vector graphics", vec!["svg"])
                        ]);
        if let Some(path_s) = result{
            println!("Frame path: {}", path_s);
            let path = std::path::Path::new(&path_s);
            self.save_chart(path, padamo);
        }
    }

    fn save_chart(&self, path:&std::path::Path, padamo:crate::application::PadamoStateRef){
        use plotters::prelude::*;
        if let Some(signal_ref) = &self.signal{
                //let signal = signal_ref.clone();
                let spatial:padamo_api::lazy_array_operations::LazyDetectorSignal = signal_ref.0.clone();
                let temporal:padamo_api::lazy_array_operations::LazyTimeSignal = signal_ref.1.clone();
                let width = self.form_instance.single_frame.width;
                let height = self.form_instance.single_frame.height;

                if let Some(ext) = path.extension(){
                    if let Some(ext_str) = ext.to_str(){
                        match ext_str {
                            "png" | "jpg" => {
                                let backend = BitMapBackend::new(&path, (width+80,height));
                                let root = backend.into_drawing_area();
                                animator::make_frame(&root, &spatial, &temporal, self.pointer, &self.chart, self.plot_scale);
                            },
                            "svg" => {
                                let backend = SVGBackend::new(&path, (width+80,height));
                                let root = backend.into_drawing_area();
                                animator::make_frame(&root, &spatial, &temporal, self.pointer, &self.chart, self.plot_scale);
                            },
                            ue=>{
                                padamo.show_error(format!("Unsupported extension {}",ue));
                                // eprintln!("Unsupported extension {}",ue);
                            }
                        }
                    }
                }

                // let backend = BitMapBackend::new(, )

                // let mut frame = spatial.request_range(i,i+1);
                // frame.shape.drain(0..1);
                // let tim = temporal.request_range(i,i+1)[0];
                //
                // root.fill(&WHITE).unwrap();
                // let root_area = plotter.into_drawing_area();
                // root_area.fill(&WHITE).unwrap();
                //let cc = ChartBuilder::on(&root_area);
                // let charter = self.chart.build_chart_generic(root_area, , , , );
                // charter.draw_chart(&(), root_area);
        }


    }
}

impl PadamoViewer{
    pub fn new()->Self{
        // let animation_params = Default::default();
        // let export_params = Default::default();
        let mut res = Self{
            chart:Detector::default_vtl(),
            view_transform:Default::default(),
            length:100,
            pointer:0,
            start:0,
            end:100,
            signal:None,
            buffer:None,
            playstate: PlayState::Stop,
            start_str:"".into(),
            end_str:"".into(),
            pointer_str:"".into(),
            datetime_entry:"".into(),
            min_signal_entry:"".into(),
            max_signal_entry:"".into(),
            is_autoscale:true,
            plot_scale:padamo_detectors::Scaling::Autoscale,
            //plotter_needs_reset:false,
            file_changed:true,
            // animation_fields:Default::default(),
            // animation_parameters: animation_params,

            // export_fields:Default::default(),
            // export_parameters:export_params,
            form:Default::default(),
            form_instance:Default::default(),

            animator:None,
            exporter:None,
            export_status:"IDLE".into(),
            animation_status:"IDLE".into(),

            // stop_on_trigger:false,
        };
        res.fill_strings();
        res
    }

    fn rerun(&mut self, padamo:crate::application::PadamoStateRef)->Option<PadamoAppMessage>{
        if let Some(padamo_api::prelude::Content::DetectorFullData(signal)) = padamo.compute_graph.environment.0.get(crate::builtin_nodes::viewer::VIEWER_SIGNAL_VAR){
            //let signal_w = signal.clone();
            self.signal = Some(signal.clone());
            //self.signal = Some((signal_w.0,signal_w.1,signal_w.2.into()));
            self.length = signal.0.length()-1;
            if self.file_changed{
                self.end = self.length;
                self.file_changed = false;
            }
            self.clamp();
            self.update_buffer(Some(padamo));
            self.fill_strings();

            return Some(PadamoAppMessage::PlotterMessage(super::plotter::messages::PlotterMessage::SyncData {
                start: self.start,
                end: self.end+1,
                pointer: self.pointer,
                force_clear:true,
            }));
        }
        else {
            None
        }
    }

    fn update_buffer(&mut self, padamo:Option<crate::application::PadamoStateRef>){
        self.clamp();
        if let Some(signal) = &self.signal{
            let time_start = Instant::now();
            let mut frame = signal.0.request_range(self.pointer,self.pointer+1);
            frame.shape.drain(0..1);
            let tim = signal.1.request_range(self.pointer,self.pointer+1)[0];
            self.buffer = Some((frame,tim));
            let time_stop = time_start.elapsed();
            if let Some(p) = padamo{
                let t = time_stop.as_millis() as u64;
                p.add_delay_ms = t*3;
            }
            if self.form_instance.stop_on_trigger{
                if let ROption::RSome(v) = &signal.2{
                    let trig = v.request_range(self.pointer,self.pointer+1);
                    let muststop:bool = trig.flat_data.iter().fold(false, |a,b| a||*b);
                    if muststop{
                        println!("Triggered!");
                        self.playstate = PlayState::Stop;
                    }
                }
            }
        }
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

    fn fill_strings(&mut self){
        self.pointer_str = self.pointer.to_string();
        self.start_str = self.start.to_string();
        self.end_str = self.end.to_string();
        if let Some(frame) = &self.buffer{
            let (min,max) = self.plot_scale.get_bounds(&frame.0,&self.chart.alive_pixels);
            self.min_signal_entry = min.to_string();
            self.max_signal_entry = max.to_string();
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

    fn stop_animator(&mut self){
        if let Some(mut a) = self.animator.take(){
            a.stop();
        }
        self.animation_status = "IDLE".into();
    }

    fn stop_exporter(&mut self){
        if let Some(mut e) = self.exporter.take(){
            e.stop();
        }
        self.export_status = "IDLE".into();
    }

    fn update_pixels(&self, padamo :&mut PadamoState, save:bool){
        padamo.compute_graph.environment.0.insert("alive_pixels".into(),Content::DetectorSignal(make_lao_box(self.chart.alive_pixels_mask())));
        //let arr = self.chart.alive_pixels.clone().to_ndarray();
        if save{
            padamo.persistent_state.serialize("viewer_pixels",&self.chart.alive_pixels);
        }
    }
}



impl PadamoTool for PadamoViewer{
    fn view<'a>(&'a self)->iced::Element<'a, crate::messages::PadamoAppMessage> {
        let frame_num = &self.pointer_str;
        let start_frame = &self.start_str;
        let end_frame = &self.end_str;
        let mut frame = None;
        let detector_shape = self.chart.shape();
        if let Some(buffer) = &self.buffer{
            if buffer.0.form_compatible(&detector_shape){
                frame = Some((&buffer.0,buffer.1));
                //println!("OK {:?} and {:?}", detector_shape, buffer.shape);
            }
            else{
                println!("Incompatible shapes {:?} and {:?}", buffer.0.shape, detector_shape);
            }
        }

        let view_transform: iced::Element<'_,_> = self.view_transform.view().width(300).into();
        let lower_col:iced::Element<'a, ViewerMessage> = column![
            //self.chart.view(frame,self.plot_scale, Some(ViewerMessage::plot_pixel(self.start, self.end))),

            row![
                column![
                    row![
                        iced::widget::checkbox("Autoscale",self.is_autoscale).on_toggle(ViewerMessage::SetAutoscale),
                        iced::widget::TextInput::new("Min signal", &self.min_signal_entry).width(100).on_input(ViewerMessage::SetMinSignal),
                        iced::widget::text("-").align_x(iced::alignment::Horizontal::Center).width(100),
                        iced::widget::TextInput::new("Max signal", &self.max_signal_entry).width(100).on_input(ViewerMessage::SetMaxSignal),
                        iced::widget::Space::new(10,10).width(iced::Length::Fill),
                        view_transform.map(ViewerMessage::PlotZoomMessage),
                        // iced::widget::Space::new(10,10).width(iced::Length::Fill),
                    ],
                    iced::widget::Container::new(
                        iced::Element::new(TimeLine::new(self.length,self.pointer, self.start, self.end,Some(ViewerMessage::SetViewPosition))),
                    ).center_x(iced::Length::Fill).center_y(iced::Length::Shrink).width(iced::Length::Fill).height(iced::Length::Fill),
                ],

                iced::widget::container(column![
                    row![
                        iced::widget::TextInput::new("",start_frame.as_str())
                            .on_input(ViewerMessage::SetViewStartText)
                            .on_submit(ViewerMessage::SubmitTimeline)
                            .width(100),
                        iced::widget::TextInput::new("",frame_num.as_str())
                            .on_input(ViewerMessage::SetViewPositionText)
                            .on_submit(ViewerMessage::SubmitTimeline)
                            .width(100),
                        iced::widget::TextInput::new("",end_frame.as_str())
                            .on_input(ViewerMessage::SetViewEndText)
                            .on_submit(ViewerMessage::SubmitTimeline)
                            .width(100),
                    ],
                    row![
                        //iced::widget::button("To start").on_press(ViewerMessage::JumpToStart).width(100),
                        iced::widget::button("Set start").on_press(ViewerMessage::SetStart).width(100),
                        iced::widget::button("Reset").on_press(ViewerMessage::Reset).width(100),
                        iced::widget::button("Set end").on_press(ViewerMessage::SetEnd).width(100),
                        //iced::widget::button("To end").on_press(ViewerMessage::JumpToEnd).width(100),
                    ],
                    // row![
                        iced::widget::TextInput::new("Enter datetime",self.datetime_entry.as_str())
                            .on_input(ViewerMessage::EditDatetime)
                            .on_submit(ViewerMessage::SubmitDatetime)
                            .width(300),
                        make_player_pad().center_x(iced::Length::Shrink)
                    // ],
                ]).style(iced::widget::container::bordered_box),

            ].height(iced::Length::Shrink),
        ].into();
        let lower_col = lower_col.map(crate::messages::PadamoAppMessage::ViewerMessage);

        let start = self.start;
        let end = self.end;

        // if let Some(anim) = &self.animator{
        //     let pip = &anim.2;
        //     if let Ok(v) = pip.try_recv(){
        //         *animation_status = v;
        //     }
        // }
        // else{
        //     *animation_status = "IDLE".into();
        // };


        let settings_column:iced::Element<'_,ViewerMessage> = column![
            // row![
            //     iced::widget::button("Create animation").on_press( ViewerMessage::CreateAnimation),
            //     iced::widget::button("Stop").on_press( ViewerMessage::StopAnimation),
            // ],
            row![iced::widget::text("Animation status:"),iced::widget::text(&self.animation_status)],
            iced::widget::rule::Rule::horizontal(10),
            // row![
            //     iced::widget::button("Export").on_press( ViewerMessage::Export),
            //     iced::widget::button("Stop").on_press(ViewerMessage::StopExport),
            // ],
            row![iced::widget::text("Export status:"),iced::widget::text(&self.export_status)],
            iced::widget::rule::Rule::horizontal(10),
            // iced::widget::checkbox("Stop on trigger", self.stop_on_trigger).on_toggle(ViewerMessage::SetAutostop),
            // iced::widget::rule::Rule::horizontal(10),
            //iced::widget::text("Export settings"),
            //self.export_fields.view().map(ViewerMessage::EditExportSettings),

            //iced::widget::rule::Rule::horizontal(10),
            //iced::widget::text("Animation settings"),
            self.form.view(None).map(ViewerMessage::EditForm),
            //self.animation_fields.view().map(ViewerMessage::EditAnimationSettings),
        ].width(200).into();

        //let action:Option<fn(Vec<usize>)->PadamoAppMessage> = None;
        let top_row = row![
            self.chart.view(frame,self.view_transform.transform(),self.plot_scale,
                            Some(move |x| PadamoAppMessage::PlotterMessage(super::plotter::messages::PlotterMessage::PlotPixel(start, end, x))),
                            Some(move |x| PadamoAppMessage::ViewerMessage(ViewerMessage::TogglePixel(x))),
                            ),
            iced::widget::rule::Rule::vertical(10),
            iced::widget::scrollable(settings_column.map(PadamoAppMessage::ViewerMessage)).width(200),
        ];

        column!(
            top_row,
            //self.chart.view(frame,self.plot_scale, Some(ViewerMessage::plot_pixel(self.start, self.end))),
            lower_col,
        ).into()
    }

    fn tab_name(&self)->String {
        "Viewer".into()
    }

    fn update(&mut self, msg: std::rc::Rc<crate::messages::PadamoAppMessage>, padamo:crate::application::PadamoStateRef) {
        if let crate::messages::PadamoAppMessage::Run = msg.as_ref(){
            self.update_pixels(padamo,true);
        }
        else if let crate::messages::PadamoAppMessage::ViewerMessage(view) = msg.as_ref(){
            let mut request_buffer_fill = true;
            match view {
                ViewerMessage::SetViewPosition(pos)=>{
                    self.pointer = *pos;
                    if self.pointer<self.start{
                        self.start = self.pointer;
                    }
                    if self.pointer>self.end{
                        self.end = self.pointer;
                    }
                },
                ViewerMessage::SetViewPositionUnixTime(f)=>{
                    if let Some(signal) = &self.signal{
                        let pos = crate::time_search::find_unixtime(&signal.1, *f);

                        self.pointer = pos;
                        if self.pointer<self.start{
                            self.start = self.pointer;
                        }
                        if self.pointer>self.end{
                            self.end = self.pointer;
                        }
                    }
                }

                ViewerMessage::SetStart=>{
                    self.start= self.pointer;
                }
                ViewerMessage::JumpToStart=>{
                    self.pointer = self.start;
                }
                ViewerMessage::FocusOn(start, end)=>{
                    self.pointer = *start;
                    self.start = *start;
                    padamo.current_page = 0;
                    self.end = *end;
                }
                ViewerMessage::JumpToEnd=>{
                    self.pointer = self.end;
                }
                ViewerMessage::SetEnd=>{
                    self.end= self.pointer;
                }
                ViewerMessage::Reset=>{
                    self.start = 0;
                    self.end = self.length;
                }
                ViewerMessage::Stop=>{
                    self.playstate = PlayState::Stop;
                }
                ViewerMessage::Forward=>{
                    self.playstate = PlayState::Forward;
                }
                ViewerMessage::Backward=>{
                    self.playstate = PlayState::Backward;
                }
                ViewerMessage::StepBack=>{
                    self.playstate = PlayState::Stop;
                    if self.pointer>0{
                        self.pointer-=1;
                    }
                }
                ViewerMessage::StepFwd=>{
                    self.playstate = PlayState::Stop;
                    if self.pointer<self.length{
                        self.pointer+= 1;
                    }
                }
                ViewerMessage::SetViewPositionText(txt)=>{
                    self.pointer_str = txt.clone();
                    request_buffer_fill = false;
                }
                ViewerMessage::SetViewStartText(txt)=>{
                    self.start_str = txt.clone();
                    request_buffer_fill = false;
                }
                ViewerMessage::SetViewEndText(txt)=>{
                    self.end_str = txt.clone();
                    request_buffer_fill = false;
                }
                ViewerMessage::SubmitTimeline=>{
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
                }
                ViewerMessage::EditDatetime(s)=>{
                    self.datetime_entry = s.clone();
                }
                // ViewerMessage::EditAnimationSettings(v)=>{
                //     self.animation_fields.update(v.clone(),&mut self.animation_parameters);
                // }
                // ViewerMessage::EditExportSettings(v)=>{
                //     self.export_fields.update(v.clone(), &mut self.export_parameters);
                // }
                ViewerMessage::EditForm(v)=>{
                    match v{
                        ActionOrUpdate::Action(a)=>{
                            if let Some(action) =  a.as_ref().downcast_ref::<form::ViewerActions>(){
                                match action {
                                    form::ViewerActions::Noop=>{},
                                    form::ViewerActions::StartAnimation=>{self.run_animation(padamo);},
                                    form::ViewerActions::StopAnimation=>{self.stop_animation();},
                                    form::ViewerActions::StartExport=>{self.run_export(padamo);},
                                    form::ViewerActions::StopExport=>{self.stop_export();},
                                    form::ViewerActions::MakeFrame=>{self.make_frame(padamo);},
                                }
                            }
                        },
                        ActionOrUpdate::Update(u)=>{
                            self.form.update(u.to_owned());
                            match self.form.get(){

                                Ok(v) =>self.form_instance = v,
                                Err(e)=>eprintln!("Form get error: {}",e),
                            }

                        }
                    }
                }

                ViewerMessage::SubmitDatetime=>{
                    println!("Submitted DT {}",self.datetime_entry);
                    if let Some(signal) = &self.signal{

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
                }
                ViewerMessage::SetAutoscale(v)=>{
                    self.is_autoscale = *v;
                    self.update_scale();
                    //request_buffer_fill = false;
                }
                ViewerMessage::SetMinSignal(s)=>{
                    self.min_signal_entry = s.clone();
                    self.update_scale();
                    request_buffer_fill = false;
                }
                ViewerMessage::SetMaxSignal(s)=>{
                    self.max_signal_entry = s.clone();
                    self.update_scale();
                    request_buffer_fill = false;
                }
                // ViewerMessage::SetAutostop(v)=>{
                //     self.stop_on_trigger = *v;
                // }
                ViewerMessage::TogglePixel(pix)=>{
                    self.chart.toggle_pixel(pix);
                    self.update_pixels(padamo,true);
                }


                ViewerMessage::PlotZoomMessage(msg)=>{
                    self.view_transform.update(msg.to_owned());
                }
            }

            self.update_buffer(Some(padamo));
            if request_buffer_fill{
                self.fill_strings();
            }
        }
        else if let crate::messages::PadamoAppMessage::ClearState = msg.as_ref(){
            self.initialize(padamo);
        }
        else if let crate::messages::PadamoAppMessage::SetDetector(v) = msg.as_ref(){
            self.chart = Detector::from_cells(v.clone());
            println!("Viewer has new detector loaded");
        }
        else if let crate::messages::PadamoAppMessage::Tick = msg.as_ref(){
            match self.playstate {
                PlayState::Stop=>(),
                PlayState::Forward=>{
                    if self.pointer<self.end{
                        self.pointer += 1;
                        self.update_buffer(Some(padamo));
                        self.fill_strings();
                    }
                    else{
                        self.playstate = PlayState::Stop;
                    }
                }
                PlayState::Backward=>{
                    if self.pointer>self.start{
                        self.pointer-=1;
                        self.update_buffer(Some(padamo));
                        self.fill_strings();
                    }
                    else{
                        self.playstate = PlayState::Stop;
                    }
                }
            }
        }
    }

    fn late_update(&mut self, msg: std::rc::Rc<crate::messages::PadamoAppMessage>, padamo:crate::application::PadamoStateRef)->Option<PadamoAppMessage> {
        if let crate::messages::PadamoAppMessage::Run = msg.as_ref(){
            if let Some(v) = self.rerun(padamo){
                return Some(v);
            }
        }

        if let crate::messages::PadamoAppMessage::RerollRun = msg.as_ref(){
            if let Some(v) = self.rerun(padamo){
                return Some(v);
            }
        }


        if let PadamoAppMessage::Tick = msg.as_ref(){
            let mut will_stop = false;
            if let Some(anim) = &self.animator{
                if anim.is_finished(){
                    will_stop = true;
                }
                let pip = &anim.feedback;
                while let Ok(v) = pip.try_recv(){
                    self.animation_status = v;
                }
            }
            if will_stop{
                self.stop_animator();
            }

            let mut will_stop = false;
            if let Some(exp) = &self.exporter{
                if exp.is_finished(){
                    will_stop = true;
                }
                let pip = &exp.feedback;
                while let Ok(v) = pip.try_recv(){
                    self.export_status = v;
                }
            }
            if will_stop{
                self.stop_exporter();
            }

            if let PlayState::Stop = self.playstate{
                None
            }
            else{
                Some(PadamoAppMessage::PlotterMessage(super::plotter::messages::PlotterMessage::SyncData { start: self.start, end: self.end+1, pointer: self.pointer, force_clear:false }))
                //Some(PadamoAppMessage::PlotterMessage(super::plotter::messages::PlotterMessage::SyncPointer(self.pointer)))
            }
        }

        else if let PadamoAppMessage::ViewerMessage(_) = msg.as_ref(){
            // if self.plotter_needs_reset{
            //     self.plotter_needs_reset = false;
            //     Some(PadamoAppMessage::PlotterMessage(super::plotter::messages::PlotterMessage::LazySelectData(self.start, self.end+1)))
            // }
            // else{
                //Some(PadamoAppMessage::PlotterMessage(super::plotter::messages::PlotterMessage::SyncPointer(self.pointer)))
            //}
            Some(PadamoAppMessage::PlotterMessage(super::plotter::messages::PlotterMessage::SyncData { start: self.start, end: self.end+1, pointer: self.pointer, force_clear:false }))
        }
        else {
            None
        }
    }

    fn context_update(&mut self, msg: std::rc::Rc<PadamoAppMessage>, padamo:crate::application::PadamoStateRef) {
        match msg.as_ref() {
            PadamoAppMessage::Open=>{
                let open_res = padamo.workspace.workspace("viewed_hdf5_data")
                    .open_dialog(vec![
                        ("HDF5, MATLAB 7.3 data",vec!["h5", "mat"]),
                        ("old MATLAB data",vec!["mat"]),
                        ("Cern ROOT data",vec!["root"]),
                        ("Plain text", vec!["csv", "tsv", "txt"])
                    ]);
                if let Some(file_path) = open_res{
                    padamo.compute_graph.environment.0.insert(crate::builtin_nodes::viewer::VIEWER_FILENAME_VAR.into(), file_path.into());
                    self.file_changed = true;
                }
            },
            _=>()
        }
    }

    fn initialize(&mut self, padamo:crate::application::PadamoStateRef) {
        let mask_loaded: Option<padamo_api::lazy_array_operations::ArrayND<bool>> = padamo.persistent_state.deserialize("viewer_pixels");
        if let Some(mask) = mask_loaded{
            if mask.form_compatible(self.chart.shape()){
                self.chart.alive_pixels = mask;
                self.update_pixels(padamo,false);
                return;
            }
        }
        self.chart = Detector::default_vtl();
        //self.chart.alive_pixels = ArrayND::new(self.chart.shape().clone(), true);
    }
}

