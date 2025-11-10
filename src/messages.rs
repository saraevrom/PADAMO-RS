
use crate::tools::editor::messages::EditorMessage;
use crate::tools::viewer::ViewerMessage;
// use crate::tools::plotter::messages::PlotterMessage;
use crate::tools::plotter_new::messages::NewPlotterMessage;
use crate::tools::trigger::messages::TriggerMessage;
use crate::tools::detectors::messages::DetectorManagerMessage;
use crate::loaded_detectors_storage::LoadedDetectorsMessage;

#[derive(Clone,Debug)]
pub enum PadamoAppMessage{
    Noop,
    TabSelect(usize),
    // FontLoaded(Result<(), font::Error>),
    EditorMessage(EditorMessage),
    ViewerMessage(ViewerMessage),
    // PlotterMessage(PlotterMessage),
    TriggerMessage(TriggerMessage),
    NewPlotterMessage(NewPlotterMessage),
    DetectorManagerMessage(DetectorManagerMessage),
    DetectorUpdate,
    LoadedDetectorsMessage(LoadedDetectorsMessage),
    SetEditLoadedDetectors(bool),
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

