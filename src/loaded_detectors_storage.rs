use padamo_detectors::DetectorAndMask;
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


#[derive(Clone,Debug, Serialize, Deserialize)]
pub struct LoadedDetectors{
    detectors: Vec<DetectorEntry>,
}

impl LoadedDetectors{
    pub fn new()->Self{
        Self { detectors: Vec::new() }
    }

    pub fn get_primary(&self)->Option<&DetectorEntry>{
        self.detectors.get(0)
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
            self.detectors[..=index].rotate_right(index);
        }
    }

    pub fn add_detector(&mut self, detector:DetectorAndMask){
        self.detectors.push(DetectorEntry::from_detector(detector));
    }

    pub fn clear(&mut self){
        self.detectors.clear();
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
        self.detectors.iter_mut().for_each(|x| x.sync_form());
    }

    pub fn process_message(&mut self, workspace:&padamo_workspace::PadamoWorkspace, msg:LoadedDetectorsMessage)->anyhow::Result<()>{
        match msg{
            LoadedDetectorsMessage::Clear => self.clear(),
            // LoadedDetectorsMessage::AddDetector(det) => self.add_detector(det),
            LoadedDetectorsMessage::AddDetector => {
                if let Some(path) = workspace.workspace("detectors").open_dialog(vec![("Detector",vec!["json"])]){
                    let s = std::fs::read_to_string(path)?;
                    let det = serde_json::from_str(&s)?;
                    self.add_detector(DetectorAndMask::from_cells(det));
                }
            },
            LoadedDetectorsMessage::SetPrimary(id) => self.set_primary_detector_by_index(id),
            LoadedDetectorsMessage::EntryForm(id, msg)=>{
                match msg{
                    ActionOrUpdate::Update(form_msg)=>{
                        if let Some(d) = self.detectors.get_mut(id){
                            d.update(form_msg);
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
        if let Some(prim) = self.get_primary(){
            res = res.push(iced::widget::text(format!("Primary: {}",prim.detector.cells.name)));
            res = res.push(prim.buffer.view(None).map(|x| LoadedDetectorsMessage::EntryForm(0, x)));
            res = res.push(iced::widget::horizontal_rule(3));
            for (i,d) in self.iter_aux_detectors().enumerate(){
                res = res.push(iced::widget::row![
                    iced::widget::button("S").on_press(LoadedDetectorsMessage::SetPrimary(i+1)),
                    iced::widget::text(format!("{}: {}",i+1,d.detector.cells.name)),
                ]);
                res = res.push(d.buffer.view(None).map(move |x| LoadedDetectorsMessage::EntryForm(i+1, x)));
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
