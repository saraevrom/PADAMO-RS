use std::cell::RefCell;

use padamo_api::lazy_array_operations::ArrayND;
use padamo_detectors::{DetectorChartMap, DetectorChart, DetectorPlotter};
use padamo_iced_forms::{IcedForm, IcedFormBuffer};
use plotters::{coord::{types::RangedCoordf64, ReverseCoordTranslate, Shift}, prelude::Cartesian2d};
use plotters_iced::{
    sample::lttb::{DataPoint, LttbSource}, Chart, ChartBuilder, ChartWidget, DrawingArea, DrawingBackend, Renderer
};

use iced::{widget::canvas::Cache, Length};

use crate::{application::PadamoState, messages::PadamoAppMessage, tools::{plotter_new::messages::NewPlotterMessage, viewer::{cross_progress::CrossProgressMessage, ViewerMessage}}};
use super::form::LCSelection;

fn find_unixtime(haystack: &[f64],unixtime:f64)->Option<usize>{
    //let unixtime:f64 = (dt.naive_utc().timestamp_millis() as f64)
    let mut start:usize = 0;
    let op_length = haystack.len();
    let mut end:usize = op_length;
    let mut middle:usize = (start+end)/2;
    if op_length==0{
        return None;
    }

    if unixtime>haystack[end-1]{
        return None;
    }
    if unixtime<haystack[0]{
        return None;
    }
    while start != middle{
        let item = haystack[middle];
        if item<=unixtime{
            start = middle;
        }
        if item>=unixtime{
            end = middle;
        }
        middle = (start+end)/2;
    }
    //println!("Datetime search result. req: {}, actual: {}",unixtime, op.request_item(middle));
    let mut res = middle;
    if middle>0{
        let a = haystack[middle-1];
        let b = haystack[middle];
        if (a-unixtime).abs()<(b-unixtime).abs(){
            res = middle-1;
        }
    }
    if middle<op_length-1{
        let a = haystack[middle];
        let b = haystack[middle+1];
        if (a-unixtime).abs()>(b-unixtime).abs(){
            res = middle+1;
        }
    }
    Some(res)
}


#[derive(Clone, Debug)]
pub enum SubplotterMessage{
    TogglePixel(Vec<usize>),
    Transform(crate::transform_widget::TransformMessage),
    PlotXClicked(f64),
    Clear
}

pub struct Subplotter{
    pixels:Option<ArrayND<bool>>,
    //display_mode:DisplayMode,
    settings:super::form::PlotSettings,
    override_range: Option<(f64,f64)>,
    cache: Cache,
    displaying_signal:Option<super::loader::StoredSignal>,
    // y_range: Option<(f64,f64)>,
    last_detector:Option<padamo_detectors::DetectorAndMask>,
    plotter: padamo_detectors::DetectorPlotter<SubplotterMessage>,
    transform: crate::transform_widget::TransformState,
    pointer: Option<usize>,
    plot_spec:RefCell<Option<Cartesian2d<RangedCoordf64, RangedCoordf64>>>,
}

impl Subplotter{
    pub fn new()->Self{
        Self {
            pixels: None,
            override_range: None,
            // y_range:None,
            cache: Cache::new(),
            displaying_signal: None,
            settings:Default::default(),
            // display_mode:DisplayMode::Seconds,
            last_detector:None,
            plotter: padamo_detectors::DetectorPlotter::new(),
            transform: Default::default(),
            pointer:None,
            plot_spec:RefCell::new(None)
        }
    }

    pub fn clear_cache(&mut self){
        self.cache.clear();
    }

    pub fn set_pointer(&mut self, frame:Option<usize>){
        // let new_value = Some(frame);
        if self.pointer != frame{
            self.cache.clear();
        }
        self.pointer = frame;
    }

    pub fn get_pointer_unixtime(&self)->Option<f64>{
        if let Some(i) = self.pointer{
            if let Some(s) = &self.displaying_signal{
                if i>=s.start_frame && (i-s.start_frame)<s.time.len(){
                    Some(s.time[i-s.start_frame])
                }
                else{
                    None
                }
            }
            else{
                None
            }
        }
        else{
            None
        }
    }

    pub fn set_pointer_unixtime(&mut self, unixtime: Option<f64>){
        if let Some(ut) = unixtime{
            if let Some(x) = &self.displaying_signal{
                // use abi_stable::std_types::RVec;
                if let Some(i) = find_unixtime(&x.time, ut){
                    self.set_pointer(Some(i+x.start_frame));
                }
            }
            else{
                self.set_pointer(None);
            }
        }
        else{
            self.set_pointer(None);
        }
    }

    pub fn get_mutable_detector_info<'a>(&'a mut self)-> (&'a mut Option<padamo_detectors::DetectorAndMask>, &'a mut Option<ArrayND<bool>>){
        (&mut self.last_detector, &mut self.pixels)
    }

    pub fn get_mutable_mask_info<'a>(&'a mut self)-> (&'a Option<super::loader::StoredSignal>, &'a mut Option<ArrayND<bool>>){
        (&self.displaying_signal, &mut self.pixels)
    }

    pub fn set_data(&mut self, signal:Option<super::loader::StoredSignal>, last_detector:Option<padamo_detectors::DetectorAndMask>){
        // if self.y_range.is_none(){
        //     if let Some(s) = &signal{
        //         self.y_range = Some((
        //             *s.signals.flat_data.iter().min_by(|a,b| a.total_cmp(b)).unwrap_or(&0.0),
        //             *s.signals.flat_data.iter().max_by(|a,b| a.total_cmp(b)).unwrap_or(&1e-3)
        //         ));
        //     }
        // }
        self.displaying_signal = signal;
        self.cache.clear();
        self.last_detector = last_detector;
        let shape = if let Some(det) = &self.last_detector{
            Some(det.cells.compat_shape.clone())
        }
        else{
            None
        };

        // Borrow checker shenanigans. Sorry.
        if let Some(s) = shape{
            self.sync_detector(Some(&s));
        }
        else{
            self.sync_detector(None);
        }
    }

    fn sync_detector(&mut self, detector_shape:Option<&[usize]>){
        if let Some(det) = detector_shape{
            self.sync_detector_present(det);
            self.cache.clear();
        }
        else{
            self.pixels = None;
            self.cache.clear();
        }
    }

    pub fn has_data(&self)->bool{
        self.displaying_signal.is_some()
    }

    fn sync_detector_present(&mut self, detector_shape:&[usize]){
        let matching = if let Some(pixels) = &self.pixels{
            if pixels.shape.len() == detector_shape.len(){
                pixels.shape.iter().zip(detector_shape.iter())
                .map(|(a,b)| a == b)
                .fold(true, |a,b| a && b)
            }
            else{
                false
            }
        }
        else{
            false
        };
        if !matching{
            self.pixels = Some(ArrayND::new(detector_shape.into(), false));
        }
    }

    pub fn view(&self)->iced::Element<'_, SubplotterMessage>{
        if self.has_data(){
            let chart = ChartWidget::new(self).width(Length::Fill).height(Length::Fill);
            chart.into()
        }
        else{
            iced::widget::container(iced::widget::text("No data"))
                .center(Length::Fill)
                .into()
        }
    }

    pub fn view_mask(&self)->iced::Element<'_, SubplotterMessage>{
        if let Some(pix) = &self.pixels{
            let action:Option<fn(Vec<usize>)->SubplotterMessage> = None;
            let det = if let Some(det) = &self.last_detector {Some(det)} else {None};

            let transformer:iced::Element<'_,_> = self.transform.view().into();
            iced::widget::column![
                self.plotter.view_map_simple(det, pix, Some(self.transform.transform()), Some(SubplotterMessage::TogglePixel), action),
                iced::widget::container(
                    iced::widget::row![
                        iced::widget::button("Clear").on_press(SubplotterMessage::Clear),
                        iced::widget::Space::new(10,10).width(iced::Length::Fill),
                        transformer.map(SubplotterMessage::Transform),
                    ]
                ).center_x(iced::Length::Fill)
            ].into()
        }
        else{
            iced::widget::text!("No pixelmap").into()
        }
    }

    pub fn set_pixel(&mut self, id:&[usize], value:bool){
        if let Some(pixels) = &mut self.pixels{
            if pixels.index_compatible(id){
                pixels[id] = value;
                self.cache.clear();
            }
        }
    }

    pub fn update(&mut self, msg: SubplotterMessage, padamo:&mut PadamoState){
        match msg{
            SubplotterMessage::TogglePixel(id)=>{
                        if let Some(pixels) = &mut self.pixels{
                            if pixels.index_compatible(&id){
                                pixels[&id] = !pixels[&id];
                                self.cache.clear();
                            }
                        }
                    },
            SubplotterMessage::Clear=>{
                        if let Some(pixels) = &mut self.pixels{
                            pixels.flat_data.iter_mut().for_each(|x| *x = false);
                        }
                    },
            SubplotterMessage::Transform(transform_message) => {
                self.transform.update(transform_message);
            },
            SubplotterMessage::PlotXClicked(_)=>(),
        }
    }

    fn get_x_scale(&self)->Option<(f64,f64)>{
        let signal = if let Some(x) = &self.displaying_signal {x} else {return None;};
        if self.settings.display_mode.is_temporal(){
            if let Some(s) = self.override_range{
                Some(s)
            }
            else{
                let start = signal.time[0];
                let end = signal.time[signal.time.len()-1];
                Some((start,end))
            }
        }
        else{
            Some((signal.start_frame as f64, (signal.time.len()+signal.start_frame) as f64))
        }
    }

    pub fn get_override_range_master(&self)->Option<(f64,f64)>{
        if self.settings.display_mode.is_temporal(){
            self.get_x_scale()
        }
        else{
            None
        }
    }

    pub fn set_override_range(&mut self, range: Option<(f64,f64)>){
        self.override_range = range;
    }

    pub fn update_settings(&mut self, settings:&super::form::PlotSettings){
        self.settings = settings.clone();
        self.cache.clear();
    }

    pub fn get_late_update_message(&self, msg:&SubplotterMessage)->Option<PadamoAppMessage>{
        if let SubplotterMessage::PlotXClicked(plot_x) = msg{
            let unixtime = if self.settings.display_mode.is_temporal(){
                *plot_x
            }
            else{
                let signal = if let Some(v) = &self.displaying_signal {v} else {return None;};
                let frame_number = plot_x.round() as usize;
                if frame_number<signal.start_frame{return None;}
                let i = frame_number - signal.start_frame;
                if i>=signal.time.len(){return None;}
                signal.time[i]
            };
            Some(PadamoAppMessage::ViewerMessage(ViewerMessage::TimeLine(CrossProgressMessage::SetViewPositionUnixTime(unixtime))))
        }
        else{
            None
        }
    }
}

impl Chart<SubplotterMessage> for Subplotter{
    type State = ();

    #[inline]
    fn draw<R: Renderer, F: Fn(&mut iced::widget::canvas::Frame)>(&self, renderer: &R, size: iced::Size, f: F) -> iced::widget::canvas::Geometry {
        renderer.draw_cache(&self.cache, size, f)
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: iced::widget::canvas::Event,
        bounds: iced::Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> (iced::event::Status, Option<SubplotterMessage>) {

        if let iced::mouse::Cursor::Available(point) = cursor {
            match event {
                iced::widget::canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) if bounds.contains(point) => {
                    let p_origin = bounds.position();
                    let p = point - p_origin;
                    if let Some(spec) = self.plot_spec.borrow().as_ref(){
                        if let Some(inpoint) = spec.reverse_translate((p.x as i32,p.y as i32)){
                            //println!("Clicked X = {}", inpoint.0);
                            return (iced::event::Status::Captured,Some(SubplotterMessage::PlotXClicked(inpoint.0)));
                        }
                    }
                }
                _=>(),
            }
        }
        (iced::event::Status::Ignored,None)
    }
    #[inline]
    fn draw_chart<DB: DrawingBackend>(&self, state: &Self::State, root: DrawingArea<DB, Shift>) {
        root.fill(&plotters::prelude::WHITE).unwrap();
        let builder = ChartBuilder::on(&root);
        self.build_chart(state, builder);
    }

    fn build_chart<DB: plotters_iced::DrawingBackend>(&self, state: &Self::State, mut builder: plotters::prelude::ChartBuilder<DB>) {
        use plotters::prelude::*;
        // let y_range = self.y_range.unwrap_or((0.0,1.0));
        let x_range = self.get_x_scale().unwrap();
        let mut chart = builder
            .x_label_area_size(20)
            .y_label_area_size(50)
            .margin(20)
            .build_cartesian_2d(x_range.0..x_range.1, self.settings.min_value..self.settings.max_value)
            .expect("failed to build chart");

        let signal = if let Some(x) = &self.displaying_signal {x} else {return};

        chart.configure_mesh()
            .x_label_formatter(&|x| self.settings.display_mode.format_x(*x, signal.time[0]))
            .draw()
            .expect("failed to draw chart mesh");

        let pixels = if let Some(pix) = &self.pixels {pix} else {return;};
        let detector = if let Some(det) = &self.last_detector {det} else {return;};

        let mut lc: Vec<f64> = vec![0.0; signal.time.len()];
        let mut pixels_shown:usize = 0;

        for pixel_id in pixels.enumerate(){
            if pixels[&pixel_id]{
                let mut indexer = vec![0];
                indexer.extend(pixel_id.clone());

                let color = detector.cells.find_color(&pixel_id).unwrap_or((0.0,0.0,0.0));
                let r = (color.0*256.0) as u8;
                let g = (color.1*256.0) as u8;
                let b = (color.2*256.0) as u8;
                let col = RGBColor(r,g,b);
                pixels_shown += 1;

                let series = LineSeries::new(
                    (0..signal.time.len()).map(|i|{
                        indexer[0] = i;

                        if let LCSelection::Selected = self.settings.lc_display.selection{
                            lc[i] += signal.signals[&indexer];
                        }

                        if self.settings.display_mode.is_temporal(){
                            (signal.time[i], signal.signals[&indexer])
                        }
                        else{
                            ((i+signal.start_frame) as f64, signal.signals[&indexer])
                        }
                    }),
                    &col
                );
                if !self.settings.lc_display.only{
                    chart.draw_series(series).unwrap();
                }
            }

            if let LCSelection::All = self.settings.lc_display.selection{
                for i in 0..signal.time.len(){
                    let mut indexer = vec![i];
                    indexer.extend(pixel_id.clone());
                    lc[i] += signal.signals[&indexer];
                }
                pixels_shown += 1;
            }


        }

        if let LCSelection::All | LCSelection::Selected = self.settings.lc_display.selection{
            chart.draw_series(LineSeries::new(
                (0..signal.time.len()).map(|i|{
                    let v = if self.settings.lc_display.mean{
                        if pixels_shown>0{
                            lc[i]/(pixels_shown as f64)
                        }
                        else{
                            0.0
                        }
                    }
                    else{
                        lc[i]
                    };

                    if self.settings.display_mode.is_temporal(){
                        (signal.time[i], v)
                    }
                    else{
                        ((i+signal.start_frame) as f64, v)
                    }
                }),
                &BLACK
            )).unwrap();
        }

        if let Some(x) = self.pointer{
            if self.settings.display_mode.is_temporal() && x>=signal.start_frame{
                let frame = x-signal.start_frame;
                if let Some(t) = signal.time.get(frame){
                    let pointermap = vec![(*t,self.settings.min_value),(*t, self.settings.max_value)];
                    chart.draw_series(LineSeries::new(pointermap.iter().map(|a| *a), &RED)).unwrap();
                }
            }
            else{
                let pointermap = vec![(x as f64,self.settings.min_value),(x as f64, self.settings.max_value)];
                chart.draw_series(LineSeries::new(pointermap.iter().map(|a| *a), &RED)).unwrap();
            };
        }

        *self.plot_spec.borrow_mut() = Some(chart.as_coord_spec().clone());
    }
}
