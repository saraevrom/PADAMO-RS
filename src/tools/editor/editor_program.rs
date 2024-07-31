use std::cell::RefCell;
use std::rc::Weak;

use iced::mouse;
use iced::widget::canvas::event::{self, Event};
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke};
use iced::{Element, Length, Point, Rectangle, Renderer, Theme};
use iced::widget::scrollable::{self, Properties};
use once_cell::sync::Lazy;
pub use super::messages::EditorCanvasMessage;

use super::nodes::constants::{NodeConstantStorage, NodeConstantContent, NodeConstantMessage, NodeConstantMessageContent};


static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);

pub struct EditorState{
    pub nodes: super::nodes::GraphNodeStorage,
}


impl EditorState{
    pub fn new()->Self{
        let nodes = super::nodes::GraphNodeStorage::new();
        Self { nodes }
    }

    pub fn view(&self, offset:u16)->iced::Element<'_, EditorCanvasMessage> {

        let scale = self.nodes.full_size();
        let canv_width = (scale.width+200.0).max(1000.0);
        let canv_height = (scale.height+200.0).max(1000.0);
        let canv:iced::Element<'_, EditorCanvasMessage> = iced::widget::canvas::Canvas::new(EditorProgram::new(&self))
            .width(canv_width)
            .height(canv_height)
            .into();
        let mut constcol:iced::widget::Column<'_,super::nodes::constants::NodeConstantMessage> = iced::widget::Column::new();
        if let Some(x) = self.nodes.view_selected_constants(){
            constcol = constcol.push(iced::widget::Text::new("Constants"));
            // Kicked by borrow checker. Ouch. So I have to make display logic here.
            for (key,c) in x.constants.iter(){
                //let check = NodeConstantMessage::check(key.into());
                let field:iced::Element<'_,super::nodes::constants::NodeConstantMessage> = match &c.buffer{
                    NodeConstantMessageContent::Check(x) => {
                        iced::widget::Checkbox::new(key, *x).on_toggle(NodeConstantMessage::check(key.into())).into()
                    },
                    NodeConstantMessageContent::Text(x) => {
                        let mut label = iced::widget::Text::new(key.clone());
                        if !c.ok{
                            label = label.style(iced::theme::Text::Color(iced::Color::new(1.0, 0.0, 0.0, 1.0)));
                        }
                        let editor = iced::widget::TextInput::new("", x).on_input(NodeConstantMessage::text(key.into()));
                        iced::widget::row!(
                            label,
                            editor
                        ).into()
                    },
                };
                constcol = constcol.push(field);
            }
        }

        // if let Some(node) = self.nodes.view_selected_node(){
        //     let node_view = node.borrow();
        //     for (key,c) in node_view.constants.constants.iter(){
        //         constcol = constcol.push(c.view_editor(key))
        //     }
        // }
        let constcol_container = iced::widget::Container::new(constcol).width(200);
        let constcol_elem:iced::Element<'_,super::nodes::constants::NodeConstantMessage> = constcol_container.into();

        // iced_aw::Split::new(
        //     canv,
        //     constcol_elem.map(EditorMessage::ConstantEdit),
        //     Some(self.canvas_split_postition),
        //     iced_aw::split::Axis::Vertical,
        //     EditorMessage::ConstantSplitPositionSet
        // ).into()
        iced::widget::row!{
            iced::widget::scrollable(canv)
                .width(Length::Fill)
                .height(Length::Fill)
                .direction(scrollable::Direction::Both{
                    vertical: Properties::new()
                        .scroller_width(20),
                    horizontal:
                    Properties::new()
                        .scroller_width(20),
                })
                .id(SCROLLABLE_ID.clone()),
            constcol_elem.map(EditorCanvasMessage::ConstantEdit),
        }.into()
    }

    pub fn draw(&self,frame: &mut canvas::Frame){
        self.nodes.draw(frame);
    }
}


#[derive(Debug, Clone)]
pub enum EditorProgramState{
    Idle,
    Dragging{
        index:usize,
        start_position:iced::Point,
        cursor_start_position:Point,
        size:iced::Size
    },
    Linking{
        from:Option<(usize, iced::Point, String)>,
        to:Option<(usize, iced::Point, String)>
    },
    Selecting{
        start_position:iced::Point,
    }
}

impl Default for EditorProgramState{
    fn default() -> Self {
        Self::Idle
    }
}



pub struct EditorProgram<'a>{
    editor_state:&'a EditorState
}

impl<'a> EditorProgram<'a>{
    pub fn new(editor_state:&'a EditorState) -> Self{
        Self{
            editor_state
        }
    }
}

impl<'a> canvas::Program<EditorCanvasMessage> for EditorProgram<'a>{
    type State = EditorProgramState;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let background = Path::rectangle(Point::new(0., 0.), bounds.size());
        frame.fill(&background, iced::Color::new(0.5, 0.5, 0.5, 1.0));
        self.editor_state.draw(&mut frame);
        if let Some(curpos) = cursor.position(){
            let curpos = iced::Point::new(curpos.x-bounds.x,curpos.y-bounds.y);
            match state{
                EditorProgramState::Idle=>(),
                EditorProgramState::Dragging { index: _, start_position,cursor_start_position, size }=>{
                    let ghost_pos:iced::Point = *start_position+(curpos-*cursor_start_position);
                    let ghost = Path::rectangle(ghost_pos,*size);
                    frame.fill(&ghost, iced::Color::new(0.9, 0.9, 0.9, 1.0));
                }
                EditorProgramState::Linking { from, to }=>{
                    if let Some((_,p,_)) = from {
                        let line = Path::line(*p, curpos);
                        frame.stroke(&line, canvas::stroke::Stroke::default().with_color(iced::Color::BLACK).with_width(2.0));
                    }
                    else if let Some((_,p,_)) = to {
                        let line = Path::line(*p, curpos);
                        frame.stroke(&line, canvas::stroke::Stroke::default().with_color(iced::Color::BLACK).with_width(2.0));
                    }
                }
                EditorProgramState::Selecting { start_position }=>{
                    let tl_x = start_position.x.min(curpos.x);
                    let tl_y = start_position.y.min(curpos.y);
                    let tl = iced::Point::new(tl_x, tl_y);
                    let sx = (start_position.x-curpos.x).abs();
                    let sy = (start_position.y-curpos.y).abs();
                    let size = iced::Size::new(sx, sy);
                    let selector = Path::rectangle(tl, size);
                    frame.stroke(&selector, canvas::stroke::Stroke::default().with_color(iced::Color::BLACK).with_width(2.0))
                }
            }
        }

        vec![frame.into_geometry()]
    }


    fn update(&self, state: &mut Self::State, event: Event,
            bounds: Rectangle,
            cursor: mouse::Cursor,
        )->(event::Status, Option<EditorCanvasMessage>){

            let mut msg:Option<EditorCanvasMessage> = None;
            if let Some(curpos) = cursor.position(){
                if !bounds.contains(curpos){
                    return (event::Status::Ignored, None);
                }
                let curpos = iced::Point::new(curpos.x-bounds.x,curpos.y-bounds.y);

                match event{
                    Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left))=>{
                        if let Some((i,pos,size,mouse_status)) = self.editor_state.nodes.get_node_data(curpos){
                            msg = Some(EditorCanvasMessage::Select(i));
                            match mouse_status{
                                super::nodes::NodeMouseHit::MainRect=>{
                                    *state = EditorProgramState::Dragging { index: i, start_position: pos, cursor_start_position:curpos, size };
                                },
                                super::nodes::NodeMouseHit::Output(port,center)=>{
                                    let linkage = Some((i,center,port));
                                    if let EditorProgramState::Linking{from,to:_} = state{
                                        *from = linkage;
                                    }
                                    else{
                                        *state = EditorProgramState::Linking { from: linkage, to: None };
                                    }
                                },
                                super::nodes::NodeMouseHit::Input(port,center)=>{
                                    let linkage = Some((i,center,port));
                                    if let EditorProgramState::Linking{from:_,to} = state{
                                        *to = linkage;
                                    }
                                    else{
                                        *state = EditorProgramState::Linking { from: None, to: linkage };
                                    }
                                },
                            }
                            if let EditorProgramState::Linking{from:Some(from),to:Some(to)} = state{
                                msg = Some(EditorCanvasMessage::LinkNode { from: from.0, output_port:from.2.clone() , to: to.0, input_port: to.2.clone() });
                                *state = EditorProgramState::Idle;
                            }
                        }
                        else{
                            *state = EditorProgramState::Selecting { start_position: curpos };
                            // msg = Some(EditorCanvasMessage::Unselect);
                            // *state = EditorProgramState::Idle
                        }
                    },
                    Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Right))=>{
                        if let Some((i,pos,size,mouse_status)) = self.editor_state.nodes.get_node_data(curpos){
                            match mouse_status{
                                super::nodes::NodeMouseHit::Input(port,_)=>{
                                    msg = Some(EditorCanvasMessage::UnlinkInput { node: i, input_port: port })
                                },
                                super::nodes::NodeMouseHit::Output(port,_)=>{
                                    msg = Some(EditorCanvasMessage::UnlinkOutput { node: i, output_port: port })
                                },
                                _=>{
                                    return (event::Status::Ignored, None);
                                }
                            }
                        }
                    },
                    Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) | Event::Mouse(iced::mouse::Event::CursorLeft)=>{
                        if let EditorProgramState::Dragging { index, start_position,cursor_start_position, size: _ } = state{
                             msg = Some(EditorCanvasMessage::MoveNode { index:*index, position: *start_position+(curpos-*cursor_start_position) });
                            *state = EditorProgramState::Idle;
                        }
                        else if let EditorProgramState::Selecting { start_position } = state{
                            msg = Some(EditorCanvasMessage::SquareSelect(*start_position, curpos));
                            *state = EditorProgramState::Idle
                        }
                    },

                    Event::Keyboard(iced::keyboard::Event::KeyPressed { key:iced::keyboard::Key::Named(pressed_key), location:_, modifiers:_,text:_ })=>{
                        match pressed_key{
                            iced::keyboard::key::Named::Delete=>{
                                msg = Some(EditorCanvasMessage::DeleteSelectedNode)
                            }
                            iced::keyboard::key::Named::Shift=>{
                                msg = Some(EditorCanvasMessage::SetShift(true))
                            }
                            _=>{}
                        }
                       ;
                    },
                    Event::Keyboard(iced::keyboard::Event::KeyReleased { key:iced::keyboard::Key::Named(pressed_key), location:_, modifiers:_ })=>{
                        match pressed_key{
                            iced::keyboard::key::Named::Shift=>{
                                msg = Some(EditorCanvasMessage::SetShift(false))
                            }
                            _=>{}
                        }
                       ;
                    },
                    _=>{
                        return (event::Status::Ignored, None);
                    },
                }
                return (event::Status::Captured, msg);
            }
            else{
                return (event::Status::Ignored, None);
            }

        }
}
