use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs;
use std::rc::Rc;

use iced::{ Application, Theme, Command};
use iced::widget::{row, button};
use crate::messages::PadamoAppMessage;
use crate::nodes_interconnect::NodesRegistry;
use crate::tools::{self as ctools, PadamoTool};
use crate::builtin_nodes;

use iced_aw::{menu_bar, menu_items, menu, Tabs};
use iced_aw::menu::{Menu,Item, MenuBar, StyleSheet};
use iced_aw::modal;
use crate::popup_message::MessageList;
use crate::popup_message::PadamoPopupMessageType;
// use iced_aw::{menu_bar, menu_items};

use iced_aw::style::MenuBarStyle;
use std::path::Path;
use padamo_iced_forms::double_entry_state::EntryState;

use rand::prelude::*;


fn menu_button(action:&str, msg:PadamoAppMessage)->iced::widget::Button<'_,PadamoAppMessage>{
    button(action).on_press(msg).width(iced::Length::Fill)
}

fn title_menu_button(action:&str)->iced::widget::Button<'_,PadamoAppMessage>{
    button(action).on_press(PadamoAppMessage::Noop)
}




pub struct Padamo{
    tools: Vec<Box<dyn crate::tools::PadamoTool>>,
    pub state:PadamoState,
}

pub struct PadamoState{
    pub nodes: crate::nodes_interconnect::NodesRegistry,
    pub compute_graph: padamo_api::calculation_nodes::graph::CalculationSequenceStorage,
    pub workspace:padamo_workspace::PadamoWorkspace,
    pub add_delay_ms:u64,
    pub current_page:usize,
    pub current_seed:EntryState<u64>,
    popup_messages:MessageList,
}

pub type PadamoStateRef<'a> = &'a mut PadamoState;

/*
#[derive(Clone)]
pub struct PadamoStateRef<'a>{
    pub appref:&'a Padamo,
    // pub nodes: Rc<crate::nodes_interconnect::NodesRegistry>,
    // pub compute_graph: Rc<RefCell<padamo_api::calculation_nodes::graph::CalculationSequenceStorage>>,
    // pub workspace:Rc<RefCell<padamo_workspace::PadamoWorkspace>>,
    // pub add_delay_ms:Rc<RefCell<u64>>,
    //pub popup_messages:Rc<RefCell<MessageList>>,
}*/


impl PadamoState{
    pub fn show_info<T:Into<String>>(&mut self, msg:T){
        self.popup_messages.add_message(msg.into(), PadamoPopupMessageType::Info);
    }

    pub fn show_warning<T:Into<String>>(&mut self, msg:T){
        self.popup_messages.add_message(msg.into(), PadamoPopupMessageType::Warning);
    }


    pub fn show_error<T:Into<String>>(&mut self, msg:T){
        self.popup_messages.add_message(msg.into(), PadamoPopupMessageType::Error);
    }

    pub fn reroll(&mut self){
        let mut rng = rand::thread_rng();
        self.current_seed.set_value(rng.next_u64());
    }
}

impl Padamo{
    pub fn generate_state<'a>(&'a mut self)->PadamoStateRef<'a>{
        //PadamoStateRef { appref: self }
        &mut self.state
        // PadamoStateRef {
        //     nodes: self.nodes.clone(),
        //     compute_graph: self.compute_graph.clone(),
        //     workspace:self.workspace.clone(),
        //     add_delay_ms:self.add_delay_ms.clone(),
        //     //popup_messages:self.popup_messages.clone(),
        // }
    }


    pub fn update_tools(&mut self, msg:Rc<PadamoAppMessage>){
        let state = &mut self.state;
        for tool in self.tools.iter_mut(){
            tool.update(msg.clone(), state);
        }
    }

    pub fn late_update_tools(&mut self, msg:Rc<PadamoAppMessage>)->Vec<Rc<PadamoAppMessage>>{
        let state = &mut self.state;
        let tools = &mut self.tools;
        let mut res = Vec::with_capacity(tools.len());
        for tool in tools.iter_mut(){

            if let Some(v) = tool.late_update(msg.clone(), state){
                res.push(Rc::new(v));
            }
        }
        res
    }

    pub fn update_context_tool(&mut self, msg:Rc<PadamoAppMessage>){
        let state = &mut self.state;
        let tools = &mut self.tools;
        let current_page = state.current_page;
        let tool = &mut tools[current_page];
        //println!("Context tool {}",tool.0);
        tool.context_update(msg.clone(), state);
    }

    pub fn update_tools_sequence(&mut self, reference:Rc<PadamoAppMessage>)->Vec<Rc<PadamoAppMessage>>{
        self.update_tools(reference.clone());
        self.update_context_tool(reference.clone());
        self.late_update_tools(reference)
    }


    fn update_tools_loop(&mut self, msg:Rc<PadamoAppMessage>){
        let mut references:VecDeque<Rc<PadamoAppMessage>> = VecDeque::new();
        references.push_back(msg);
        while references.len()>0{
            while let Some(v) = references.pop_front(){
                references.extend(self.update_tools_sequence(v));
            }
        }
    }
}


fn register_nodes(nodes:&mut NodesRegistry, seekdir:&Path, look_in_directories:bool){
    println!("Seeking for plugins in {}", seekdir.to_str().unwrap());
    let paths = fs::read_dir(seekdir).unwrap();
    for path in paths{
        if let Ok(p_res) = path{
            let p = p_res.path();
            if p.is_dir() && look_in_directories{
                println!("Looking into dir: {:?}",p);

                //NO FULL RECURSIVE SEARCH. Only first layer.
                register_nodes(nodes, &p, false);
            }
            else if p.is_file(){
                if let Err(e) = nodes.load_lib(p.as_path()){
                    println!("Error reading library: {}",e);
                }
            }
            else {
                println!("Skipped: {:?}",p);
            }
        }
    }
}

impl Application for Padamo{
    type Executor = iced::executor::Default;
    type Message = PadamoAppMessage;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ())->(Self, iced::Command<PadamoAppMessage>){
        let mut nodes = crate::nodes_interconnect::NodesRegistry::new();

        nodes.register_node(builtin_nodes::viewer::LoadedFileNode).unwrap();
        nodes.register_node(builtin_nodes::viewer::ViewerNode).unwrap();
        nodes.register_node(builtin_nodes::viewer::ViewerMaskNode).unwrap();

        let current_exe = std::env::current_exe().unwrap();
        let current_dir = current_exe.parent().unwrap();
        let plugins_dir = current_dir.join("plugins");
        register_nodes(&mut nodes, &plugins_dir, true);
        // println!("Seeking for plugins in {}", plugins_dir.to_str().unwrap());
        // let paths = fs::read_dir(plugins_dir).unwrap();
        // for path in paths{
        //     if let Ok(p_res) = path{
        //         let p = p_res.path();
        //         if p.is_dir(){
        //             println!("Looking into dir: {:?}",p);
        //
        //         }
        //         else if p.is_file(){
        //             if let Err(e) = nodes.load_lib(p.as_path()){
        //                 println!("Error reading library: {}",e);
        //             }
        //         }
        //         else {
        //             println!("Skipped: {:?}",p);
        //         }
        //     }
        // }

        let tools: Vec<Box<dyn crate::tools::PadamoTool>> = vec![
            Box::new(ctools::PadamoViewer::new()),
            Box::new(ctools::PadamoEditor::new(nodes.make_tree())),
            Box::new(ctools::PadamoTrigger::new()),
            Box::new(ctools::Plotter::new()),
            Box::new(ctools::PadamoDetectorManager::new())
        ];

        let mut compute_graph = padamo_api::calculation_nodes::graph::CalculationSequenceStorage::new();

        let det = serde_json::to_string(&padamo_detectors::polygon::DetectorContent::default_vtl()).unwrap();
        compute_graph.environment.0.insert("detector".into(), padamo_api::calculation_nodes::content::Content::String(det.into()));

        let state = PadamoState{
            nodes,
            compute_graph,
            workspace: padamo_workspace::PadamoWorkspace::initialize(),
            add_delay_ms: 0,
            current_page: 0,
            current_seed: EntryState::new(0),
            popup_messages:MessageList::new(),
        };

        (
            Self {
                //current_page: 0,
                tools,
                state,
                //popup_messages:Rc::new(RefCell::new(MessageList::new())),
            },
            iced::font::load(iced_aw::BOOTSTRAP_FONT_BYTES).map(PadamoAppMessage::FontLoaded)
        )
    }


    fn update(&mut self, msg: PadamoAppMessage)->Command<PadamoAppMessage>{
        match msg{
            PadamoAppMessage::TabSelect(tab)=> {
                //println!("Tab select {}", tab);
                self.state.current_page = tab
            },
            PadamoAppMessage::ResetWorkspace=> {self.state.workspace.recreate()},

            PadamoAppMessage::PopupMessageClick=>{
                self.state.popup_messages.pop_oldest_message();
            },
            PadamoAppMessage::SetSeed(seed)=>{
                self.state.current_seed.set_string(seed);
            },
            PadamoAppMessage::ChooseDetector=>{
                if let Some(path) = self.state.workspace.workspace("detectors").open_dialog(vec![("Detector",vec!["json"])]){
                    let s = match std::fs::read_to_string(path) {
                        Ok(v)=>{v},
                        Err(e)=> {self.state.show_error(format!("{:?}",e)); return Command::none();}
                    };
                    let detector = serde_json::from_str(&s);
                    let detector = match detector{
                        Ok(v)=>{v},
                        Err(e)=> {self.state.show_error(format!("{:?}",e)); return Command::none();}
                    };
                    self.state.compute_graph.environment.0.insert("detector".into(), padamo_api::calculation_nodes::content::Content::String(s.into()));
                    let msg = PadamoAppMessage::SetDetector(detector);
                    self.update_tools_loop(Rc::new(msg));
                }
            }
            other=>{
                self.update_tools_loop(Rc::new(other));
            }
        };
        Command::none()
    }


    fn title(&self) -> String {
        "Padamo".into()
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let mut vlist = iced::widget::Column::new();
        vlist = vlist.spacing(10);

        let file_menu = Item::with_menu(title_menu_button("File"), Menu::new(vec![
            Item::new(menu_button("Open", PadamoAppMessage::Open)),
            Item::new(menu_button("Save", PadamoAppMessage::Save)),
            Item::new(menu_button("Choose detector", PadamoAppMessage::ChooseDetector)),
        ]).max_width(100.0).offset(0.0).spacing(5.0));

        let edit_menu = Item::with_menu(title_menu_button("Edit"), Menu::new(vec![
            Item::new(menu_button("Copy", PadamoAppMessage::Copy)),
            Item::new(menu_button("Paste", PadamoAppMessage::Paste)),
        ]).max_width(100.0).offset(0.0).spacing(5.0));

        //let v = self.state.current_seed.view_row("Seed", "Seed", )
        let run_menu = Item::with_menu(title_menu_button("Run"), Menu::new(vec![
            Item::new(menu_button("Run", PadamoAppMessage::Run)),
            Item::new(menu_button("Reroll and run", PadamoAppMessage::RerollRun)),
            Item::new(self.state.current_seed.view_row("Seed","0 or maybe 42",PadamoAppMessage::SetSeed))
        ]).max_width(150.0).offset(0.0).spacing(5.0));

        let settings_menu = Item::with_menu(title_menu_button("Settings"), Menu::new(vec![
            Item::new(menu_button("Choose workspace directory", PadamoAppMessage::ResetWorkspace)),
        ]).max_width(200.0).offset(0.0).spacing(5.0));

        let menu_bar = MenuBar::new(vec![file_menu,edit_menu,run_menu,settings_menu])
            .draw_path(iced_aw::menu::DrawPath::Backdrop)
            .style(|theme:&iced::Theme| iced_aw::menu::Appearance{
                    path_border: iced::Border{
                    radius: [6.0; 4].into(),
                    ..Default::default()
                },
                ..theme.appearance(&MenuBarStyle::Default)
            });
        vlist = vlist.push(menu_bar);




        let mut tabs = Tabs::new(PadamoAppMessage::TabSelect)
                    .tab_icon_position(iced_aw::tabs::Position::Bottom);
                    //.on_close(PadamoAppMessage::Noop);

        //let tools = self.tools;
        for (i, tab_obj) in self.tools.iter().enumerate(){
            tabs = tabs.push(i,tab_obj.tab_label(),tab_obj.view());
        }
        tabs = tabs.set_active_tab(&self.state.current_page);

        vlist = vlist.push(tabs);

        //vlist.into()
        let underlay:iced::Element<'_,Self::Message> = vlist.into();
        //let overlay:Option<iced::Element<'_,Self::Message>> = None;
        let overlay:Option<iced::Element<'_,Self::Message>> = if let Some(msg) = self.state.popup_messages.oldest_message(){
            Some(msg.view())
        }
        else{
            None
        };
        let popup_window = modal(underlay,overlay).align_x(iced::alignment::Horizontal::Center);
        popup_window.into()

    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::time::every(std::time::Duration::from_millis(33+self.state.add_delay_ms)).map(|_| {
            PadamoAppMessage::Tick
        })
    }
}
