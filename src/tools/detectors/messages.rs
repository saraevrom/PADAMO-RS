
#[derive(Clone,Debug)]
pub enum DetectorManagerMessage{
    EditorActionPerformed(iced::widget::text_editor::Action),
    SetSplitPosition(u16),
    Rebuild,
    Export,
}
