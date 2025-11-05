use crate::DetectorAndMask;
use padamo_iced_forms::{ActionOrUpdate, IcedForm, IcedFormBuffer};
use serde::{Deserialize, Serialize};

#[derive(Clone,Debug)]
pub enum LoadedDetectorsMessage{
    Clear,
    // AddDetector(DetectorContent),
    AddDetector,
    SetPrimary(usize),
    EntryForm(usize, ActionOrUpdate<ProvidedDetectorInfoMessage>)
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, IcedForm)]

#[cfg_attr(feature = "abi_stable", derive(abi_stable::StableAbi))]
#[cfg_attr(feature = "abi_stable", repr(C))]
pub struct ProvidedDetectorInfo{
    #[field_name("Focal distance")] pub focal_distance: f64,
    #[field_name("Nickname")] pub nickname: Option<String>,
}

impl Default for ProvidedDetectorInfo{
    fn default() -> Self {
        Self {
            focal_distance: 100.0,
            nickname: None
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DetectorEntry{
    pub detector_info: ProvidedDetectorInfo,
    pub detector: DetectorAndMask,
    #[serde(skip_serializing, skip_deserializing)]
    pub buffer:ProvidedDetectorInfoBuffer,
}

impl DetectorEntry {
    pub fn new(detector_info: ProvidedDetectorInfo, detector: DetectorAndMask) -> Self {
        let mut buffer = ProvidedDetectorInfoBuffer::default();
        buffer.set(detector_info.clone());
        Self { detector_info, detector, buffer }
    }

    pub fn from_detector(detector: DetectorAndMask) -> Self {
        Self::new(Default::default(), detector)
    }

    pub fn sync_form(&mut self){
        self.buffer.set(self.detector_info.clone());
    }

    pub fn update(&mut self, msg:ProvidedDetectorInfoMessage){
        self.buffer.update(msg);
        if let Ok(v) = self.buffer.get(){
           self.detector_info = v;
        }
    }

    pub fn get_friendly_name(&self)->&str{
        if let Some(v) = &self.detector_info.nickname{
            v
        }
        else{
            &self.detector.cells.name
        }
    }
}


