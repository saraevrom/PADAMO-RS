mod messages;
mod animator;
mod form;
mod detector_display;
pub mod cross_progress;
mod norm_entry;

use super::PadamoTool;
use iced::widget::text::Catalog;
use iced_font_awesome::FaIcon;
use padamo_api::calculation_nodes::content::Content;
use padamo_api::lazy_array_operations::make_lao_box;
use padamo_detectors::{DetectorAndMask, DetectorPlotter};
use plotters_video::VideoBackend;
use crate::application::PadamoState;
use crate::custom_widgets::timeline::TimeLine;
use crate::detector_muxer::get_signal_var;
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

// use iced_font_awesome::fa_icon_solid;

// fn get_icon<Theme:Catalog>(icon:&'static str)->FaIcon<'static,Theme>{
//     fa_icon_solid(icon).size(20.0).color(iced::color![255,255,255])
// }

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

    form:form::ViewerFormBuffer,
    form_instance:ViewerForm,


    animator:Option<Worker<String>>,
    exporter:Option<Worker<String>>,
    animation_status: String,
    export_status:String,

    file_changed:bool,
    // view_transform: crate::transform_widget::TransformState,

    window_view:detector_display::SingleDetectorDisplay<PadamoAppMessage>,
    playbar_state: cross_progress::CrossProgress,
}

impl PadamoViewer{
    fn run_export(&mut self, padamo:crate::application::PadamoStateRef){
        let (start, end) = if let Some(v) = self.playbar_state.get_interval(padamo, self.window_view.get_id()) {v} else {return;};
        if let Some(filename) = padamo.workspace.workspace("viewed_hdf5_data").save_dialog(vec![("HDF5 data", vec!["h5"])]){
            //if let nfd::Response::Okay(filename) = res{
            self.stop_exporter();
            // let detector = padamo.detectors.get_primary();

            if let Some(signal_ref) = self.window_view.try_get_signal(padamo){
                let spatial:padamo_api::lazy_array_operations::LazyDetectorSignal = signal_ref.0.clone();
                let temporal:padamo_api::lazy_array_operations::LazyTimeSignal = signal_ref.1.clone();

                let start = start;
                let end = end+1;
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
            }
            //}
        }
    }

    // fn get_detector<'a>(&'a self, padamo:&'a PadamoState)->Option<&'a DetectorAndMask>{
    //     let res = padamo.detectors.get_primary();
    //     res
    // }

    fn run_animation(&mut self, padamo:crate::application::PadamoStateRef){
        let (start, end) = if let Some(v) = self.playbar_state.get_interval(padamo, 0) {v} else {return;};
        if let Some(filename) = padamo.workspace.workspace("animations").save_dialog(vec![
            ("MP4 animation", vec!["mp4"]),
            ("GIF animation", vec!["gif"]),
        ]){

            self.stop_animator();
            use plotters::prelude::*;

            let animation_parameters = self.form_instance.animation.clone();

            println!("Animation parameters: {:?}",animation_parameters);
            let chart = DetectorPlotter::new();

            //TODO: Make proper multidetector
            //let detector = if let Some(det) = self.get_detector(padamo){det} else {return;};
            let detector = if let Some(det) = padamo.detectors.get(self.window_view.get_id()){det} else {return;};

            let primary = padamo.compute_graph.environment.0.get(get_signal_var(0).as_str());
            let time_primary = if let Some(padamo_api::prelude::Content::DetectorFullData(signal_p)) = primary{
                signal_p.1.clone()
            }
            else{
                return;
            };


            if let Some(signal_ref) = self.window_view.try_get_signal(padamo){
                let spatial:padamo_api::lazy_array_operations::LazyDetectorSignal = signal_ref.0.clone();
                let temporal:padamo_api::lazy_array_operations::LazyTimeSignal = signal_ref.1.clone();
                let start = start;
                let end = end+1;
                let plot_scale = self.window_view.get_scale();
                let height = if animation_parameters.displaylc {animation_parameters.height+animation_parameters.lcheight} else {animation_parameters.height};
                let f:&std::path::Path = filename.as_ref();
                self.animator = if let Some(ext) = f.extension(){
                    if let Some(ext_str) = ext.to_str(){
                        match ext_str {
                            "gif"=>{
                                let backend = BitMapBackend::gif(filename,(animation_parameters.width+80, height), animation_parameters.framedelay);
                                match backend{
                                    Ok(back)=>{Some(animator::animate(back, spatial, temporal, time_primary, start, end, animation_parameters, chart, detector.clone(), plot_scale))}
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
                                    Ok(back)=>{Some(animator::animate(back, spatial, temporal, time_primary, start, end, animation_parameters, chart, detector.clone(), plot_scale))}
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

        //TODO: Make proper multidetector
        // let detector = if let Some(det) = self.get_detector(padamo){det} else {return;};
        let detector = if let Some(det) = padamo.detectors.get(self.window_view.get_id()) {det} else {return;};
        let pointer = if let Some(v) = self.playbar_state.get_frame(&padamo, 0) {v} else {return;};

        let primary = padamo.compute_graph.environment.0.get(get_signal_var(0).as_str());
        let time_primary = if let Some(padamo_api::prelude::Content::DetectorFullData(signal_p)) = primary{
            signal_p.1.clone()
        }
        else{
            return;
        };

        let plot_scale = self.window_view.get_scale();

        if let Some(signal_ref) = self.window_view.try_get_signal(padamo){
                //let signal = signal_ref.clone();
                let spatial:padamo_api::lazy_array_operations::LazyDetectorSignal = signal_ref.0.clone();
                let temporal:padamo_api::lazy_array_operations::LazyTimeSignal = signal_ref.1.clone();
                let width = self.form_instance.single_frame.width;
                let height = self.form_instance.single_frame.height;
                let chart = DetectorPlotter::new();
                if let Some(ext) = path.extension(){
                    if let Some(ext_str) = ext.to_str(){
                        match ext_str {
                            "png" | "jpg" => {
                                let backend = BitMapBackend::new(&path, (width+80,height));
                                let root = backend.into_drawing_area();
                                animator::make_frame(&root, &spatial, &temporal, &time_primary, pointer, &chart, detector, plot_scale);
                            },
                            "svg" => {
                                let backend = SVGBackend::new(&path, (width+80,height));
                                let root = backend.into_drawing_area();
                                animator::make_frame(&root, &spatial, &temporal, &time_primary, pointer, &chart, detector, plot_scale);
                            },
                            ue=>{
                                padamo.show_error(format!("Unsupported extension {}",ue));
                                // eprintln!("Unsupported extension {}",ue);
                            }
                        }
                    }
                }

        }


    }
}

impl PadamoViewer{
    pub fn new()->Self{

        let res = Self{

            file_changed:true,

            form:Default::default(),
            form_instance:Default::default(),

            animator:None,
            exporter:None,
            export_status:"IDLE".into(),
            animation_status:"IDLE".into(),
            window_view: detector_display::SingleDetectorDisplay::new(0),
            playbar_state: cross_progress::CrossProgress::new(0),
            // stop_on_trigger:false,
        };
        // res.fill_strings();
        res
    }

    fn rerun(&mut self, padamo:crate::application::PadamoStateRef)->Option<PadamoAppMessage>{
        if let Some(rep) = self.playbar_state.update_signal_info(padamo){
            self.update_buffer(Some(padamo));
            self.fill_strings(padamo);
            Some(PadamoAppMessage::NewPlotterMessage(super::plotter_new::messages::NewPlotterMessage::SyncData {
                start: rep.start,
                end: rep.end+1,
                pointer: Some(rep.pointer),
                poked_pixel: None,
            }))
            // Some(PadamoAppMessage::PlotterMessage(super::plotter::messages::PlotterMessage::SyncData {
            //     start: rep.start,
            //     end: rep.end+1,
            //     pointer: rep.pointer,
            //     force_clear:true,
            // }))
        }
        else{
            None
        }
    }

    fn update_buffer(&mut self, padamo:Option<crate::application::PadamoStateRef>){

        let time_start = Instant::now();

        if let Some(p) = &padamo{
            self.window_view.pump_frame(p, &self.playbar_state);
            // self.window_view.set_frame(self.pointer, p);
        }
        let time_stop = time_start.elapsed();
        if let Some(p) = padamo{
            let t = time_stop.as_millis() as u64;
            p.add_delay_ms = t*3;
        }

    }


    fn fill_strings(&mut self, padamo:crate::application::PadamoStateRef){
        self.playbar_state.fill_strings();
        self.window_view.fill_strings(&padamo);
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

    // fn update_pixels(&self, padamo :&mut PadamoState, save:bool){
    //     // let detector = if let Some(det) = self.get_detector(padamo){det} else {return;};
    //     let detector = if let Some(det) = padamo.detectors.get(self.window_view.get_id()){det} else {return;};
    //     let mask = detector.alive_pixels_mask();
    //     if save{
    //         padamo.persistent_state.serialize("viewer_pixels",&mask);
    //     }
    //     padamo.compute_graph.environment.0.insert("alive_pixels".into(),Content::DetectorSignal(make_lao_box(mask)));
    //     //let arr = self.chart.alive_pixels.clone().to_ndarray();
    //
    // }
}



impl PadamoTool for PadamoViewer{
    fn view<'a>(&'a self, padamo:&'a PadamoState)->iced::Element<'a, crate::messages::PadamoAppMessage> {

        let lower_col:iced::Element<'a, ViewerMessage> = self.playbar_state.view(padamo).map(ViewerMessage::TimeLine);

        let lower_col = lower_col.map(crate::messages::PadamoAppMessage::ViewerMessage);




        let settings_column:iced::Element<'_,ViewerMessage> = column![

            row![iced::widget::text("Animation status:"),iced::widget::text(&self.animation_status)],
            iced::widget::rule::Rule::horizontal(10),

            row![iced::widget::text("Export status:"),iced::widget::text(&self.export_status)],
            iced::widget::rule::Rule::horizontal(10),

            self.form.view(None).map(ViewerMessage::EditForm),
        ].width(200).into();

        // let a1 = if self.window_view.is_primary(){
        let id = self.window_view.get_id();
        let a1 = if let Some((a,b)) = self.playbar_state.get_interval(padamo, 0){
            //Some(move |x| PadamoAppMessage::PlotterMessage(super::plotter::messages::PlotterMessage::PlotPixel(a, b, x)))
            Some(move |x| PadamoAppMessage::NewPlotterMessage(super::plotter_new::messages::NewPlotterMessage::SyncData {
                start: a,
                end: b+1,
                pointer: None,
                poked_pixel: Some(crate::tools::plotter_new::messages::PokedPixel {
                    detector_id: id, pixel_id: x
                }),
            }))
        }
        else{
            None
        };
        // }
        // else{
        //     None
        // };
        // let a2 = if self.window_view.is_primary(){
        //     Some(move |x| PadamoAppMessage::ViewerMessage(ViewerMessage::TogglePixel(x)))
        // }
        // else{
        //     None
        // };

        let top_row = row![



            self.window_view.view(padamo, |x| PadamoAppMessage::ViewerMessage(ViewerMessage::WindowView(x)),
                                a1,
                                ),

            iced::widget::rule::Rule::vertical(10),
            iced::widget::scrollable(settings_column.map(PadamoAppMessage::ViewerMessage)).width(200),
        ];

        column!(
            top_row,
            lower_col,
        ).into()
    }

    fn tab_name(&self)->String {
        "Viewer".into()
    }

    fn update(&mut self, msg: std::rc::Rc<crate::messages::PadamoAppMessage>, padamo:crate::application::PadamoStateRef) {
        if let crate::messages::PadamoAppMessage::Run = msg.as_ref(){
            self.window_view.update_pixels(padamo,true);
        }
        else if let crate::messages::PadamoAppMessage::ViewerMessage(view) = msg.as_ref(){
            // let mut request_buffer_fill = true;
            match view {
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
                // ViewerMessage::TogglePixel(pix)=>{
                //     if let Some(det) = padamo.detectors.get_primary_mut(){
                //         det.toggle_pixel(pix);
                //         self.update_pixels(padamo,true);
                //     }
                //
                // }
                // ViewerMessage::ResetMask=>{
                //     if let Some(det) = padamo.detectors.get_primary_mut(){
                //         det.reset_mask();
                //         self.update_pixels(padamo,true);
                //     }
                // }
                ViewerMessage::WindowView(msg)=>{
                    self.window_view.update(msg.to_owned(), padamo);
                },
                ViewerMessage::TimeLine(msg)=>{
                    self.playbar_state.update(msg.to_owned(), padamo);
                }

            }

            self.update_buffer(Some(padamo));
            // if request_buffer_fill{
            //     self.fill_strings(padamo);
            // }
        }
        else if let crate::messages::PadamoAppMessage::ClearState = msg.as_ref(){
            self.initialize(padamo);
        }
        else if let crate::messages::PadamoAppMessage::Tick = msg.as_ref(){
            if self.playbar_state.run_tick(){
                self.update_buffer(Some(padamo));
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

            self.playbar_state.get_sync_message(true)
        }

        else if let PadamoAppMessage::ViewerMessage(_) = msg.as_ref(){

            self.playbar_state.get_sync_message(false)
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
        self.window_view.initialize(&padamo);
    }
}

