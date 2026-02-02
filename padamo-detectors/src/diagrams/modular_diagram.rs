use std::cell::RefCell;

use crate::polygon::DetectorContent;
use super::traits::ColorValueSource;
use crate::transformer::Transform;
use plotters::{coord::{ReverseCoordTranslate, types::RangedCoordf64}, prelude::*};
use plotters_layout::{ChartLayout, centering_ranges};

struct TransformedMesh{
    mesh: crate::Mesh,
    transformation_matrix:nalgebra::Matrix4<f64>,
    style:plotters::prelude::ShapeStyle,
}

#[derive(Default)]
pub struct PadamoDetectorDiagramState{
    pub click_state: super::auxiliary::ClickTracker,
    pub pos: (f64,f64),
    pub unmapped: (i32, i32),
    pub spec: RefCell<Option<Cartesian2d<RangedCoordf64, RangedCoordf64>>>,
}

pub struct PadamoDetectorDiagram<'a, Msg>
where
    // F1:'static + Fn(Vec<usize>)->Msg,
    // F2:'static + Fn(Vec<usize>)->Msg,
{
    detector: &'a DetectorContent,
    color_source: Box<dyn ColorValueSource>,
    title:Option<String>,
    lmb_action:Option<Box<dyn 'static + Fn(Vec<usize>)->Msg>>,
    rmb_action:Option<Box<dyn 'static + Fn(Vec<usize>)->Msg>>,
    multiselect_action:Option<Box<dyn 'static + Fn(Vec<&'a [usize]>, iced::mouse::Button)->Msg>>,
    transform: Transform,
    mesh:Option<TransformedMesh>,
}

impl<'a, Msg> PadamoDetectorDiagram<'a, Msg>
{
    pub fn new<C:ColorValueSource+'static>(detector: &'a DetectorContent, color_source: C) -> Self {
        Self {
            detector,
            color_source: Box::new(color_source),
            lmb_action: None,
            rmb_action: None,
            multiselect_action: None,
            title:None,
            transform: Transform::new(1.0, 0.0, 0.0),
            mesh:None,
        }
    }

    pub fn on_left_click<F:'static + Fn(Vec<usize>)->Msg>(mut self, action:F) -> Self{
        self.lmb_action = Some(Box::new(action));
        self
    }

    pub fn on_right_click<F:'static + Fn(Vec<usize>)->Msg>(mut self, action:F) -> Self{
        self.rmb_action = Some(Box::new(action));
        self
    }

    fn get_click_event(&self, btn:iced::mouse::Button)->Option<&Box<dyn 'static + Fn(Vec<usize>)->Msg>>{
        match btn{
            iced::mouse::Button::Left => self.lmb_action.as_ref(),
            iced::mouse::Button::Right => self.rmb_action.as_ref(),
            _=>None,
        }
    }

    pub fn on_multiselect<F:'static + Fn(Vec<&'a[usize]>, iced::mouse::Button)->Msg>(mut self, action:F) -> Self{
        self.multiselect_action = Some(Box::new(action));
        self
    }

    pub fn transformed(mut self, transform:Transform) -> Self{
        self.transform = transform;
        self
    }

    pub fn with_title(mut self, title:String) -> Self{
        self.title = Some(title);
        self
    }

    pub fn with_title_unixtime(self, ut:f64) -> Self{
        let ut_s = ut as i64;
        let ut_ns = ((ut-ut_s as f64)*1e9) as u32;

        let title = if let Some(datetime) = chrono::DateTime::from_timestamp(ut_s,ut_ns){
            let newdate = datetime.format("%Y-%m-%d %H:%M:%S.%f");
            newdate.to_string()
        }
        else{
            format!("Invalid unixtime: {}", ut)
        };

        self.with_title(title)
    }

    pub fn draw_main<DB: DrawingBackend>(&self, root: &DrawingArea<DB, plotters::coord::Shift>,state:Option<&PadamoDetectorDiagramState>){
        let ((min_x, min_y), (max_x, max_y)) = self.detector.size();
        let min_range = (min_x..max_x, min_y..max_y);

        let mut layout = ChartLayout::new();
        if let Some(title) = &self.title{
            layout.caption(title, ("sans-serif", 20)).unwrap();
        }
        let main_builder = layout
        .margin(4)
        //.margin_top(10)
        .x_label_area_size(28)
        .y_label_area_size(40)
        .bind(root).unwrap();

        let (width, height) = main_builder.estimate_plot_area_size();
        let (x_range, y_range) = centering_ranges(&min_range, &(width as f64, height as f64));
        let x_range = self.transform.transform_x_range(x_range);
        let y_range = self.transform.transform_y_range(y_range);

        // let (min,max) =  if let Some(pix) = pixels{
        //     scale.get_bounds(pix.0,&detector.alive_pixels)
        // }
        // else{
        //     (0.0,1.0)
        // };

        let mut chart = main_builder.build_cartesian_2d(x_range, y_range).unwrap();
        chart.configure_mesh()
        .disable_mesh()
        .draw().unwrap();

        let rects = super::PolyIterator::new(self.color_source.as_ref(), self.detector);
        chart.draw_series(rects).unwrap();

        if self.color_source.has_outline(){
            let polys = self.detector.pixels_outlines();
            chart.draw_series(polys).unwrap();
        }

        if let Some(m) = &self.mesh{
            m.mesh.draw(m.transformation_matrix, m.style, &mut chart);
        }

        //TODO: display pixel

        if let Some(s) = state{
            *s.spec.borrow_mut() = Some(chart.as_coord_spec().clone());
            super::auxiliary::display_pixel_id(self.detector, root, self.color_source.as_ref(), s);
        }

    }

    pub fn build_chart_generic<DB: DrawingBackend>(&self,
                                                           root: &DrawingArea<DB, plotters::coord::Shift>,
                                                           state:Option<&PadamoDetectorDiagramState>){
        root.fill(&WHITE).unwrap();


        let pixel_width = root.dim_in_pixel().0;

        if let Some((min,max)) = self.color_source.get_norm(){
            let (main_part, colorbar_part) = root.split_horizontally(pixel_width-80);
            self.draw_main(&main_part, state);

            let cmap_builder = ChartLayout::new()
            .y_label_area_size(50)
            .x_label_area_size(28)
            //.margin(4)
            .margin_top(10)
            .margin_right(0)
            .bind(&colorbar_part).unwrap();
            //
            let mut cmap_chart = cmap_builder.build_cartesian_2d(0.0..super::colorbar::COLORBAR_WIDTH, min..max).unwrap();
            cmap_chart.configure_mesh()
                .set_all_tick_mark_size(5)
                .disable_x_axis()
                .disable_mesh()
                .label_style("sans-serif".into_font())
                .draw()
                .unwrap();
            let color_rects = super::colorbar::ColorbarRects::new(min, max);
            cmap_chart.draw_series(color_rects).unwrap();
        }
        else{
            self.draw_main(root, state);
        }

    }
}

impl<'a,Message> plotters_iced::Chart<Message> for PadamoDetectorDiagram<'a,Message>{
    type State = PadamoDetectorDiagramState;

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, _builder: ChartBuilder<DB>) {}

    fn draw_chart<DB: DrawingBackend>(&self, state: &Self::State, root: DrawingArea<DB, plotters::coord::Shift>) {
        self.build_chart_generic(&root, Some(state));
        //self.detector_plotter.build_chart_generic(&self.detector, &root, &self.source,self.scale,self.transform,state, self.mesh_info);
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: iced::Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> (iced::event::Status, Option<Message>) {
        if let iced::mouse::Cursor::Available(point) = cursor {
            if bounds.contains(point){
                if let iced::widget::canvas::Event::Mouse(evt) = event{
                    let p_origin = bounds.position();
                    let p = point - p_origin;

                    if let Some(spec) = state.spec.borrow().as_ref(){
                        if let Some(inpoint) = spec.reverse_translate((p.x as i32,p.y as i32)){
                            state.pos = inpoint;
                            state.unmapped = (p.x as i32,p.y as i32);

                            if let Some(index) = self.detector.position_index(inpoint){
                                match evt{
                                    iced::mouse::Event::ButtonPressed(btn)=>{
                                        if let iced::mouse::Button::Left | iced::mouse::Button::Right = btn {
                                            state.click_state.click(*btn, inpoint);
                                            if let Some(caller) = self.get_click_event(*btn){
                                                let msg = Some(caller(index.into()));
                                                return (iced::event::Status::Captured, msg);
                                            }
                                        }
                                    }
                                    iced::mouse::Event::ButtonReleased(btn)=>{
                                        if let iced::mouse::Button::Left | iced::mouse::Button::Right = btn {
                                            if let Some(pos1) = state.click_state.release(*btn){
                                                if let Some(caller) = &self.multiselect_action{
                                                    let (left, top) = pos1;
                                                    let (right, bottom) = inpoint;
                                                    let indices = self.detector.select_indices_in_rectangle(left, right, top, bottom);
                                                    let msg = Some(caller(indices, *btn));
                                                    return (iced::event::Status::Captured, msg);
                                                }
                                            }
                                        }
                                    }
                                    iced::mouse::Event::CursorLeft=>{
                                        state.click_state.reset();
                                    }
                                    _=>(),
                                }
                            }

                        }
                    }
                }
            }
            // match event {
            //     iced::widget::canvas::Event::Mouse(evt) if bounds.contains(point) => {
            //         let p_origin = bounds.position();
            //         let p = point - p_origin;
            //         if let Some(spec) = self.detector_plotter.spec.borrow().as_ref(){
            //             if let Some(inpoint) = spec.reverse_translate((p.x as i32,p.y as i32)){
            //                 //println!("{:?}",inpoint);
            //                 *state = Some((inpoint,(p.x as i32,p.y as i32)));
            //                 let mut msg = None;
            //                 if let iced::mouse::Event::ButtonPressed(iced::mouse::Button::Right) = evt{
            //                     if let Some(caller) = &self.rclick_event{
            //                         if let Some(index) = self.detector.cells.position_index(inpoint){
            //                             msg = Some(caller(index.into()));
            //                         }
            //                     }
            //                 }
            //                 else if let iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) = evt{
            //                     if let Some(caller) = &self.lclick_event{
            //                         if let Some(index) = self.detector.cells.position_index(inpoint){
            //                             msg = Some(caller(index.into()));
            //                         }
            //                     }
            //                 }
            //
            //                 return (
            //                     iced::event::Status::Captured,
            //                     msg,
            //                 );
            //             }
            //         }
            //
            //     }
            //     _ => {}
            // }
        }
        // *state = None;
        (iced::event::Status::Ignored, None)
    }
}
