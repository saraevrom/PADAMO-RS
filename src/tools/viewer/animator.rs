use std::{sync::mpsc, thread};
use crate::messages::PadamoAppMessage;

use super::{AnimationParameters, Worker};
use padamo_detectors::{DetectorAndMask, DetectorPlotter, Scaling};
use plotters::coord::Shift;
use plotters::prelude::*;


pub fn make_frame<'a,T:plotters_backend::DrawingBackend+Send+Sync+'a>(root:&'a DrawingArea<T,Shift>,spatial:&'a padamo_api::lazy_array_operations::LazyDetectorSignal,
               temporal:&'a padamo_api::lazy_array_operations::LazyTimeSignal,
               temporal_primary:&'a padamo_api::lazy_array_operations::LazyTimeSignal, current_frame:usize, chart:&DetectorPlotter<PadamoAppMessage>, detector:&DetectorAndMask,plot_scale:Scaling){

    let unixtime = temporal_primary.request_range(current_frame,current_frame+1)[0];
    let remapped_frame = temporal.find_unixtime(unixtime);

    let mut frame = spatial.request_range(remapped_frame,remapped_frame+1);
    frame.shape.drain(0..1);
    let tim = temporal.request_range(remapped_frame,remapped_frame+1)[0];
    root.fill(&WHITE).unwrap();
    chart.build_chart_generic(detector,root,&Some((&frame,tim)),plot_scale,Default::default(),&None);
}

pub fn animate<T:plotters_backend::DrawingBackend+Send+Sync+'static>(root:T,spatial:padamo_api::lazy_array_operations::LazyDetectorSignal,
               temporal:padamo_api::lazy_array_operations::LazyTimeSignal,
               temporal_main:padamo_api::lazy_array_operations::LazyTimeSignal,
               start:usize,
               end:usize,
               animation_parameters:AnimationParameters,
               chart:DetectorPlotter<PadamoAppMessage>, detector:DetectorAndMask, plot_scale:Scaling)->Worker<String>{
    //let signal = signal_ref.clone();

    let (tx,rx) = mpsc::channel::<bool>();

    let (tx_status,rx_status) = mpsc::channel::<String>();
    //let status = self.animation_status.clone();

    let handle = thread::spawn( move || {
        //80 pixels for colormap

        //let height = if animation_parameters.displaylc {animation_parameters.height+animation_parameters.lcheight} else {animation_parameters.height};
        //BitMapBackend::gif(filename,(animation_parameters.width+80, height), animation_parameters.framedelay)

        let root = root.into_drawing_area();
        // let (plot_root,lc_plot,root) = if animation_parameters.displaylc{
        //     let (a,b) = root.split_vertically(height);
        //     //b.fill(&WHITE);
        //     (a,Some(b),oot)
        // }
        // else{
        //     (root,None)
        // };
        let lc_pair = if animation_parameters.displaylc{
            //let (a,b) = root.split_vertically(height);
            let space_out = spatial.request_range(start,end);
            let mut lc:Vec<f64> = Vec::with_capacity(end-start);
            lc.resize(end-start, 0.0);
            let pixel_count = (space_out.flat_data.len()/(end-start)) as f64;

            for index in space_out.enumerate(){
                lc[index[0]] += space_out[&index]/pixel_count;
            }
            let llc = lc.iter().min_by(|a, b| a.partial_cmp(b).unwrap());
            let mlc = lc.iter().max_by(|a, b| a.partial_cmp(b).unwrap());
            match (llc,mlc) {
                        (Some(minv), Some(maxv)) =>{Some((*minv,*maxv,lc))},
                        _=>{None}
            }
        }
        else{
            None
        };

        for i in start..end{
            //let t1 = Instant::now();
            //if (i-start)%10==0{
            tx_status.send(format!("{}/{}",i-start,end-start)).unwrap();

            //}
            //let report_time = t1.elapsed().as_secs_f64();

                //let time_start = Instant::now();

            //let mut frame = spatial.request_range(i,i+1);
            //frame.shape.drain(0..1);
            //let tim = temporal.request_range(i,i+1)[0];

            root.fill(&WHITE).unwrap();
            if let Some((low,high,lc)) = &lc_pair{
                let (a,b) = root.split_vertically(animation_parameters.height);
                //a.fill(&WHITE).unwrap();
                // chart.build_chart_generic(&a,&Some((&frame,tim)),plot_scale,Default::default(),&None);
                make_frame(&a, &spatial, &temporal, &temporal_main, i, &chart, &detector, plot_scale);

                //b.fill(&WHITE).unwrap();
                let mut chart = ChartBuilder::on(&b)
                    .x_label_area_size(0.0)
                    .y_label_area_size(0.0)
                    .margin(1.0)
                    .build_cartesian_2d((start as f64)..(end as f64), *low..*high)
                    .unwrap();

                chart.configure_mesh().disable_x_mesh().axis_style(&BLACK).draw().unwrap();
                chart.draw_series(LineSeries::new((start..end).map(|j| (j as f64,lc[j-start])), &BLACK)).unwrap();
                let ptr = vec![(i as f64,*low),(i as f64,*high)];
                chart.draw_series(LineSeries::new((0..2).map(|j| ptr[j]), RED)).unwrap();
            }
            else{
                //chart.build_chart_generic(&root,&Some((&frame,tim)),plot_scale,Default::default(),&None);
                make_frame(&root, &spatial, &temporal, &temporal_main, i, &chart, &detector, plot_scale);
            }



            //let chart_time = t1.elapsed().as_secs_f64();


            if let Err(e) = root.present(){
                println!("{:?}",e);
            };
            //let preirq = t1.elapsed().as_secs_f64();
            if let Ok(v) = rx.try_recv(){
                if v{
                    println!("Interrupt requested");
                    break;
                }
            }


            //let fin_time = t1.elapsed().as_secs_f64();
            //println!("PROFILING {}/{}/{}/{}",report_time,chart_time,preirq,fin_time);
            //padamo.compute_graph.borrow().

        }
        tx_status.send("END".into()).unwrap();
    });

    Worker::new(handle, tx, rx_status)
}
