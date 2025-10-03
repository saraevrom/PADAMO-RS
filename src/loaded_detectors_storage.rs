use padamo_detectors::DetectorAndMask;
use serde::{Deserialize, Serialize};

#[derive(Clone,Debug)]
pub enum LoadedDetectorsMessage{
    Clear,
    // AddDetector(DetectorContent),
    AddDetector,
    SetPrimary(usize)
}


#[derive(Clone,Debug, Serialize, Deserialize)]
pub struct LoadedDetectors{
    detectors: Vec<DetectorAndMask>
}

impl LoadedDetectors{
    pub fn new()->Self{
        Self { detectors: Vec::new() }
    }

    pub fn get_primary(&self)->Option<&DetectorAndMask>{
        self.detectors.get(0)
    }

    pub fn get_primary_mut(&mut self)->Option<&mut DetectorAndMask>{
        self.detectors.get_mut(0)
    }

    pub fn get(&self, id:usize)->Option<&DetectorAndMask>{
        self.detectors.get(id)
    }

    pub fn get_mut(&mut self, id:usize)->Option<&mut DetectorAndMask>{
        self.detectors.get_mut(id)
    }

    pub fn set_primary_detector_by_index(&mut self, index:usize){
        if index<self.detectors.len() && index>0{
            self.detectors[..=index].rotate_right(index);
        }
    }

    pub fn add_detector(&mut self, detector:DetectorAndMask){
        self.detectors.push(detector);
    }

    pub fn clear(&mut self){
        self.detectors.clear();
    }

    pub fn iter_detectors(&self)->std::slice::Iter<'_, DetectorAndMask>{
        self.detectors.iter()
    }

    pub fn iter_aux_detectors(&self)->std::iter::Skip<std::slice::Iter<'_, DetectorAndMask>>{
        self.detectors.iter().skip(1)
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
        }
        Ok(())
    }

    pub fn view(&self)->iced::Element<'_, LoadedDetectorsMessage>{
        let mut res = iced::widget::column!();
        if let Some(prim) = self.get_primary(){
            res = res.push(iced::widget::text(format!("Primary: {}",prim.cells.name)));
            res = res.push(iced::widget::horizontal_rule(3));
            for (i,d) in self.iter_aux_detectors().enumerate(){
                res = res.push(iced::widget::row![
                    iced::widget::button("S").on_press(LoadedDetectorsMessage::SetPrimary(i+1)),
                    iced::widget::text(format!("{}: {}",i+1,d.cells.name)),
                ]);
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
