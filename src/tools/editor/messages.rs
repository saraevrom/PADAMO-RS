
use iced::widget::scrollable;

#[derive(Debug, Clone)]
pub enum EditorCanvasMessage{
    MoveNode{index:usize, position:iced::Point},
    LinkNode{from:usize,output_port:String, to:usize, input_port:String},
    UnlinkOutput{node:usize, output_port:String},
    UnlinkInput{node:usize, input_port:String},
    #[allow(dead_code)]
    Unselect,
    SquareSelect(iced::Point,iced::Point),
    Select(usize),
    SetShift(bool),
    DeleteSelectedNode,
    ConstantEdit(super::nodes::constants::NodeConstantMessage),
    CancelPaste,
    CommitPaste(iced::Point),
    CanvasScroll(scrollable::Viewport),
}

#[derive(Debug, Clone)]
pub enum EditorMessage{
    CanvasMessage(EditorCanvasMessage),
    //TreeSplitPositionSet(u16),
    PaneDrag(iced::widget::pane_grid::DragEvent),
    PaneResize(iced::widget::pane_grid::ResizeEvent),
    NodeListClicked(String),
    EditorScroll(scrollable::Viewport),
    CompileGraph,
}

