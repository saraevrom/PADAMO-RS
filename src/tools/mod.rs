pub mod viewer;
pub mod editor;
pub mod plotter;
pub mod trigger;
pub mod detectors;

use std::rc::Rc;

pub trait PadamoTool{
    fn tab_label(&self) -> iced_aw::TabLabel {
        iced_aw::TabLabel::Text(self.tab_name())
    }

    fn tab_name(&self)->String;
    fn view<'a>(&'a self)->iced::Element<'a, crate::messages::PadamoAppMessage>;
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
