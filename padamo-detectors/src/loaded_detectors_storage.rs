use crate::Detector;
use abi_stable::std_types::{ROption, RString, RVec};
use padamo_arraynd::ArrayND;
use padamo_iced_forms::{ActionOrUpdate, IcedForm, IcedFormBuffer};
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum DetectorLoadError{
    #[error("{0}")]
    Error(Box<dyn std::error::Error>),
    #[error("The load of detector was refused")]
    None,
}

impl DetectorLoadError{
    pub fn from_err<T:std::error::Error+'static>(value: T) -> Self {
        Self::Error(Box::new(value))
    }
}

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
    #[field_name("Rotation [°]")] pub rotation:f64,
}

impl Default for ProvidedDetectorInfo{
    fn default() -> Self {
        Self {
            focal_distance: 100.0,
            nickname: ROption::RNone,
            rotation:0.0,
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


#[derive(Clone,Debug, Serialize, Deserialize)]
pub struct LoadedDetectors{
    detectors: RVec<DetectorEntry>,
    #[serde(skip_serializing, skip_deserializing)]
    buffers:Vec<ProvidedDetectorInfoBuffer>,
}

#[allow(dead_code)]
impl LoadedDetectors{
    pub fn new()->Self{
        Self { detectors: RVec::new(), buffers: Vec::new() }
    }

    pub fn get_detectors(&self)->&RVec<DetectorEntry>{
        &self.detectors
    }

    pub fn get_primary(&self)->Option<&DetectorEntry>{
        self.detectors.get(0)
    }

    fn get_primary_buffer(&self)->Option<&ProvidedDetectorInfoBuffer>{
        self.buffers.get(0)
    }

    pub fn get_primary_mut(&mut self)->Option<&mut DetectorEntry>{
        self.detectors.get_mut(0)
    }

    pub fn get(&self, id:usize)->Option<&DetectorEntry>{
        self.detectors.get(id)
    }

    pub fn get_mut(&mut self, id:usize)->Option<&mut DetectorEntry>{
        self.detectors.get_mut(id)
    }

    pub fn set_primary_detector_by_index(&mut self, index:usize){
        if index<self.detectors.len() && index>0{
            self.detectors[..=index].rotate_left(index);
            self.buffers[..=index].rotate_left(index);
        }
    }

    pub fn add_detector(&mut self, detector:Detector){
        let det = DetectorEntry::from_detector(detector);
        self.buffers.push(ProvidedDetectorInfoBuffer::from_value(det.detector_info.clone()));
        self.detectors.push(det);
    }

    pub fn clear(&mut self){
        self.detectors.clear();
        self.buffers.clear();
    }

    pub fn iter_detectors(&self)->std::slice::Iter<'_, DetectorEntry>{
        self.detectors.iter()
    }

    pub fn iter_aux_detectors(&self)->std::iter::Skip<std::slice::Iter<'_, DetectorEntry>>{
        self.detectors.iter().skip(1)
    }

    pub fn len(&self)->usize{
        self.detectors.len()
    }

    pub fn sync_forms(&mut self){
        //self.detectors.iter_mut().for_each(|x| x.sync_form());
        if self.buffers.len()>self.detectors.len(){
            for _ in 0..(self.buffers.len()-self.detectors.len()){
                self.buffers.pop();
            }
        }

        if self.buffers.len()<self.detectors.len(){
            for _ in 0..(self.detectors.len()-self.buffers.len()){
                self.buffers.push(Default::default());
            }
        }

        for (detector, buffer) in self.detectors.iter().zip(self.buffers.iter_mut()){
            buffer.set(detector.detector_info.clone());
        }
    }

    pub fn process_message<T>(&mut self, msg:LoadedDetectorsMessage, adder:T)->Result<(),DetectorLoadError>
    where
        T:Fn()->Result<Detector,DetectorLoadError>
    {
        match msg{
            LoadedDetectorsMessage::Clear => self.clear(),
            // LoadedDetectorsMessage::AddDetector(det) => self.add_detector(det),
            LoadedDetectorsMessage::AddDetector => {

                //param workspace:&padamo_workspace::PadamoWorkspace

                // if let Some(path) = workspace.workspace("detectors").open_dialog(vec![("Detector",vec!["json"])]){
                //     let s = std::fs::read_to_string(path)?;
                //     let det = serde_json::from_str(&s)?;
                //     self.add_detector(det);
                // }
                let det = adder()?;
                self.add_detector(det);
            },
            LoadedDetectorsMessage::SetPrimary(id) => self.set_primary_detector_by_index(id),
            LoadedDetectorsMessage::EntryForm(id, msg)=>{
                match msg{
                    ActionOrUpdate::Update(form_msg)=>{
                        if let (Some(d), Some(b)) = (self.detectors.get_mut(id),self.buffers.get_mut(id)){
                            b.update(form_msg);
                            if let Ok(v) = b.get(){
                                d.detector_info = v;
                            }
                        }
                    },
                    ActionOrUpdate::Action(_)=>(),
                }
            }
        }
        Ok(())
    }

    pub fn view(&self)->iced::Element<'_, LoadedDetectorsMessage>{
        let mut res = iced::widget::column!();
        if let (Some(prim_det),Some(prim)) = (self.get_primary(),self.get_primary_buffer()){
            res = res.push(iced::widget::text(format!("Primary: {}",prim_det.detector.name)));
            res = res.push(prim.view(None).map(|x| LoadedDetectorsMessage::EntryForm(0, x)));
            res = res.push(iced::widget::rule::horizontal(3));
            for (i,d) in self.iter_aux_detectors().enumerate(){
                res = res.push(iced::widget::row![
                    iced::widget::button("S").on_press(LoadedDetectorsMessage::SetPrimary(i+1)),
                               iced::widget::text(format!("{}: {}",i+1,d.detector.name)),
                ]);
                res = res.push(self.buffers[i+1].view(None).map(move |x| LoadedDetectorsMessage::EntryForm(i+1, x)));
            }
        }
        else{
            res = res.push(iced::widget::text("No detectors loaded"));
        }
        res = res.push(iced::widget::row![
            iced::widget::button("Add").on_press(LoadedDetectorsMessage::AddDetector),
                       iced::widget::button("Clear").on_press(LoadedDetectorsMessage::Clear),
        ]);
        res.into()
    }
}
