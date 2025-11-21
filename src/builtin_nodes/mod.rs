use padamo_api::prelude::CalculationNodeArguments;
use padamo_detectors::loaded_detectors_storage::DetectorEntry;

pub mod viewer;
pub mod viewer_smart;

pub fn register_nodes(nodes:&mut crate::nodes_interconnect::NodesRegistry){
    viewer::register_nodes(nodes);
    viewer_smart::register_nodes(nodes);
}

pub fn find_detector<'a>(args:&'a CalculationNodeArguments, detector_name:&'a str) -> Option<&'a DetectorEntry>{
    let mut detector = None;
    for det in args.detectors.iter(){
        if det.get_friendly_name()==detector_name{
            detector = Some(det);
            break;
        }
    }
    detector
}

pub fn find_primary_detector<'a>(args:&'a CalculationNodeArguments) -> Option<&'a DetectorEntry>{
    // let mut detector = None;
    // for det in args.detectors.iter(){
    //     if det.get_friendly_name()==detector_name{
    //         detector = Some(det);
    //         break;
    //     }
    // }
    // detector
    args.detectors.get(0)
}
