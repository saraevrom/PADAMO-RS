use std::borrow::BorrowMut;
use std::cell::RefCell;

use iced::mouse;
use iced::widget::canvas::event::{self, Event};
use iced::widget::canvas::{self, Geometry, Path};
use iced::{Length, Point, Rectangle, Renderer, Theme};
use iced::widget::scrollable;

pub use super::messages::EditorCanvasMessage;

use super::nodes::constants::{NodeConstantBuffer, NodeConstantMessage};



pub struct EditorState{
    pub nodes: super::nodes::GraphNodeStorage,
    pub copied_data: Option<std::rc::Rc<super::nodes::GraphNodeCloneBuffer>>,
    pub pending_paste:RefCell<Option<std::rc::Rc<super::nodes::GraphNodeCloneBuffer>>>,
    pub divider_position: Option<u16>,
    pub scroll_offset: scrollable::AbsoluteOffset
}


impl EditorState{
    pub fn new()->Self{
        let nodes = super::nodes::GraphNodeStorage::new();
        Self { nodes, copied_data:None, pending_paste:RefCell::new(None), divider_position:None, scroll_offset:scrollable::AbsoluteOffset{x:0.0, y:0.}}
    }

    pub fn request_paste(&mut self){
        *self.pending_paste.borrow_mut() = self.copied_data.clone();
    }

    pub fn copy_buffer(&mut self){
        if let Some(cop) = self.nodes.clone_selection(){
            self.copied_data = Some(std::rc::Rc::new(cop));
        }
        else{
            self.copied_data = None;
        }

    }

    pub fn paste_buffer(&mut self, position:iced::Point){
        if let Some(buf) = &self.copied_data{
            self.nodes.instantiate(&buf, position);
        }
    }

    pub fn view(&self)->(iced::Element<'_, EditorCanvasMessage>,iced::Element<'_, EditorCanvasMessage>) {

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

                let mut constant_row:iced::widget::Column<'_,super::nodes::constants::NodeConstantMessage> = iced::widget::Column::new().padding(10);
                constant_row = constant_row.push(iced::widget::Text::new(format!("{}:",&c.display_name)));

                let external_toggle:iced::widget::Checkbox<'_,super::nodes::constants::NodeConstantMessage> = iced::widget::Checkbox::new("External", c.use_external).on_toggle(NodeConstantMessage::external_toggle(key.into())).into();
                constant_row = constant_row.push(external_toggle);

                if !c.use_external{
                    let field:iced::Element<'_,super::nodes::constants::NodeConstantMessage> = match &c.buffer{
                        NodeConstantBuffer::Check(x) => {
                            iced::widget::Checkbox::new("Value", *x).on_toggle(NodeConstantMessage::check(key.into())).into()
                        },
                        NodeConstantBuffer::Text(x) => {
                            let mut label = iced::widget::Text::new("Value");
                            if !c.ok{
                                label = label.style(|_| iced::widget::text::Style{color:Some(iced::Color::new(1.0, 0.0, 0.0, 1.0))});
                            }
                            let editor = iced::widget::TextInput::new("", x).on_input(NodeConstantMessage::text(key.into()));
                            iced::widget::row!(
                                label,
                                editor
                            ).into()
                        },
                    };
                    constant_row = constant_row.push(field);
                }

                constcol = constcol.push(constant_row);
            }
        }

        // if let Some(node) = self.nodes.view_selected_node(){
        //     let node_view = node.borrow();
        //     for (key,c) in node_view.constants.constants.iter(){
        //         constcol = constcol.push(c.view_editor(key))
        //     }
        // }
        let constcol_container = iced::widget::Container::new(constcol);
        let constcol_elem:iced::Element<'_,super::nodes::constants::NodeConstantMessage> = constcol_container.into();

        // iced_aw::Split::new(
        //     canv,
        //     constcol_elem.map(EditorMessage::ConstantEdit),
        //     Some(self.canvas_split_postition),
        //     iced_aw::split::Axis::Vertical,
        //     EditorMessage::ConstantSplitPositionSet
        // ).into()

        let scrollbar = scrollable::Scrollbar::new()
                        .scroller_width(20);

        let first_part = iced::widget::scrollable(canv)
                .width(Length::Fill)
                .height(Length::Fill)
                .direction(scrollable::Direction::Both{
                    vertical: scrollbar,
                    horizontal: scrollbar,
                })
                .on_scroll(EditorCanvasMessage::CanvasScroll);
        let second_part = iced::widget::scrollable(constcol_elem.map(EditorCanvasMessage::ConstantEdit))
                .width(300)
                .height(Length::Fill);
        // iced::widget::row!{
        //     first_part,second_part
        // }.into()
        (first_part.into(), second_part.into())
    }

    pub fn draw(&self,frame: &mut canvas::Frame){
        self.nodes.draw(frame);
    }

    pub fn handle_message(&mut self,msg:&EditorCanvasMessage){
        match msg{
            EditorCanvasMessage::CancelPaste=>{
                *self.pending_paste.borrow_mut() = None;
                println!("Paste cancelled");
            },
            EditorCanvasMessage::CommitPaste(point) =>{
                let mut newstate = None;

                if let Some(buf) = self.pending_paste.take(){
                    if self.nodes.shift_mod{
                        if let Some(storage)= buf.clone_whole_for_repeating_copy(){
                            newstate = Some(std::rc::Rc::new(storage));
                        }
                    }
                    self.nodes.instantiate(buf.as_ref(), *point);
                }
                if newstate.is_some(){
                    *self.pending_paste.borrow_mut() = newstate;
                    println!("Starting new paste");
                }
                //self.paste_buffer(*point);
                println!("Pasted");
            },
            EditorCanvasMessage::CanvasScroll(v)=>{
                self.scroll_offset = v.absolute_offset();
            }
            _=>{
                self.nodes.handle_message(msg)
            }
        }
    }
}


#[derive(Debug, Clone)]
pub enum EditorProgramState{
    Idle,
    Dragging{
        index:usize,
        start_position:iced::Point,
        cursor_start_position:Point,
        can_delete:bool,
        size:iced::Size
    },
    Linking{
        from:Option<(usize, iced::Point, String)>,
        to:Option<(usize, iced::Point, String)>
    },
    Selecting{
        start_position:iced::Point,
    },
    Inserting{
        buffer: std::rc::Rc<super::nodes::GraphNodeCloneBuffer>
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

    pub fn deletion_box(&self)->iced::Rectangle{
        let tl = iced::Point::new(self.editor_state.scroll_offset.x, self.editor_state.scroll_offset.y);
        let size = iced::Size::new(100., 100.);
        iced::Rectangle::new(tl,size)
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
        if let EditorProgramState::Dragging { index:_, start_position:_, cursor_start_position:_, size:_ , can_delete}=state{
            //let deletion_ghost = Path::rectangle(iced::Point::new(self.editor_state.scroll_offset.x, self.editor_state.scroll_offset.y),iced::Size::new(100., 100.));
            let rect= self.deletion_box();
            let position = rect.position();
            let rect = Path::rectangle(rect.position(),rect.size());
            if *can_delete{
                let label = iced::widget::canvas::Text{content:"Move here\nto delete".into(), position, ..Default::default()};
                frame.fill(&rect, iced::Color::new(0.8, 0.4, 0.4, 1.0));
                frame.fill_text(label);
            }
            else{
                let label = iced::widget::canvas::Text{content:"Deletion safe".into(), position, ..Default::default()};
                frame.fill(&rect, iced::Color::new(0.8, 0.8, 0.8, 1.0));
                frame.fill_text(label);
            }
        }


        self.editor_state.draw(&mut frame);
        if let Some(curpos) = cursor.position(){
            let curpos = iced::Point::new(curpos.x-bounds.x,curpos.y-bounds.y);
            match state{
                EditorProgramState::Idle=>(),
                EditorProgramState::Dragging { index: _, start_position,cursor_start_position, size, can_delete:_ }=>{
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
                EditorProgramState::Inserting { buffer }=>{
                    for node in buffer.storage.nodes.iter(){
                        let ghost_pos = node.borrow().position;
                        let ghost_size = node.borrow().size;
                        let ghost = Path::rectangle(ghost_pos+(curpos-buffer.offset),ghost_size);
                        frame.fill(&ghost, iced::Color::new(0.9, 0.9, 0.9, 1.0));
                    }
                }
            }

        }

        vec![frame.into_geometry()]
    }


    fn update(&self, state: &mut Self::State, event: Event,
            bounds: Rectangle,
            cursor: mouse::Cursor,
        )->(event::Status, Option<EditorCanvasMessage>){


            if let EditorProgramState::Inserting{buffer:_} = state{
                if self.editor_state.pending_paste.borrow().is_none(){
                    *state = EditorProgramState::Idle;
                    println!("Cancelled insert mode");
                }
            }

            if let Some(buffer) = self.editor_state.pending_paste.borrow().as_ref(){
                *state = EditorProgramState::Inserting { buffer:buffer.clone()}
            }

            let mut msg:Option<EditorCanvasMessage> = None;
            if let Some(curpos) = cursor.position(){
                // if !bounds.contains(curpos){
                //     return (event::Status::Ignored, None);
                // }
                let curpos_origin = curpos;
                let curpos = iced::Point::new(curpos.x-bounds.x,curpos.y-bounds.y);

                match event{

                    // Event::Mouse(iced::mouse::Event::CursorLeft)=>{
                    //     println!("Cursor left the area");
                    // },
                    Event::Mouse(iced::mouse::Event::CursorMoved { position })=>{
                        if let EditorProgramState::Dragging { index:_, start_position:_, cursor_start_position:_, can_delete, size:_ } = state{
                            if ! *can_delete{
                                let deletion_rect = self.deletion_box();
                                if !deletion_rect.contains(curpos){
                                    *can_delete = true;
                                }
                            }
                        }
                    }

                    Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left))=>{
                        if let Some((i,pos,size,mouse_status)) = self.editor_state.nodes.get_node_data(curpos){
                            msg = Some(EditorCanvasMessage::Select(i));

                            match mouse_status{
                                super::nodes::NodeMouseHit::MainRect=>{
                                    let deletion_rect = self.deletion_box();
                                    let delete_enable = !deletion_rect.contains(curpos);
                                    *state = EditorProgramState::Dragging { index: i, start_position: pos, cursor_start_position:curpos, size, can_delete:delete_enable };
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

                            if let EditorProgramState::Inserting { buffer:_ } = state{
                                *state = EditorProgramState::Idle;
                                msg = Some(EditorCanvasMessage::CommitPaste(curpos));
                                println!("Commiting paste...");
                            }
                            else{
                                *state = EditorProgramState::Selecting { start_position: curpos };
                            }
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
                        else{
                            if let EditorProgramState::Inserting { buffer:_ } = state{
                                *state = EditorProgramState::Idle;
                                msg = Some(EditorCanvasMessage::CancelPaste);
                                println!("Commiting paste...");
                            }
                        }
                    },
                    Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left))=>{
                        if let EditorProgramState::Dragging { index, start_position,cursor_start_position, size: _, can_delete } = state{
                            let delete_bounds = self.deletion_box();
                            if delete_bounds.contains(curpos) && *can_delete{
                                println!("Released LMB inside deleter");
                                msg = Some(EditorCanvasMessage::DeleteSelectedNode)
                            }
                            else{
                                msg = Some(EditorCanvasMessage::MoveNode { index:*index, position: *start_position+(curpos-*cursor_start_position) });
                            }

                            *state = EditorProgramState::Idle;
                        }
                        else if let EditorProgramState::Selecting { start_position } = state{
                            msg = Some(EditorCanvasMessage::SquareSelect(*start_position, curpos));
                            *state = EditorProgramState::Idle
                        }
                    },

                    Event::Keyboard(iced::keyboard::Event::KeyPressed { key:iced::keyboard::Key::Named(pressed_key), ..})=>{
                        match pressed_key{
                            // This code turned out to be evil
                            // iced::keyboard::key::Named::Delete=>{
                            //     msg = Some(EditorCanvasMessage::DeleteSelectedNode)
                            // }
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
