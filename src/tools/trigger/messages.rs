use padamo_iced_forms::ActionOrUpdate;

use super::TriggerSettingsFormMessage;

#[derive(Clone,Debug)]
pub enum TriggerMessage{
    ChooseTrigger,
    CancelChoseTrigger,
    ConfirmTrigger,
    SelectionMessage(SelectionMessage),
    SettingsMessage(ActionOrUpdate<TriggerSettingsFormMessage>),
    // #[allow(dead_code)] // (usize, String for iced_aw compat)
    // SelectPositive(usize, String),
    // #[allow(dead_code)] // (usize, String for iced_aw compat)
    // SelectNegative(usize, String),
    #[allow(dead_code)] // (usize, String for iced_aw compat)
    SelectEvent(usize, String),
    ExamineEvent,
    Stop,
    PlotZoomMessage(crate::transform_widget::TransformMessage),
    Export,
    ExportStop,
}

#[derive(Clone,Debug)]
pub enum SelectionMessage{
    SetStart(String),
    SetEnd(String),
    //CommitInterval,
    #[allow(dead_code)] // (usize, String for iced_aw compat)
    IntervalSelected(usize,String),
}
