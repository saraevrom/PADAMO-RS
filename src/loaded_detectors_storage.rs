use abi_stable::std_types::RVec;
use padamo_detectors::Detector;
pub use padamo_detectors::loaded_detectors_storage::{DetectorEntry, LoadedDetectorsMessage, ProvidedDetectorInfoBuffer};
use padamo_iced_forms::ActionOrUpdate;
use padamo_iced_forms::IcedFormBuffer;
use serde::{Deserialize, Serialize};

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
            self.detectors[..=index].rotate_right(index);
            self.buffers[..=index].rotate_right(index);
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

    pub fn process_message(&mut self, workspace:&padamo_workspace::PadamoWorkspace, msg:LoadedDetectorsMessage)->anyhow::Result<()>{
        match msg{
            LoadedDetectorsMessage::Clear => self.clear(),
            // LoadedDetectorsMessage::AddDetector(det) => self.add_detector(det),
            LoadedDetectorsMessage::AddDetector => {
                if let Some(path) = workspace.workspace("detectors").open_dialog(vec![("Detector",vec!["json"])]){
                    let s = std::fs::read_to_string(path)?;
                    let det = serde_json::from_str(&s)?;
                    self.add_detector(det);
                }
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
