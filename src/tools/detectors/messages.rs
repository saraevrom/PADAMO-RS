
#[derive(Clone,Debug)]
pub enum DetectorManagerMessage{
    EditorActionPerformed(iced::widget::text_editor::Action),
    //SetSplitPosition(u16),
    PaneDrag(iced::widget::pane_grid::DragEvent),
    PaneResize(iced::widget::pane_grid::ResizeEvent),
    Rebuild,
    Export,
}
