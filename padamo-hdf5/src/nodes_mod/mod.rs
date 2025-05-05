pub mod singular;
pub use singular::LazyHDF5SignalNode;

pub mod joined;
pub use joined::LazyHDF5DirSignalNode;

pub mod array_source;
pub use array_source::LazyHDF5ArrayNode;

pub mod time_source;
pub use time_source::LazyHDF5TimeNode;

pub mod save;
pub use save::SaveHDF5Node;
pub use save::SaveHDF5ArrayNode;

use abi_stable::sabi_trait::prelude::TD_Opaque;


use crate::ops::{LazyHDF5Reader3D,ArrayCaster,UnsignedToFloatArrayCaster};
use padamo_api::lazy_array_operations::{LazyArrayOperationBox,LazyDetectorSignal};
use padamo_api::lazy_array_operations::ndim_array;


pub fn make_spatial(filename:String, spatial:String)->Result<LazyDetectorSignal,hdf5::Error>{
    let first_reader = LazyHDF5Reader3D::<f64>::new(filename.clone().into(), spatial.clone())?;
    //first_reader.print_dtype();
    if first_reader.check_read(){
        return Ok(LazyDetectorSignal::from_value(first_reader,TD_Opaque));
    }
    else{
        println!("Failed reading as f64");
    }

    let reader_u64 = LazyHDF5Reader3D::<u64>::new(filename.clone().into(), spatial.clone())?;
    if reader_u64.check_read(){
        let ubox:LazyArrayOperationBox<ndim_array::ArrayND<u64>> = LazyArrayOperationBox::from_value(reader_u64,TD_Opaque);
        let conv = UnsignedToFloatArrayCaster::new(ubox);
        let res = LazyDetectorSignal::from_value(conv,TD_Opaque);
        return Ok(res);
    }
    else{
        println!("Failed reading as u64");
    }

    let reader_u32 = LazyHDF5Reader3D::<u32>::new(filename.clone().into(), spatial.clone())?;
    if reader_u32.check_read(){
        let ubox:LazyArrayOperationBox<ndim_array::ArrayND<u32>> = LazyArrayOperationBox::from_value(reader_u32,TD_Opaque);
        let conv = ArrayCaster::new(ubox);
        let res = LazyDetectorSignal::from_value(conv,TD_Opaque);
        return Ok(res);
    }
    else{
        println!("Failed reading as u32");
    }

    let reader_i32 = LazyHDF5Reader3D::<i32>::new(filename.clone().into(), spatial)?;
    if reader_i32.check_read(){
        let ubox:LazyArrayOperationBox<ndim_array::ArrayND<i32>> = LazyArrayOperationBox::from_value(reader_i32,TD_Opaque);
        let conv = ArrayCaster::new(ubox);
        let res = LazyDetectorSignal::from_value(conv,TD_Opaque);
        return Ok(res);
    }
    else{
        println!("Failed reading as i32");
    }

    panic!("Could not determine reader type");
}
