
use padamo_arraynd::ArrayND;
use plotters::{coord::ReverseCoordTranslate, prelude::*};
use plotters_iced::Chart;


use crate::{DetectorPlotter, DetectorAndMask, Margins};

pub struct DetectorChartMap<'a,Msg,F1,F2>
where
    F1:'static + Fn(Vec<usize>)->Msg,
    F2:'static + Fn(Vec<usize>)->Msg,
{
    detector_plotter:&'a DetectorPlotter<Msg>,
    detector:&'a DetectorAndMask,
    //source:Option<(&'a ArrayND<f64>,f64)>,
    //scale:Scaling,
    // pixels:&'a Vec<Vec<usize>>,
    // pixels_show:&'a Vec<bool>,
    pixel_mask:&'a ArrayND<bool>,
    lclick_event:Option<F1>,
    rclick_event:Option<F2>,
    transform: Option<crate::transformer::Transform>,
}

impl<'a,Msg,F1,F2> DetectorChartMap<'a,Msg,F1,F2>
where
    F1:'static + Fn(Vec<usize>)->Msg,
    F2:'static + Fn(Vec<usize>)->Msg,
{
    pub fn new(detector_plotter:&'a DetectorPlotter<Msg>,detector:&'a DetectorAndMask, pixel_mask:&'a ArrayND<bool>, transform:Option<crate::transformer::Transform>, lclick_event:Option<F1>, rclick_event:Option<F2>)->Self{
        Self { detector_plotter, detector ,pixel_mask, lclick_event, rclick_event, transform}
    }
}


impl<'a,Msg,F1,F2> Chart<Msg> for DetectorChartMap<'a,Msg,F1,F2>
where
    F1:'static + Fn(Vec<usize>)->Msg,
    F2:'static + Fn(Vec<usize>)->Msg,
{
    type State = Option<((f64,f64),(i32,i32))>;


    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, _builder: ChartBuilder<DB>) {}

    fn draw_chart<DB: DrawingBackend>(&self, state: &Self::State, root: DrawingArea<DB, plotters::coord::Shift>) {
        self.detector_plotter.build_chart_aux_simple(&self.detector,&root, self.pixel_mask, Margins { top: 5, bottom: 5, left: 5, right: 5 }, self.transform, *state);
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
                    if let Some(spec) = self.detector_plotter.spec.borrow().as_ref(){
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

