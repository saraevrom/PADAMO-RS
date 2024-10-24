use std::{marker::PhantomData, cell::RefCell};

use iced::Element;
use serde::{Serialize,Deserialize};
use plotters::{coord::{types::RangedCoordf64, ReverseCoordTranslate}, element::BackendCoordOnly, prelude::*};
use plotters_iced::{Chart, ChartWidget};
use plotters_layout::{centering_ranges, ChartLayout};
use padamo_api::lazy_array_operations::ndim_array::ArrayND;

use chrono::prelude::*;


pub mod polygon;
pub mod parser;
pub mod colors;
pub mod selector_chart;
pub mod transformer;

pub use selector_chart::DetectorChartMap;

//use polygon::StableColorMatrix;

const COLORBAR_SEGMENTS:usize = 256;

const COLORBAR_WIDTH:f64 = 1.0;

#[derive(Clone, Copy)]
pub struct Margins{
    pub top:u32,
    pub bottom:u32,
    pub left:u32,
    pub right:u32
}

#[derive(Clone)]
pub struct Detector<Message>{
    pub cells:polygon::DetectorContent,
    pub alive_pixels:ArrayND<bool>,
    spec:RefCell<Option<Cartesian2d<RangedCoordf64, RangedCoordf64>>>,
    _marker: PhantomData<Message>,
}

impl<Message> Detector<Message>{
    pub fn new(compat_shape:Vec<usize>,name:String)->Self{
        let capacity = compat_shape.iter().fold(1, |a,b| a*b);
        let cells = polygon::DetectorContent::new(compat_shape, name);
        Self::from_cells(cells)
    }

    pub fn alive_pixels_mask(&self)->ArrayND<f64>{
        let shape = self.alive_pixels.shape.clone();
        let flat_data = self.alive_pixels.flat_data.iter().map(|x| if *x {1.0} else {0.0}).collect::<Vec<f64>>().into();
        ArrayND { flat_data, shape }
    }

    pub fn toggle_pixel(&mut self, index:&Vec<usize>){
        self.alive_pixels[index] = !self.alive_pixels[index];
    }

    pub fn from_cells(cells:polygon::DetectorContent)->Self{
        let alive_pixels = ArrayND::new(cells.compat_shape.clone(), true) ;
        Self { cells, _marker:PhantomData, spec: RefCell::new(None),alive_pixels}
    }


    pub fn shape(&self)->&Vec<usize>{
        &self.cells.compat_shape
    }

    pub fn default_vtl()->Self{
        Self::from_cells(polygon::DetectorContent::default_vtl())
    }


//     pub fn plot_chart_aux<DB: DrawingBackend>(&self,root: &DrawingArea<DB, plotters::coord::Shift>, pixels:&[Vec<usize>], vis:&[bool]) {
//
//     }


    pub fn build_chart_aux<DB: DrawingBackend>(&self,root: &DrawingArea<DB, plotters::coord::Shift>, pixels:&[Vec<usize>], vis:&[bool], margins:Margins) {
        let ((min_x, min_y), (max_x, max_y)) = self.cells.size();
        let min_range = (min_x..max_x, min_y..max_y);

        let main_part = root;

        let main_builder = ChartLayout::new()
            //.caption("", ("sans-serif", 20))
            //.unwrap()
            .margin(2)
            //.margin_top(10)
            .margin_top(margins.top)
            .margin_bottom(margins.bottom)
            .margin_left(margins.left)
            .margin_right(margins.right)
            .x_label_area_size(0)
            .y_label_area_size(0)
            .bind(&main_part)
            .unwrap();


        let (width, height) = main_builder.estimate_plot_area_size();
        let (x_range, y_range) = centering_ranges(&min_range, &(width as f64, height as f64));

        let mut chart = main_builder.build_cartesian_2d(x_range, y_range).unwrap();
        chart.configure_mesh()
            .disable_mesh()
            .draw()
            .unwrap();

        let mut infills:Vec<Polygon<(f64,f64)>> = Vec::new();
        let mut outlines:Vec<PathElement<(f64,f64)>> = Vec::new();
        for (infill,outline) in self.cells.pixels_colors(pixels, vis){
            infills.push(infill);
            outlines.push(outline);
        }


        chart.draw_series::<BackendCoordOnly,Polygon<(f64,f64)>,_,_>(
              infills.iter()
        ).unwrap();

        chart.draw_series::<BackendCoordOnly,PathElement<(f64,f64)>,_,_>(
              outlines.iter()
        ).unwrap();
        *self.spec.borrow_mut() = Some(chart.as_coord_spec().clone());

    }

    pub fn build_chart_generic<DB: DrawingBackend>(&self,root: &DrawingArea<DB, plotters::coord::Shift>, pixels:&Option<(& ArrayND<f64>,f64)>, scale:Scaling, state:&Option<((f64,f64),(i32,i32))>) {
        root.fill(&plotters::prelude::WHITE).unwrap();
        let ((min_x, min_y), (max_x, max_y)) = self.cells.size();
        let min_range = (min_x..max_x, min_y..max_y);

        let title:String = if let Some((_,ut)) = pixels{
            let ut_s = *ut as i64;
            let ut_ns = ((ut-ut_s as f64)*1e9) as u32;
            //println!("Datetime from {}=>{}; {}",ut,ut_s,ut_ns);
            let naive = NaiveDateTime::from_timestamp_opt(ut_s,ut_ns).unwrap();

            // Create a normal DateTime from the NaiveDateTime
            let datetime: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);

            // Format the datetime how you want
            let newdate = datetime.format("%Y-%m-%d %H:%M:%S.%f");
            newdate.to_string()
        }
        else{
            "View".to_string()
        };

        let pixel_width = root.dim_in_pixel().0;

        let (main_part, colorbar_part) = root.split_horizontally(pixel_width-80);

        let main_builder = ChartLayout::new()
            .caption(title, ("sans-serif", 20))
            .unwrap()
            .margin(4)
            //.margin_top(10)
            .x_label_area_size(28)
            .y_label_area_size(40)
            .bind(&main_part)
            .unwrap();

        let (width, height) = main_builder.estimate_plot_area_size();
        let (x_range, y_range) = centering_ranges(&min_range, &(width as f64, height as f64));



        let mut chart = main_builder.build_cartesian_2d(x_range, y_range).unwrap();
        chart.configure_mesh()
            .disable_mesh()
            .draw()
            .unwrap();
        let rects = self.cells.pixels_values(&self.alive_pixels,pixels,scale);
        chart.draw_series(
            rects
        ).unwrap();

        *self.spec.borrow_mut() = Some(chart.as_coord_spec().clone());

        Some(chart.as_coord_spec().clone());

        let (min,max) =  if let Some(pix) = pixels{
            scale.get_bounds(pix.0,&self.alive_pixels)
        }
        else{
            (0.0,1.0)
        };

        if let Some((pos,unmapped)) = state{
            if let Some(index) = self.cells.position_index(*pos){
                //println!("{:?}",index);

                let mut unmapped_pos = *unmapped;
                unmapped_pos.1 -= 20;
                let txt = if let Some((buf,t)) = pixels {
                    if let Some(val) = buf.try_get(index){
                        format!("{:?} {:.3}",index,val)
                    }
                    else{
                        format!("{:?} MAPPING INVALID",index)
                    }

                } else{format!("{:?}",index)};
                root.draw_text(&txt, &(("sans-serif", 15).into()), unmapped_pos).unwrap();
            }
        }

        let cmap_builder = ChartLayout::new()
            .y_label_area_size(50)
            .x_label_area_size(28)
            //.margin(4)
            .margin_top(10)
            .margin_right(0)
            .bind(&colorbar_part)
            .unwrap();

        let mut cmap_chart = cmap_builder.build_cartesian_2d(0.0..COLORBAR_WIDTH, min..max).unwrap();
        cmap_chart.configure_mesh()
            .set_all_tick_mark_size(5)
            .disable_x_axis()
            .disable_mesh()
            .label_style("sans-serif".into_font())
            .draw()
            .unwrap();
        let color_rects = ColorbarRects::new(min, max);
        cmap_chart.draw_series(color_rects).unwrap();


    }

    pub fn view<'a,F1:'static+Fn(Vec<usize>)->Message,F2:'static+Fn(Vec<usize>)->Message>(&'a self, source:Option<(&'a ArrayND<f64>,f64)>, scale:Scaling, lclick_event:Option<F1>, rclick_event:Option<F2>)->iced::Element<'a,Message>{
        ChartWidget::new(DetectorChart::new(self,source,scale, lclick_event, rclick_event)).into()
    }

    pub fn view_map<'a,F1:'static+Fn(Vec<usize>)->Message,F2:'static+Fn(Vec<usize>)->Message>(&'a self, pixels:&'a Vec<Vec<usize>>, pixels_show:&'a Vec<bool>, lclick_event:Option<F1>, rclick_event:Option<F2>)->iced::Element<'a,Message>{
        ChartWidget::new(DetectorChartMap::new(self,pixels,pixels_show,lclick_event,rclick_event)).into()
    }
}


pub struct DetectorChart<'a,Msg,F1,F2>
where
    F1:'static + Fn(Vec<usize>)->Msg,
    F2:'static + Fn(Vec<usize>)->Msg,
{
    detector:&'a Detector<Msg>,
    source:Option<(&'a ArrayND<f64>,f64)>,
    scale:Scaling,
    lclick_event:Option<F1>,
    rclick_event:Option<F2>,
}

impl<'a,Msg,F1,F2> DetectorChart<'a,Msg,F1,F2>
where
    F1:'static + Fn(Vec<usize>)->Msg,
    F2:'static + Fn(Vec<usize>)->Msg,
{
    pub fn new(detector:&'a Detector<Msg>, source:Option<(&'a ArrayND<f64>,f64)>, scale:Scaling, lclick_event:Option<F1>, rclick_event:Option<F2>)->Self{
        Self { detector ,source, scale, lclick_event, rclick_event}
    }
}


impl<'a,Msg,F1,F2> Chart<Msg> for DetectorChart<'a,Msg,F1,F2>
where
    F1:'static + Fn(Vec<usize>)->Msg,
    F2:'static + Fn(Vec<usize>)->Msg,
{
    type State = Option<((f64,f64),(i32,i32))>;


    fn build_chart<DB: DrawingBackend>(&self, state: &Self::State, builder: ChartBuilder<DB>) {}

    fn draw_chart<DB: DrawingBackend>(&self, state: &Self::State, root: DrawingArea<DB, plotters::coord::Shift>) {
        self.detector.build_chart_generic(&root, &self.source,self.scale,state);
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: iced::widget::canvas::Event,
        bounds: iced::Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> (iced::event::Status, Option<Msg>) {
        if let iced::mouse::Cursor::Available(point) = cursor {
            match event {
                iced::widget::canvas::Event::Mouse(evt) if bounds.contains(point) => {
                    let p_origin = bounds.position();
                    let p = point - p_origin;
                    if let Some(spec) = self.detector.spec.borrow().as_ref(){
                        if let Some(inpoint) = spec.reverse_translate((p.x as i32,p.y as i32)){
                            //println!("{:?}",inpoint);
                            *state = Some((inpoint,(p.x as i32,p.y as i32)));
                            let mut msg = None;
                            if let iced::mouse::Event::ButtonPressed(iced::mouse::Button::Right) = evt{
                                if let Some(caller) = &self.rclick_event{
                                    if let Some(index) = self.detector.cells.position_index(inpoint){
                                        msg = Some(caller(index.clone()));
                                    }
                                }
                            }
                            else if let iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) = evt{
                                if let Some(caller) = &self.lclick_event{
                                    if let Some(index) = self.detector.cells.position_index(inpoint){
                                        msg = Some(caller(index.clone()));
                                    }
                                }
                            }

                            return (
                                iced::event::Status::Captured,
                                msg,
                            );
                        }
                    }

                }
                _ => {}
            }
        }
        *state = None;
        (iced::event::Status::Ignored, None)
    }
}




#[derive(Clone,Copy,Debug)]
pub enum Scaling{
    Autoscale,
    Fixed(f64,f64)
}

impl Scaling{
    pub fn get_bounds(&self, frame:&ArrayND<f64>, alive_pixels:&ArrayND<bool>)->(f64,f64){
        let first = frame.flat_data[0];
        match self{
            Self::Autoscale=>{
                let (min,max) = frame.flat_data.iter()
                    .enumerate()
                    .filter(|x| alive_pixels.flat_data[x.0])
                    .map(|x| x.1)
                    .fold((first,first), |a,b| (a.0.min(*b),a.1.max(*b)));
                if max<=min{
                    (min-0.1,max+0.1)
                }
                else{
                    (min,max)
                }
            }
            Self::Fixed(min, max)=>{
                if max<=min{
                    (*min,*min+0.001)
                }
                else{
                    (*min,*max)
                }
            }
        }
    }
}


struct ColorbarRects{
    pub min:f64,
    pub max:f64,
    index:usize,
}

impl ColorbarRects{
    pub fn new(min:f64, max:f64)->Self{
        Self { min, max, index: 0 }
    }
}

impl Iterator for ColorbarRects{
    type Item = Rectangle<(f64,f64)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index>=COLORBAR_SEGMENTS{
            return None;
        }

        let step = (self.max - self.min)/(COLORBAR_SEGMENTS as f64);
        let start_y = step * (self.index as f64)+self.min;
        let end_y = start_y+step;
        let mid_y = start_y+step*0.5;

        let s_coords = (0.0,start_y);
        let e_coords = (COLORBAR_WIDTH,end_y);


        let color = plotters::style::colors::colormaps::ViridisRGB::get_color_normalized(mid_y,self.min, self.max).filled();

        self.index += 1;
        Some(Rectangle::new(
            [s_coords, e_coords],
            color
        ))
    }

}
