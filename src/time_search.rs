use padamo_api::lazy_array_operations::LazyTimeSignal;
use chrono::{DateTime, Utc};


pub fn find_unixtime(op:&LazyTimeSignal,unixtime:f64)->usize{
    //let unixtime:f64 = (dt.naive_utc().timestamp_millis() as f64)
    // let mut start:usize = 0;
    // let op_length = op.length();
    // let mut end:usize = op_length;
    // let mut middle:usize = (start+end)/2;
    // if unixtime>op.request_range(end-1,end)[0]{
    //     return end-1;
    // }
    // if unixtime<op.request_range(0,1)[0]{
    //     return 0;
    // }
    // while start != middle{
    //     let item = op.request_range(middle,middle+1)[0];
    //     if item<=unixtime{
    //         start = middle;
    //     }
    //     if item>=unixtime{
    //         end = middle;
    //     }
    //     middle = (start+end)/2;
    // }
    // //println!("Datetime search result. req: {}, actual: {}",unixtime, op.request_item(middle));
    // let mut res = middle;
    // if middle>0{
    //     let twoval = op.request_range(middle-1,middle+1);
    //     if (twoval[0]-unixtime).abs()<(twoval[1]-unixtime).abs(){
    //         res = middle-1;
    //     }
    // }
    // if middle<op_length-1{
    //     let twoval = op.request_range(middle,middle+2);
    //     if (twoval[0]-unixtime).abs()>(twoval[1]-unixtime).abs(){
    //         res = middle+1;
    //     }
    // }
    // res
    op.find_unixtime(unixtime)
}

pub fn find_time(op:&LazyTimeSignal,dt:DateTime<Utc>)->usize{
    // let unixtime:f64 = (dt.naive_utc().and_utc().timestamp_micros() as f64)*1e-6;
    // find_unixtime(op, unixtime)
    op.find_time(dt)
}
