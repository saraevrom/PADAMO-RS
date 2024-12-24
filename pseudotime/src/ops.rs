use abi_stable::std_types::RVec;
use padamo_api::lazy_array_operations::LazyArrayOperation;


#[derive(Clone,Debug)]
pub struct AddTime{
    pub length:usize,
    pub step:f64,
    pub offset_time:f64,
}

impl AddTime {
    pub fn new(length: usize, step: f64, offset_time: &str) -> Option<Self> {
        let dt = if let Some(v) = datetime_parser::parse_datetimes(offset_time, chrono::Utc::now()) {v}
            else {return None;};
        let tmpbase = (dt.naive_utc().and_utc().timestamp_micros() as f64)*1e-6;
        //let tmpres = constants.request_float("tmpres")?;
        let temporal = Self{length, step, offset_time:tmpbase};
        Some(temporal)
    }
}

impl LazyArrayOperation<RVec<f64>> for AddTime{
    fn length(&self,) -> usize where {
        self.length
    }

    fn calculate_overhead(&self,start:usize,end:usize,) -> usize where {
        end-start
    }

    fn request_range(&self,start:usize,end:usize,) -> RVec<f64>where {
        (start..end).map(|i| self.offset_time+self.step*(i as f64)).collect()
    }
}
