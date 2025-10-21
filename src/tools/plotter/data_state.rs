use std::thread;

use padamo_api::lazy_array_operations::ArrayND;

pub enum DataState{
    NoData,
    PendingLoad(usize,usize),
    Loading(thread::JoinHandle<DataCache>),
    Loaded(DataCache)
}

#[derive(Clone,Debug)]
pub struct DataCache{
    // pub signal:ArrayND<f64>,
    // pub time:Vec<f64>,
    pub primary:(ArrayND<f64>, Vec<f64>),
    pub secondary:Option<(ArrayND<f64>, Vec<f64>)>,

    pub time_step: f64,
    pub lc: Vec<f64>,
    pub pixel_count:usize,
    pub start:usize,
    pub end:usize,
    pub minv:f64,
    pub maxv:f64,
    pub last_indices:(usize,usize)
}

impl DataState{
    pub fn apply_start_end(&mut self, start:usize, end:usize){
        match self {
            DataState::NoData => *self = Self::PendingLoad(start, end),
            DataState::PendingLoad(_, _) => *self = Self::PendingLoad(start, end),

            DataState::Loading(_) => (),
            DataState::Loaded(data) => {
                if data.last_indices != (start,end)
                {
                    *self = Self::PendingLoad(start, end);
                }
            },
        }
    }

    pub fn take_worker(&mut self)->Option<thread::JoinHandle<DataCache>>{
        if let Self::Loading(_) = self{
            let moved = std::mem::replace(self, Self::NoData);
            if let Self::Loading(v) = moved{
                Some(v)
            }
            else{
                None
            }
        }
        else{
            None
        }
    }
}
