
#[derive(Clone,Debug)]
pub enum PlotterMessage{
    Clear,
    ClearPixels,
    PlotPixel(usize,usize,Vec<usize>),
    TogglePixelByName(Vec<usize>),
    //LazySelectData(usize,usize),
    PlotXClicked(f64),
    TogglePixel(usize, bool),
    SetPointerDisplay(bool),
    SetPixelmapOn(bool),
    //SyncPointer(usize),
    SyncData{start:usize, end:usize, pointer:usize, force_clear:bool},
    SetSafeguardString(String),
    //SafeguardCommit,
    SetLCMode(super::LCMode),
    SetLCMean(bool),
    SetTimeFormat(super::TimeAxisFormat),
    SetLCOnly(bool),
    SetXMin(String),
    SetXMax(String),
    SetYMin(String),
    SetYMax(String),
    SetSizeX(String),
    SetSizeY(String),
    SetThreshold(String),
    SetDiscontinuityThreshold(String),
    //SubmitThreshold,
    SelectByThreshold,
    //SubmitSize,
    SubmitLimits,
    ShowPixelSelector,
    HidePixelSelector,
    TogglePixelSelector,

    SavePlot
}

impl PlotterMessage{
    pub fn toggle_pixel(index:usize)-> impl Fn(bool)->Self{
        move |x| Self::TogglePixel(index,  x)
    }
}
