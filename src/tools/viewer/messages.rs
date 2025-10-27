#[derive(Clone,Debug)]
pub enum ViewerMessage{
    EditForm(padamo_iced_forms::ActionOrUpdate<super::form::ViewerFormMessage>),
    // TogglePixel(Vec<usize>),
    // ResetMask,
    WindowView(super::detector_display::SingleDetectorDisplayMessage),
    TimeLine(super::cross_progress::CrossProgressMessage),
}
