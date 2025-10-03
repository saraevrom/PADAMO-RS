#[derive(Clone,Debug)]
pub enum ViewerMessage{
    SetViewPosition(usize),
    SetViewPositionUnixTime(f64),
    FocusOn(usize,usize),
    SetStart,
    SetEnd,
    Reset,
    JumpToStart,
    JumpToEnd,
    Stop,
    StepBack,
    StepFwd,
    Forward,
    Backward,
    SetViewPositionText(String),
    SetViewStartText(String),
    SetViewEndText(String),
    EditDatetime(String),
    SubmitTimeline,
    SubmitDatetime,
    //SubmitSettings,
    // EditAnimationSettings(padamo_iced_forms::ActionOrUpdate<super::AnimationParametersMessage>),
    // EditExportSettings(padamo_iced_forms::ActionOrUpdate<super::ExportParametersMessage>),
    EditForm(padamo_iced_forms::ActionOrUpdate<super::form::ViewerFormMessage>),
    // CreateAnimation,
    // StopAnimation,

    // Export,
    // StopExport,
    // SetAutoscale(bool),
    // SetAutostop(bool),
    // SetMinSignal(String),
    // SetMaxSignal(String),
    TogglePixel(Vec<usize>),
    PlotZoomMessage(crate::transform_widget::TransformMessage),
    WindowView(super::detector_display::SingleDetectorDisplayMessage),

    //PopupPlot(usize,usize,Vec<usize>)
}

// impl ViewerMessage{
//     pub fn plot_pixel(start:usize, end:usize)-> impl Fn(Vec<usize>)->Self{
//         move |x| Self::PopupPlot(start, end, x)
//     }
// }
