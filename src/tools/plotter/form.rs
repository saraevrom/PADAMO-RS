use padamo_iced_forms::{Action, IcedForm, IcedFormBuffer};
use padamo_iced_forms::make_action;

#[derive(Clone,Debug,IcedForm)]
#[spoiler_hidden]
pub struct OutShape{
    #[field_name("Width [pix]")] pub width:u32,
    #[field_name("Height [pix]")] pub height:u32,
}

impl Default for OutShape{
    fn default() -> Self {
        OutShape { width: 1024, height: 768 }
    }
}

#[derive(Clone,Debug,IcedForm)]
#[spoiler_hidden]
pub struct DisplaySettings{
    #[field_name("Display pointer")] pub display_pointer:bool,
    #[field_name("Display channel map")] pub display_channel_map:bool,
    #[field_name("Safeguard")] pub safeguard: usize,
    #[field_name("Discontinuity threshold [steps]")] pub step_threshold:f64,
    #[field_name("Time format")] pub time_format: TimeAxisFormat,
    #[field_name("LC")] pub lc_mode:LCMode,
    #[field_name("Mean")] pub lc_mean:bool,
    #[field_name("Only")] pub lc_only:bool,
}

impl Default for DisplaySettings{
    fn default() -> Self {
        Self {
            display_pointer: true,
            display_channel_map: true,
            safeguard: 30000,
            step_threshold: 1.5,
            time_format:Default::default(),
            lc_mode:Default::default(),
            lc_mean:false,
            lc_only:false
        }
    }
}

#[derive(Clone,Debug,Copy,Eq,PartialEq, IcedForm)]
pub enum TimeAxisFormat{
    #[field_name("Unixtime")] AsIs,
    #[field_name("Frames")]GTU,
    #[field_name("Seconds")] Offset,
    #[field_name("Time")] Time,
}

impl Default for TimeAxisFormat{
    fn default() -> Self {
        Self::AsIs
    }
}

#[derive(Clone,Debug,Copy,Eq,PartialEq, IcedForm)]
pub enum LCMode{
    Off,
    All,
    Selected
}

impl Default for LCMode{
    fn default() -> Self {
        Self::Off
    }
}

#[derive(Clone,Debug,IcedForm)]
#[spoiler_hidden]
pub struct SelectorForm{
    pub threshold:f64,
    #[field_name("Clear pixel selection")] _clear: Action<PlotterActions,PlotterActionClearSelection>,
    #[field_name("Threshold selection")] _tselect: Action<PlotterActions,PlotterActionThresholdSelect>,
    #[field_name("Manual selection")] _mselect: Action<PlotterActions,PlotterActionManualSelect>,
}

impl Default for SelectorForm{
    fn default()->Self{
        Self{
            threshold:1.5,
            _clear:Default::default(),
            _tselect:Default::default(),
            _mselect:Default::default(),
        }
    }
}

#[derive(Clone,Debug)]
pub enum PlotterActions{
    Noop,
    Save,
    ClearSelection,
    ThresholdSelect,
    ManualSelect,
}

impl Default for PlotterActions{
    fn default()->Self{
        PlotterActions::Noop
    }
}

make_action!(PlotterActionNoop,PlotterActions,Noop);
make_action!(PlotterActionSave,PlotterActions,Save);
make_action!(PlotterActionClearSelection,PlotterActions,ClearSelection);
make_action!(PlotterActionThresholdSelect,PlotterActions,ThresholdSelect);
make_action!(PlotterActionManualSelect,PlotterActions,ManualSelect);


#[derive(Clone,Debug,Default,IcedForm)]
pub struct PlotterForm{
    #[field_name("Save")] _save: Action <PlotterActions,PlotterActionSave>,
    #[field_name("Image export settings")] pub output_shape: OutShape,
    #[field_name("Display settings")] pub display_settings: DisplaySettings,
    #[field_name("Selection")] pub selector: SelectorForm,
}
