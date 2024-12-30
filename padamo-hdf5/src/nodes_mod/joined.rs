use abi_stable::rvec;
use abi_stable::std_types::ROption::RSome;
use abi_stable::std_types::{RString,ROption};
use padamo_api::prelude::*;
use padamo_api::{ports, constants};
use abi_stable::std_types::RVec;
use crate::ops::{LazyHDF5Reader3D,LazyTimeHDF5Reader};
use padamo_api::lazy_array_operations::{LazyTriSignal,LazyDetectorSignal,LazyTimeSignal};
use abi_stable::sabi_trait::prelude::TD_Opaque;
use std::fs;

#[derive(Debug,Clone)]
pub struct LazyHDF5DirSignalNode;

fn read_time_key(filename:&str, temporal:&str)->f64{
    let hdf5_file = hdf5::File::open(&filename).unwrap();
    let dataset = hdf5_file.dataset(&temporal).unwrap();

    let s = dataset.shape();
    if s.len()>2{
        panic!("Wrong time format");
    }
    if s.len()>1{
        println!("MATLAB time detected in {}", temporal);
        let sliced = dataset.read_slice_2d::<f64,_>((0..1,0..1)).unwrap();
        println!("{:?}",sliced);
        sliced.into_raw_vec()[0]
    }
    else{
        let sliced = dataset.read_slice_1d::<f64,_>(0..1).unwrap();
        println!("{:?}",sliced);
        sliced.into_raw_vec()[0]
    }

}

fn has_spacetime(filename:&str,spatial:&str, temporal:&str)->bool{
    let hdf5_file = if let Ok(v) = hdf5::File::open(&filename) {v} else {return false};
    let dataset1 = if let Ok(v) = hdf5_file.dataset(&temporal) {v} else {return false};
    let _dataset2 = if let Ok(v) = hdf5_file.dataset(&spatial) {v} else {return false};

    let s = dataset1.shape();
    if s.len()>2{
        return false;
    }
    true
}

impl LazyHDF5DirSignalNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let dirname:String = args.inputs.request_string("Dirname")?.into();

        let spatial:String = args.constants.request_string("Spatial")?.into();
        let temporal:String = args.constants.request_string("Temporal")?.into();

        let paths = fs::read_dir(&dirname).map_err(|x| ExecutionError::OtherError(format!("{:?}",x).into()))?;

        let mut paths_done = Vec::new();
        for item in paths{
            let v = item.map_err(|x| ExecutionError::OtherError(format!("{:?}",x).into()))?;
            let path = v.path();
            if path.is_file(){
                if let Some(s) = path.extension(){
                    if let Some(ext) = s.to_str(){
                        if ext=="h5" || ext=="mat"{
                            let spath = path.to_str().unwrap().to_string();
                            if has_spacetime(&spath,&spatial,&temporal){
                                println!("Detected {}", &spath);
                                paths_done.push(spath);
                            }
                        }
                    }

                }
            }
        }

        println!("Sorting...");
        paths_done.sort_by(|a,b| read_time_key(a,&temporal).partial_cmp(&read_time_key(b,&temporal)).unwrap());
        println!("Sorted");

        let mut spatial_reader:Option<LazyDetectorSignal> = None;
        let mut temporal_reader:Option<LazyTimeSignal> = None;

        for path in paths_done.iter(){
            println!("Adding {}", &path);
            let new_spatial_reader = LazyHDF5Reader3D::<f64>::new(path.clone().into(), spatial.clone());
            let new_temporal_reader = LazyTimeHDF5Reader::<f64>::new(path.into(), temporal.clone());
            match (new_spatial_reader,new_temporal_reader){
                (Ok(sp),Ok(tmp))=>{
                    let spatial_box = LazyDetectorSignal::from_value(sp,TD_Opaque);
                    let temporal_box = LazyTimeSignal::from_value(tmp,TD_Opaque);
                    if let Some(pre) = spatial_reader.take(){
                        spatial_reader = Some(pre.merge(spatial_box));
                    }
                    else{
                        spatial_reader = Some(spatial_box);
                    }

                    if let Some(pre) = temporal_reader.take(){
                        temporal_reader = Some(pre.merge(temporal_box));
                    }
                    else{
                        temporal_reader = Some(temporal_box);
                    }

                    // let signal:LazyTriSignal = (LazyDetectorSignal::from_value(sp,TD_Opaque),LazyTimeSignal::from_value(tmp,TD_Opaque) ,ROption::RNone).into();
                    // outputs.set_value("Signal", Content::DetectorFullData(signal))
                },
                (Err(sp),Err(tmp))=>{
                    return Err(ExecutionError::OtherError(format!("HDF error (spatiotemporal): {}; {}",sp,tmp).into()));
                },
                (Ok(_),Err(tmp))=>{
                    return Err(ExecutionError::OtherError(format!("HDF error (temporal): {}",tmp).into()));
                },
                (Err(sp),Ok(_))=>{
                    return Err(ExecutionError::OtherError(format!("HDF error (spatial): {}",sp).into()));
                }
            }


        }

        // let spatial_reader = LazyHDF5Reader3D::<f64>::new(filename.clone().into(), spatial);
        // let temporal_reader = LazyTimeHDF5Reader::<f64>::new(filename.into(), temporal);
        // match (spatial_reader,temporal_reader){
        //     (Ok(sp),Ok(tmp))=>{
        //         let signal:LazyTriSignal = (LazyDetectorSignal::from_value(sp,TD_Opaque),LazyTimeSignal::from_value(tmp,TD_Opaque) ,ROption::RNone).into();
        //         outputs.set_value("Signal", Content::DetectorFullData(signal))
        //     },
        //     (Err(sp),Err(tmp))=>{
        //         Err(ExecutionError::OtherError(format!("HDF error (spatiotemporal): {}; {}",sp,tmp).into()))
        //     },
        //     (Ok(_),Err(tmp))=>{
        //         Err(ExecutionError::OtherError(format!("HDF error (temporal): {}",tmp).into()))
        //     },
        //     (Err(sp),Ok(_))=>{
        //         Err(ExecutionError::OtherError(format!("HDF error (spatial): {}",sp).into()))
        //     }
        // }
        if let (Some(sp), Some(tmp)) = (spatial_reader,temporal_reader){
            let signal:LazyTriSignal = (sp,tmp ,ROption::RNone).into();
            args.outputs.set_value("Signal", Content::DetectorFullData(signal))
        }
        else{
            Err(ExecutionError::OtherError("No files to join".into()))
        }

    }
}

impl CalculationNode for LazyHDF5DirSignalNode{
    fn name(&self,) -> RString where {
        "Lazy HDF5 Signal directory node".into()
    }

    fn category(&self,) -> abi_stable::std_types::RVec<abi_stable::std_types::RString>where {
        rvec![
            "HDF5".into()
        ]
    }

    fn old_identifier(&self,) -> ROption<RString>where {
        RSome("HDF5/Lazy HDF5 Signal directory node".into())
    }

    fn identifier(&self,) -> RString where {
        "padamohdf5.directory_reader".into()
    }

    fn inputs(&self) -> RVec<CalculationIO>{
        ports!(
            ("Dirname", ContentType::String)
        )
    }

    fn outputs(&self) -> RVec<CalculationIO> {
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!(
            ("Spatial", "pdm_2d_rot_global"),
            ("Temporal", "unixtime_dbl_global")
        )
    }

    fn calculate(&self, args:CalculationNodeArguments) -> abi_stable::std_types::RResult<(),ExecutionError> {
        self.calculate(args).into()
    }

}
