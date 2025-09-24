use padamo_detectors::{polygon::DetectorContent, Detector};

pub struct LoadedDetectors{
    detectors: Vec<DetectorContent>
}

impl LoadedDetectors{
    pub fn get_primary(&self)->Option<&DetectorContent>{
        self.detectors.get(0)
    }

    pub fn set_primary_detector_by_index(&mut self, index:usize){
        if index<self.detectors.len() && index>0{
            self.detectors[..=index].rotate_right(index);
        }
    }

    pub fn add_detector(&mut self, detector:DetectorContent){
        self.detectors.push(detector);
    }

    pub fn clear(&mut self){
        self.detectors.clear();
    }

    pub fn iter_detectors(&self)->std::slice::Iter<'_, DetectorContent>{
        self.detectors.iter()
    }

    pub fn iter_aux_detectors(&self)->std::iter::Skip<std::slice::Iter<'_, DetectorContent>>{
        self.detectors.iter().skip(1)
    }
}
