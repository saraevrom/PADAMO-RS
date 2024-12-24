use padamo_iced_forms::ActionOrUpdate;

use super::form::PlotterFormMessage;


#[derive(Clone,Debug)]
pub enum PlotterMessage{
    PlotPixel(usize,usize,Vec<usize>),
    TogglePixelByName(Vec<usize>),
    //LazySelectData(usize,usize),
    PlotXClicked(f64),
    TogglePixel(usize, bool),
    //SyncPointer(usize),
    SyncData{start:usize, end:usize, pointer:usize, force_clear:bool},
    //SafeguardCommit,
    SetXMin(String),
    SetXMax(String),
    SetYMin(String),
    SetYMax(String),
    //SubmitThreshold,
    //SubmitSize,
    SubmitLimits,
    HidePixelSelector,
    FormMessage(ActionOrUpdate<PlotterFormMessage>),


}

impl PlotterMessage{
    pub fn toggle_pixel(index:usize)-> impl Fn(bool)->Self{
        move |x| Self::TogglePixel(index,  x)
    }
}
