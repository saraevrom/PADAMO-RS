use padamo_iced_forms::ActionOrUpdate;

use super::TriggerSettingsFormMessage;

#[derive(Clone,Debug)]
pub enum TriggerMessage{
    ChooseTrigger,
    CancelChoseTrigger,
    ConfirmTrigger,
    SelectionMessage(SelectionMessage),
    SettingsMessage(ActionOrUpdate<TriggerSettingsFormMessage>),
    SelectPositive(usize, String),
    SelectNegative(usize, String),
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
    IntervalSelected(usize,String),
}
