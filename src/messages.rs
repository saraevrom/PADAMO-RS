use iced::widget::scrollable;

use crate::tools::editor::messages::{EditorCanvasMessage,EditorMessage};
use crate::tools::viewer::ViewerMessage;
use crate::tools::plotter::messages::PlotterMessage;
use crate::tools::trigger::messages::TriggerMessage;
use crate::tools::detectors::messages::DetectorManagerMessage;
use iced::font;



#[derive(Clone,Debug)]
pub enum PadamoAppMessage{
    Noop,
    TabSelect(usize),
    FontLoaded(Result<(), font::Error>),
    EditorMessage(EditorMessage),
    ViewerMessage(ViewerMessage),
    PlotterMessage(PlotterMessage),
    TriggerMessage(TriggerMessage),
    DetectorManagerMessage(DetectorManagerMessage),
    ChooseDetector,
    SetDetector(padamo_detectors::polygon::DetectorContent),
    PopupMessageClick,
    Run,
    RerollRun,
    SetSeed(String),
    Open,
    Save,
    Copy,
    Paste,
    SelectAll,
    Tick,
    ClearState,
    ResetWorkspace
}

