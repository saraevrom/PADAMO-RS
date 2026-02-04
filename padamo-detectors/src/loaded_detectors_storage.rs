use crate::Detector;
use abi_stable::std_types::{ROption, RString};
use padamo_arraynd::ArrayND;
use padamo_iced_forms::{ActionOrUpdate, IcedForm, IcedFormBuffer};

#[derive(Clone,Debug)]
pub enum LoadedDetectorsMessage{
    Clear,
    // AddDetector(DetectorContent),
    AddDetector,
    SetPrimary(usize),
    EntryForm(usize, ActionOrUpdate<ProvidedDetectorInfoMessage>)
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, IcedForm)]
#[derive(abi_stable::StableAbi)]
#[repr(C)]
pub struct ProvidedDetectorInfo{
    #[field_name("Focal distance")] pub focal_distance: f64,
    #[field_name("Nickname")] pub nickname: ROption<RString>,
}

impl Default for ProvidedDetectorInfo{
    fn default() -> Self {
        Self {
            focal_distance: 100.0,
            nickname: ROption::RNone
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[derive(abi_stable::StableAbi)]
#[repr(C)]
pub struct DetectorEntry{
    pub detector_info: ProvidedDetectorInfo,
    pub detector: Detector,
    pub mask:ArrayND<bool>,
    pub selection:ArrayND<bool>,
    // #[serde(skip_serializing, skip_deserializing)]
    // pub buffer:ProvidedDetectorInfoBuffer,
}

impl DetectorEntry {
    pub fn new(detector_info: ProvidedDetectorInfo, detector: Detector) -> Self {
        // let mut buffer = ProvidedDetectorInfoBuffer::default();
        // buffer.set(detector_info.clone());
        let mask = ArrayND::new(detector.shape().to_vec(), true);
        let selection = ArrayND::new(detector.shape().to_vec(), false);
        // let selection =
        Self { detector_info, detector, mask, selection}
    }

    pub fn from_detector(detector: Detector) -> Self {
        Self::new(Default::default(), detector)
    }

    // pub fn sync_form(&mut self){
    //     self.buffer.set(self.detector_info.clone());
    // }
    //
    // pub fn update(&mut self, msg:ProvidedDetectorInfoMessage){
    //     self.buffer.update(msg);
    //     if let Ok(v) = self.buffer.get(){
    //        self.detector_info = v;
    //     }
    // }

    pub fn get_friendly_name(&self)->&str{
        if let ROption::RSome(v) = &self.detector_info.nickname{
            v
        }
        else{
            &self.detector.name
        }
    }
}


