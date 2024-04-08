use padamo_api::lazy_array_operations::LazyTimeSignal;
use chrono::{DateTime, Utc};


pub fn find_unixtime(op:&LazyTimeSignal,unixtime:f64)->usize{
    //let unixtime:f64 = (dt.naive_utc().timestamp_millis() as f64)
    let mut start:usize = 0;
    let mut end:usize = op.length();
    let mut middle:usize = (start+end)/2;
    if unixtime>op.request_range(end-1,end)[0]{
        return end-1;
    }
    if unixtime<op.request_range(0,1)[0]{
        return 0;
    }
    while start != middle{
        let item = op.request_range(middle,middle+1)[0];
        if item<=unixtime{
            start = middle;
        }
        if item>=unixtime{
            end = middle;
        }
        middle = (start+end)/2;
    }
    //println!("Datetime search result. req: {}, actual: {}",unixtime, op.request_item(middle));
    middle
}

pub fn find_time(op:&LazyTimeSignal,dt:DateTime<Utc>)->usize{
    let unixtime:f64 = (dt.naive_utc().timestamp_millis() as f64)*1e-3;
    find_unixtime(op, unixtime)
}
