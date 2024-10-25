
use plotters::{coord::ReverseCoordTranslate, prelude::*};
use plotters_iced::Chart;


use crate::{Detector, Margins};

pub struct DetectorChartMap<'a,Msg,F1,F2>
where
    F1:'static + Fn(Vec<usize>)->Msg,
    F2:'static + Fn(Vec<usize>)->Msg,
{
    detector:&'a Detector<Msg>,
    //source:Option<(&'a ArrayND<f64>,f64)>,
    //scale:Scaling,
    pixels:&'a Vec<Vec<usize>>,
    pixels_show:&'a Vec<bool>,
    lclick_event:Option<F1>,
    rclick_event:Option<F2>,
}

impl<'a,Msg,F1,F2> DetectorChartMap<'a,Msg,F1,F2>
where
    F1:'static + Fn(Vec<usize>)->Msg,
    F2:'static + Fn(Vec<usize>)->Msg,
{
    pub fn new(detector:&'a Detector<Msg>, pixels:&'a Vec<Vec<usize>>, pixels_show:&'a Vec<bool>, lclick_event:Option<F1>, rclick_event:Option<F2>)->Self{
        Self { detector ,pixels,pixels_show, lclick_event, rclick_event}
    }
}


impl<'a,Msg,F1,F2> Chart<Msg> for DetectorChartMap<'a,Msg,F1,F2>
where
    F1:'static + Fn(Vec<usize>)->Msg,
    F2:'static + Fn(Vec<usize>)->Msg,
{
    type State = Option<((f64,f64),(i32,i32))>;


    fn build_chart<DB: DrawingBackend>(&self, state: &Self::State, builder: ChartBuilder<DB>) {}

    fn draw_chart<DB: DrawingBackend>(&self, state: &Self::State, root: DrawingArea<DB, plotters::coord::Shift>) {
        self.detector.build_chart_aux(&root, self.pixels, self.pixels_show, Margins { top: 5, bottom: 5, left: 5, right: 5 });
        //self.detector.build_chart_generic(&root, &self.source,self.scale,state);
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: iced::widget::canvas::Event,
        bounds: iced::Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> (iced::event::Status, Option<Msg>) {
        if let iced::mouse::Cursor::Available(point) = cursor {

            //println!("YOLO EVENT {:?}", event);
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

