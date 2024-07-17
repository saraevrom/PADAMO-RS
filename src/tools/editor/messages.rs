use iced::widget::scrollable;

#[derive(Debug, Clone)]
pub enum EditorCanvasMessage{
    MoveNode{index:usize, position:iced::Point},
    LinkNode{from:usize,output_port:String, to:usize, input_port:String},
    UnlinkOutput{node:usize, output_port:String},
    UnlinkInput{node:usize, input_port:String},
    Unselect,
    Select(usize),
    DeleteSelectedNode,
    ConstantEdit(super::nodes::constants::NodeConstantMessage),
    //TreeSplitPositionSet(u16)
}

#[derive(Debug, Clone)]
pub enum EditorMessage{
    CanvasMessage(EditorCanvasMessage),
    TreeSplitPositionSet(u16),
    NodeListClicked(String),
    EditorScroll(scrollable::Viewport),
}

