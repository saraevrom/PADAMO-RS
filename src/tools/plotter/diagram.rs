use super::{messages::PlotterMessage, TimeAxisFormat};
//use super::colors::get_color;
use padamo_detectors::{colors::get_color_indexed, Margins};
use iced::{
    widget::canvas::{Cache, Frame, Geometry},
    Size,
};

use plotters_iced::{
    Chart, ChartWidget, Renderer,
};

use plotters::{coord::{ReverseCoordTranslate, Shift}, style::full_palette::ORANGE};
use plotters::prelude::*;


//pub struct

pub struct PlotterChart<'a>{
    plotter_data: &'a super::Plotter,
    //cache:Cache,
}

impl<'a> PlotterChart<'a>{
    pub fn new(plotter:&'a super::Plotter)->Self{
        //println!("New DEBUG");
        Self{plotter_data: plotter}
    }

    pub fn view(self)->iced::Element<'a,PlotterMessage> {
        //println!("View DEBUG");
        ChartWidget::new(self).into()
    }




}


impl<'a> Chart<PlotterMessage> for PlotterChart<'a> {
    type State = ();

    #[inline]
    fn draw<R: Renderer, F: Fn(&mut Frame)>(
        &self,
        renderer: &R,
        bounds: Size,
        draw_fn: F,
    ) -> Geometry {
        renderer.draw_cache(&self.plotter_data.cache, bounds, draw_fn)
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: iced::widget::canvas::Event,
        bounds: iced::Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> (iced::event::Status, Option<PlotterMessage>) {

        if let iced::mouse::Cursor::Available(point) = cursor {
            match event {
                iced::widget::canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) if bounds.contains(point) => {
                    let p_origin = bounds.position();
                    let p = point - p_origin;
                    if let Some(spec) = self.plotter_data.plot_spec.borrow().as_ref(){
                        if let Some(inpoint) = spec.reverse_translate((p.x as i32,p.y as i32)){
                            //println!("Clicked X = {}", inpoint.0);
                            return (iced::event::Status::Captured,Some(PlotterMessage::PlotXClicked(inpoint.0)));
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
        if self.plotter_data.channelmap_show{
            let fullsize = root.dim_in_pixel();
            let (low,high) = self.plotter_data.detector.cells.size();
            let w = (high.0-low.0) as f32;
            let h = (high.1-low.1) as f32;

            let width:u32 = 100;
            let height:u32 =((width as f32) *(h/w)) as u32;

            let margins = Margins{left:120, bottom:fullsize.1-20-height, right:fullsize.0-120-width, top:20};
            self.plotter_data.detector.build_chart_aux(&root, &self.plotter_data.pixels, &self.plotter_data.pixels_show, margins);

        }
    }

    fn build_chart<DB:DrawingBackend>(&self, state: &Self::State,  mut builder:ChartBuilder<DB>) {
        //build your chart here, please refer to plotters for more details
        if let super::DataState::Loaded(data) = &self.plotter_data.data{
            let tim = &data.time;
            let signal = &data.signal;
            let lc_total = &data.lc;
            let total_pix_count = &data.pixel_count;
            let (ymin,ymax) = self.plotter_data.last_y_limits_live;
            let (xmin,xmax) = self.plotter_data.last_x_limits_live;
            let mut chart = builder
                .x_label_area_size(50)
                .y_label_area_size(100)
                .margin(10)

                .build_cartesian_2d(xmin..xmax, ymin..ymax)
                .unwrap();

            chart.configure_mesh()
                .x_label_formatter(&(|x| self.plotter_data.axis_formatter.format(*x, tim[0])))
                .x_labels(5)
                .y_labels(10)
                .draw().unwrap();

            let mut selected_lc:Vec<f64> = Vec::with_capacity(tim.len());
            selected_lc.resize(tim.len(), 0.0);
            let mut pixels_count:f64 = 0.0;

            for i in 0..self.plotter_data.pixels_show.len(){
                if !self.plotter_data.pixels_show[i]{
                    continue;
                }
                pixels_count+=1.0;
                let index = &self.plotter_data.pixels[i];

                //println!("DRAW {:?}", index);
                let mut view_id = Vec::with_capacity(index.len()+1);

                view_id.push(0);
                view_id.extend(index);

                if !signal.index_compatible(&view_id){
                    continue;
                }

                let color:(f32,f32,f32) = get_color_indexed(&index);
                let r = (color.0*256.0) as u8;
                let g = (color.1*256.0) as u8;
                let b = (color.2*256.0) as u8;
                //println!("RGB {} {} {}",r,g,b);
                let col = RGBColor(r,g,b);
                if signal.shape.len()==index.len()+1{
                    if self.plotter_data.lc_only{
                        for i in 0..tim.len(){
                            view_id[0] = i;
                            selected_lc[i] += signal[&view_id];
                        }
                    }
                    else{
                        let drawn_series = chart.draw_series(LineSeries::new((0..tim.len()).map(|i| {
                            view_id[0] = i;
                            selected_lc[i] += signal[&view_id];
                            let x = if let TimeAxisFormat::GTU = self.plotter_data.axis_formatter{(i+data.start) as f64} else {tim[i]};
                            (x, signal[&view_id])
                        }), &col));
                        if let Ok(series) = drawn_series{
                            series.label(format!("{:?}", index));
                        }
                        else{
                            println!("Plotter: draw error");
                        }
                    }

                }

                else{
                    println!("Plotter: NO SHAPE MATCH");
                }

            }

            let index = self.plotter_data.view_index;
            let start = self.plotter_data.view_pivot;
            if self.plotter_data.display_pointer && index>=start{
                let pos_index = index-start;
                if pos_index<tim.len(){
                    let ptr_x = if let TimeAxisFormat::GTU = self.plotter_data.axis_formatter {index as f64} else {tim[index-start]};
                    let ptr_data = vec![(ptr_x,ymin),(ptr_x,ymax)];
                    let drawn_ptr = chart.draw_series(LineSeries::new((0..2).map(|i| ptr_data[i]), &RED));
                    if let Ok(series) = drawn_ptr{
                        series.label("Current position");
                    }
                }

            }

            match self.plotter_data.lc_mode {
                super::LCMode::Off => (),
                super::LCMode::All => {

                    let divider = if self.plotter_data.lc_mean {*total_pix_count as f64} else {1.0};
                    let drawn_series = chart.draw_series(LineSeries::new((0..tim.len()).map(|i| {
                        //let x = if let TimeAxisFormat::GTU = self
                        let x = if let TimeAxisFormat::GTU = self.plotter_data.axis_formatter{(i+data.start) as f64} else {tim[i]};
                        (x, lc_total[i]/divider)
                    }), &BLACK));
                    if let Ok(series) = drawn_series{
                        series.label("LC");
                    }
                },
                super::LCMode::Selected => {

                    let divider = if self.plotter_data.lc_mean {pixels_count} else {1.0};

                    let drawn_series = chart.draw_series(LineSeries::new((0..tim.len()).map(|i| {
                        let x = if let TimeAxisFormat::GTU = self.plotter_data.axis_formatter{(i+data.start) as f64} else {tim[i]};
                        (x, selected_lc[i]/divider)
                    }), &BLACK));
                    if let Ok(series) = drawn_series{
                        series.label("LC");
                    }
                },
            }

            if let TimeAxisFormat::GTU = self.plotter_data.axis_formatter {
                // Gray line shenanigans
                for i in 0..tim.len()-1{
                    let x = (i+data.start) as f64;
                    let ptr_data = vec![(x,ymin),(x,ymax)];
                    if tim[i+1]<tim[i]{
                        chart.draw_series(LineSeries::new((0..2).map(|i| ptr_data[i]), &RGBColor(157,0,0))).unwrap();
                    }
                    else if tim[i+1]-tim[i] > self.plotter_data.step_threshold.parsed_value*data.time_step{
                        chart.draw_series(LineSeries::new((0..2).map(|i| ptr_data[i]), &RGBColor(127,127,127))).unwrap();
                    }
                }
            }
            else{
                // White rectangles shenanigans
            }

            *self.plotter_data.plot_spec.borrow_mut() = Some(chart.as_coord_spec().clone());
        }
    }
}
