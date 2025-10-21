use std::any::Any;
use std::thread::{self, JoinHandle};

use padamo_api::lazy_array_operations::{ArrayND, LazyTriSignal};

use crate::application::PadamoState;
use crate::detector_muxer::{get_mask_var, get_signal_var};

pub struct StoredSignal{
    pub signals: ArrayND<f64>,
    pub time: Vec<f64>,
    pub detector_id:usize,
    pub start_frame:usize,
}

impl StoredSignal {
    pub fn new(signals: ArrayND<f64>, time: Vec<f64>, detector_id: usize, start_frame:usize) -> Self {
        Self { signals, time, detector_id, start_frame }
    }

    pub fn request_data(signal: &LazyTriSignal, start:usize, end:usize, detector_id:usize)->Self{
        let signals = signal.0.request_range(start,end);
        let time = signal.1.request_range(start,end).to_vec();
        Self::new(signals, time, detector_id,start)
    }
}

pub struct DualSignalsCache{
    pub primary:StoredSignal,
    pub secondary:Option<StoredSignal>
}

impl DualSignalsCache{
    pub fn new(primary: StoredSignal, secondary: Option<StoredSignal>) -> Self {
        Self { primary, secondary }
    }
}

fn spawn_data_loader(padamo:&PadamoState, aux_detector:Option<usize>, start:usize, end:usize)->Option<JoinHandle<DualSignalsCache>>{
    // let primary_detector = padamo.detectors.get_primary()?;
    let signal = padamo.compute_graph.environment.request_detectorfulldata(get_signal_var(0).as_str()).ok()?;
    let signal_aux = if let Some(aux) = aux_detector{
        padamo.compute_graph.environment.request_detectorfulldata(get_signal_var(aux).as_str()).ok()
    }
    else{
        None
    };

    let worker = move || {
        let primary = StoredSignal::request_data(&signal, start, end, 0);
        let secondary = if let Some(aux) = signal_aux{
            let start1 = aux.1.find_unixtime(signal.1.request_range(start,start+1)[0]);
            let end1 = aux.1.find_unixtime(signal.1.request_range(end-1,end)[0])+1;
            if end1>start1{
                Some(StoredSignal::request_data(&aux, start1, end1, aux_detector.unwrap()))
            }
            else{
                None
            }
        }
        else{
            None
        };
        DualSignalsCache{
            primary, secondary
        }
    };

    Some(thread::spawn(worker))
}

pub enum CurrentData{
    Idle,
    PendingLoad(super::SyncDataRequest),
    Loading(JoinHandle<DualSignalsCache>, super::SyncDataRequest),
    Loaded(DualSignalsCache, super::SyncDataRequest),
    Unloaded,
    Error(Box<dyn Any+Send+'static>)
}


impl CurrentData{
    pub fn update_state(&mut self){
        *self = match std::mem::replace(self, CurrentData::Idle) {
            Self::Idle=>Self::Idle,
            Self::Loading(loader, request)=>{
                if loader.is_finished(){
                    match loader.join(){
                        Ok(r)=>Self::Loaded(r, request),
                        Err(e)=>Self::Error(e),
                    }
                }
                else{
                    Self::Loading(loader, request)
                }
            },
            res => res,
        }
    }

    pub fn update_state_context(&mut self, padamo:&mut PadamoState, safeguard:usize){
        *self = match std::mem::replace(self, CurrentData::Idle) {
            Self::PendingLoad(request)=>{
                if (request.end - request.start)<=safeguard{
                    if let Some(res) = spawn_data_loader(padamo, request.aux_detector_id, request.start, request.end){
                        Self::Loading(res,request)
                    }
                    else{
                        Self::PendingLoad(request)
                    }
                }
                else{
                    padamo.show_warning(format!("Interval is too long. {}>{}",request.end - request.start,safeguard));
                    Self::Unloaded
                }
            }
            res => res,
        }
    }

    pub fn pin_to_load(&mut self, padamo:&PadamoState, request:super::SyncDataRequest){
            if let Self::Loading(_,_) = self{
                return;
            }
            else{
                *self = Self::PendingLoad(request);
                // if let Some(res) = spawn_data_loader(padamo, request.aux_detector_id, request.start, request.end){
                //     *self = Self::Loading(res,request)
                // }
            }
    }

    // pub fn start_loader(&mut self, padamo:&PadamoState, request:super::SyncDataRequest){
    //     if let Self::Loading(_,_) = self{
    //         return;
    //     }
    //     else{
    //         if let Some(res) = spawn_data_loader(padamo, request.aux_detector_id, request.start, request.end){
    //             *self = Self::Loading(res,request)
    //         }
    //     }
    // }

    pub fn is_loading(&self)->bool{
        if let Self::Loading(_, _) = self{
            true
        }
        else{
            false
        }
    }

    pub fn is_too_long(&self)->bool{
        if let Self::Unloaded = self{
            true
        }
        else{
            false
        }
    }

    pub fn get_data_if_loaded<'a>(&mut self)->Option<(DualSignalsCache, super::SyncDataRequest)>{
        let (new_value, result) = match std::mem::replace(self, CurrentData::Idle){
            Self::Loaded(data, request)=>{
                (CurrentData::Idle, Some((data,request)))
            },
            other=>{
                (other, None)
            }
        };
        *self = new_value;
        result
    }
}
