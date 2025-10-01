pub mod viewer;
pub mod editor;
pub mod plotter;
pub mod trigger;
pub mod detectors;

use std::rc::Rc;

#[allow(unused_variables)]
pub trait PadamoTool{
    fn tab_label(&self) -> iced_aw::TabLabel {
        iced_aw::TabLabel::Text(self.tab_name())
    }

    fn tab_name(&self)->String;
    fn view<'a>(&'a self, padamo:&'a PadamoState)->iced::Element<'a, crate::messages::PadamoAppMessage>;

    fn initialize(&mut self, padamo:crate::application::PadamoStateRef){

    }

    fn update(&mut self, msg: Rc<crate::messages::PadamoAppMessage>, padamo:crate::application::PadamoStateRef){

    }
    fn late_update(&mut self, msg: Rc<crate::messages::PadamoAppMessage>, padamo:crate::application::PadamoStateRef)->Option<crate::messages::PadamoAppMessage>{
        None
    }
    fn context_update(&mut self, msg: Rc<crate::messages::PadamoAppMessage>, padamo:crate::application::PadamoStateRef){

    }
}


pub use viewer::PadamoViewer;
pub use editor::PadamoEditor;
pub use plotter::Plotter;
pub use trigger::PadamoTrigger;
pub use detectors::PadamoDetectorManager;

use crate::application::PadamoState;
