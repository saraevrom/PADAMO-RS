pub mod editor_program;
pub mod nodes;
use std::collections::HashMap;
use std::io::Read;
use std::rc::Rc;
use std::str::FromStr;
use crate::application::PadamoState;
//use crate::custom_widgets::treeview::TreeView;
use crate::messages::PadamoAppMessage;

use crate::custom_widgets::treeview::Tree;
use crate::tools::editor::nodes::{GraphNodeCloneBuffer, GraphNodeStorage};

use super::PadamoTool;
use abi_stable::traits::IntoOwned;
use iced::Length;
use iced::widget::scrollable;
use once_cell::sync::Lazy;
pub mod messages;

static SCROLLABLE_ID: Lazy<scrollable::Id> = Lazy::new(scrollable::Id::unique);


pub struct PadamoEditor{
    state: editor_program::EditorState,
    tree: Tree<String>,
    hor_divider_position: u16,
    current_scroll_offset: scrollable::RelativeOffset,
}

impl PadamoEditor{
    pub fn new(tree:Tree<String>)->Self{

        //println!("{:?}",tree);
        Self{state: editor_program::EditorState::new(), tree, hor_divider_position:200, current_scroll_offset: scrollable::RelativeOffset::START}
    }

    fn run(&self,padamo:&mut PadamoState){
        let mut x_mut = &mut padamo.compute_graph;
        padamo.nodes.make_compute_graph(&mut x_mut, &self.state.nodes);
        if let Err(err) = x_mut.execute(padamo.current_seed.parsed_value){
            padamo.show_error(format!("Execution error: {}",err));
            //println!("Execution error: {}",err);
        }
        else{
            println!("Execution success");
        }
    }
}

impl PadamoTool for PadamoEditor{
    fn view<'a>(&'a self)->iced::Element<'a, PadamoAppMessage> {
        let first:iced::Element<'_,PadamoAppMessage> = scrollable(
                iced::Element::new(self.tree.view(Some(|x| PadamoAppMessage::EditorMessage(messages::EditorMessage::NodeListClicked(x))))),
            ).id(SCROLLABLE_ID.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .direction(scrollable::Direction::Vertical(scrollable::Properties::new().width(10).alignment(scrollable::Alignment::Start)))
            .on_scroll(|x| PadamoAppMessage::EditorMessage(messages::EditorMessage::EditorScroll(x))).into()
            ;

        let second:iced::Element<'_,PadamoAppMessage> = self.state.view(self.hor_divider_position).map(messages::EditorMessage::CanvasMessage).map(PadamoAppMessage::EditorMessage);

        iced_aw::Split::new(
            first,second,
            Some(self.hor_divider_position),
            iced_aw::split::Axis::Vertical,
            PadamoAppMessage::editor_split_position()
        ).into()
        // iced::widget::row!(
        //     iced::Element::new(self.tree.view(Some(crate::messages::PadamoAppMessage::NodeListClicked))),
        //     self.state.view().map(crate::messages::PadamoAppMessage::EditorMessage)
        // ).into()
    }
    fn tab_name(&self)->String {
        "Editor".into()
    }

    fn update(&mut self, msg: Rc<crate::messages::PadamoAppMessage>, padamo:crate::application::PadamoStateRef){
        match &*msg{
            crate::messages::PadamoAppMessage::EditorMessage(emsg) =>{
                match emsg {
                    messages::EditorMessage::CanvasMessage(msg) => {
                        self.state.handle_message(msg)

                    },
                    messages::EditorMessage::TreeSplitPositionSet(pos)=>{self.hor_divider_position = *pos},
                    messages::EditorMessage::NodeListClicked(identifier)=>{
                        // let path = p.join("/");
                        let node = padamo.nodes.create_calculation_node(identifier.into_owned());

                        if let Some(n) = node{
                            let mut storage = GraphNodeStorage::new();
                            let offset = iced::Point::new(n.size.width/2.0, n.size.height/2.0);
                            storage.insert_node(n);
                            let buffer = GraphNodeCloneBuffer{storage,offset, connections:HashMap::new()};
                            *self.state.pending_paste.borrow_mut() = Some(Rc::new(buffer));
                            //self.state.nodes.insert_node(n);
                        }
                        println!("Clicked node {:?}", identifier);
                    },
                    messages::EditorMessage::EditorScroll(view) => {self.current_scroll_offset = view.relative_offset()},
                }
            },
            crate::messages::PadamoAppMessage::Run=>{
                self.run(padamo);
            },
            crate::messages::PadamoAppMessage::RerollRun=>{
                padamo.reroll();
                self.run(padamo);
            },
            _=>()
        }

    }
    fn context_update(&mut self, msg: Rc<crate::messages::PadamoAppMessage>, padamo:crate::application::PadamoStateRef) {
        match msg.as_ref() {
            crate::messages::PadamoAppMessage::Save =>{
                if let Some(file_path) = padamo.workspace.workspace("graphs-rs").save_dialog(vec![("Padamo RS compute graph",vec!["json"])]){
                    let jsd = self.state.nodes.serialize();
                    if let Ok(s) = serde_json::to_string_pretty(&jsd){
                        if let Ok(_) = std::fs::write(file_path, s){
                            println!("Wrote file");
                        }
                    }
                }
            },
            crate::messages::PadamoAppMessage::Open =>{
                if let Some(file_path) = padamo.workspace.workspace("graphs-rs").open_dialog(vec![("Padamo RS compute graph",vec!["json"])]){
                    if let Ok(mut f) = std::fs::File::open(file_path){
                        let mut buf:String = String::new();
                        if let Ok(_) = f.read_to_string(&mut buf){
                            if let Ok(jsd) = serde_json::Value::from_str(&buf){
                                if let Err(e) = self.state.nodes.deserialize(&padamo.nodes, jsd){
                                    padamo.show_error(format!("{}",e));
                                    self.state.nodes.clear();
                                }
                            }
                        }
                    }
                }
            },
            crate::messages::PadamoAppMessage::Copy => {
                self.state.copy_buffer();
            }
            crate::messages::PadamoAppMessage::Paste=>{
                self.state.request_paste();
            }
            crate::messages::PadamoAppMessage::SelectAll=>{
                self.state.nodes.select_all();
            }
            _=>(),
        }

    }

}

