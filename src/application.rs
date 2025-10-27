use std::collections::VecDeque;
use std::fs;
use std::rc::Rc;

use iced::alignment::Horizontal;
use iced::widget::{button, container};
use iced::Length;
use padamo_detectors::polygon::DetectorContent;
use crate::loaded_detectors_storage::LoadedDetectors;
use crate::messages::PadamoAppMessage;
use crate::nodes_interconnect::NodesRegistry;
use crate::tools::{self as ctools};
use crate::{builtin_nodes, loaded_detectors_storage};

use iced_aw::Tabs;
use iced_aw::menu::{primary, Item, Menu, MenuBar};
use crate::popup_message::MessageList;
use crate::popup_message::PadamoPopupMessageType;
// use iced_aw::{menu_bar, menu_items};

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
    pub persistent_state:padamo_state_persistence::PersistentState,
    popup_messages:MessageList,
    pub detectors:crate::loaded_detectors_storage::LoadedDetectors,
    pub is_editing_detectors: bool,
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

    pub fn save_detectors(&self){
        self.persistent_state.serialize("detectors", &self.detectors);
    }
}

impl Padamo{


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

    // fn set_detector(&mut self, detector:padamo_detectors::polygon::DetectorContent){
    //     self.state.compute_graph.environment.0.insert("detector".into(), padamo_api::calculation_nodes::content::Content::String(s.into()));
    //     let msg = PadamoAppMessage::SetDetector(detector);
    //     self.update_tools_loop(Rc::new(msg));
    // }
    fn set_detector(&mut self, s:String, save_state:bool)->Option<DetectorContent>{
        let detector = serde_json::from_str(&s);

        let detector = match detector{
            Ok(v)=>{v},
            Err(e)=> {self.state.show_error(format!("{:?}",e)); return None;}
        };
        if save_state{
            self.state.persistent_state.write("detector", &s);
        }
        self.state.compute_graph.environment.0.insert("detector".into(), padamo_api::calculation_nodes::content::Content::String(s.into()));
        Some(detector)
    }

    fn try_load_detector(&mut self){
        if let Some(s) = self.state.persistent_state.read("detector"){
            println!("Loading persistent detector");
            // self.set_detector(s, false);
            if let Some(detector) = self.set_detector(s, false){
                let msg = PadamoAppMessage::SetDetector(detector);
                self.update_tools_sequence(Rc::new(msg));
            }
        }
        if let Some(s) = self.state.persistent_state.read("detectors"){
            println!("Loading persistent detector array");
            match serde_json::from_str::<LoadedDetectors>(&s){
                 Ok(v)=>{
                     self.state.detectors = v;
                     let msg = PadamoAppMessage::DetectorUpdate;
                     self.update_tools_sequence(Rc::new(msg));
                 },
                 Err(e)=>self.state.show_error(format!("{}",e)),
            }
            // if let Some(detectors) = self.set_detector(s, false){
            //     //let msg = PadamoAppMessage::SetDetector(detector);
            //     //self.update_tools_sequence(Rc::new(msg));
            // }
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

type Message = PadamoAppMessage;

impl Padamo{
    // type Executor = iced::executor::Default;
    // type Message = PadamoAppMessage;
    // type Theme = Theme;
    // type Flags = ();

    fn new()->Self{
        let mut nodes = crate::nodes_interconnect::NodesRegistry::new();

        nodes.register_node(builtin_nodes::viewer::LoadedFileNode).unwrap();
        nodes.register_node(builtin_nodes::viewer::ViewerNode).unwrap();
        nodes.register_node(builtin_nodes::viewer::AuxViewerNode).unwrap();
        nodes.register_node(builtin_nodes::viewer::AuxViewerMaskNode).unwrap();

        nodes.register_node(builtin_nodes::viewer::ViewerMaskNode).unwrap();
        nodes.register_node(padamo_api::calculation_nodes::full_reader::FullReaderNode).unwrap();

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

        let mut tools: Vec<Box<dyn crate::tools::PadamoTool>> = Vec::new();

        #[cfg(feature = "viewer")]
        {
            tools.push(Box::new(ctools::PadamoViewer::new()));
            println!("Viewer present");
        }

        #[cfg(feature = "editor")]
        {
            tools.push(Box::new(ctools::PadamoEditor::new(nodes.make_tree())));
            println!("Editor present");
        }

        #[cfg(feature = "trigger")]
        {
            tools.push(Box::new(ctools::PadamoTrigger::new()));
            println!("Editor present");
        }

        #[cfg(feature = "plotter")]
        {
            //tools.push(Box::new(ctools::Plotter::new()));
            tools.push(Box::new(ctools::PlotterNew::new()));
            println!("Plotter present");
        }

        #[cfg(feature = "detector_manager")]
        {
            tools.push(Box::new(ctools::PadamoDetectorManager::new()));
            println!("Detectors present");
        }


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
            persistent_state: Default::default(),
            detectors: loaded_detectors_storage::LoadedDetectors::new(),
            is_editing_detectors: false,
        };


        let mut res = Self {
            //current_page: 0,
            tools,
            state,
            //popup_messages:Rc::new(RefCell::new(MessageList::new())),
        };
            //,
            // iced::font::load(iced_aw::BOOTSTRAP_FONT_BYTES).map(PadamoAppMessage::FontLoaded)

        res.try_load_detector();
        res.initialize_tools();
        res
    }

    pub fn initialize_tools(&mut self){
        for tool in self.tools.iter_mut(){
            tool.initialize(&mut self.state);
        }
    }

    pub fn update(&mut self, msg: PadamoAppMessage){
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
            PadamoAppMessage::LoadedDetectorsMessage(msg)=>{
                if let Err(e) = self.state.detectors.process_message(&self.state.workspace, msg){
                    self.state.show_error(format!("{}",e));
                }
                else{
                    self.state.save_detectors();
                }
            },
            PadamoAppMessage::SetEditLoadedDetectors(v)=>self.state.is_editing_detectors = v,
            PadamoAppMessage::ChooseDetector=>{
                if let Some(path) = self.state.workspace.workspace("detectors").open_dialog(vec![("Detector",vec!["json"])]){
                    let s = match std::fs::read_to_string(path) {
                        Ok(v)=>{v},
                        Err(e)=> {self.state.show_error(format!("{:?}",e)); return;}
                    };
                    if let Some(detector) = self.set_detector(s, true){
                        let msg = PadamoAppMessage::SetDetector(detector);
                        self.update_tools_loop(Rc::new(msg));
                    }
                    // let detector = serde_json::from_str(&s);
                    //
                    // let detector = match detector{
                    //     Ok(v)=>{v},
                    //     Err(e)=> {self.state.show_error(format!("{:?}",e)); return;}
                    // };
                    // self.state.compute_graph.environment.0.insert("detector".into(), padamo_api::calculation_nodes::content::Content::String(s.into()));
                    // let msg = PadamoAppMessage::SetDetector(detector);
                    // self.update_tools_loop(Rc::new(msg));
                }
            }
            PadamoAppMessage::ClearState=>{
                self.state.persistent_state.clear();
                let vtl = serde_json::to_string(&DetectorContent::default_vtl()).unwrap();
                self.set_detector(vtl, false);
                self.update_tools_loop(Rc::new(PadamoAppMessage::ClearState));
            }
            other=>{
                self.update_tools_loop(Rc::new(other));
            }
        };
    }


    // fn title(&self) -> String {
    //     "Padamo".into()
    // }

    pub fn view(&self) -> iced::Element<'_, Message> {
        if let Some(msg) = self.state.popup_messages.oldest_message(){
            msg.view()
        }
        else{
            if self.state.is_editing_detectors{
                self.view_loaded_detectors()
            }
            else{
                self.view_normal()
            }
        }

    }

    fn view_loaded_detectors(&self) -> iced::Element<'_, Message> {
        // let res = iced::widget::column![
        //     self.state.detectors.view().map(PadamoAppMessage::LoadedDetectorsMessage),
        //     iced::widget::button("Close").on_press(PadamoAppMessage::SetEditLoadedDetectors(false))
        // ];
        let res = iced_aw::card(iced::widget::text("Loaded detectors"), self.state.detectors.view().map(PadamoAppMessage::LoadedDetectorsMessage))
            .on_close(PadamoAppMessage::SetEditLoadedDetectors(false))
            .max_width(500.0);
        let cont = container(res).center_x(Length::Fill).center_y(Length::Fill);
        cont.into()
    }

    fn view_normal(&self) -> iced::Element<'_, Message> {
        let mut vlist = iced::widget::Column::new();
        vlist = vlist.spacing(10);

        let mut file_menu = Vec::new();
        file_menu.push(Item::new(menu_button("Open", PadamoAppMessage::Open)));
        file_menu.push(Item::new(menu_button("Save", PadamoAppMessage::Save)));
        #[cfg(feature = "button_choose_detector")]
        {
            //file_menu.push(Item::new(menu_button("Choose detector (legacy)", PadamoAppMessage::ChooseDetector)));
            file_menu.push(Item::new(menu_button("Loaded detectors", PadamoAppMessage::SetEditLoadedDetectors(true))));
        }

        #[cfg(feature = "button_clear")]
        file_menu.push(Item::new(menu_button("Clean up", PadamoAppMessage::ClearState)));

        let file_menu = Item::with_menu(title_menu_button("File"), Menu::new(file_menu).max_width(100.0).offset(0.0).spacing(5.0));

        let mut edit_menu = Vec::new();

        //#[cfg(feature = "buttons_edit")]
        {
            edit_menu.push(Item::new(menu_button("Select all", PadamoAppMessage::SelectAll)));
            edit_menu.push(Item::new(menu_button("Copy", PadamoAppMessage::Copy)));
            edit_menu.push(Item::new(menu_button("Paste", PadamoAppMessage::Paste)));
        }


        let edit_menu = Item::with_menu(title_menu_button("Edit"), Menu::new(edit_menu).max_width(100.0).offset(0.0).spacing(5.0));

        //let v = self.state.current_seed.view_row("Seed", "Seed", )
        let mut run_menu = Vec::new();
        run_menu.push(Item::new(menu_button("Run", PadamoAppMessage::Run)));
        #[cfg(feature = "buttons_random")]
        {
            run_menu.push(Item::new(menu_button("Reroll and run", PadamoAppMessage::RerollRun)));
            run_menu.push(Item::new(self.state.current_seed.view_row("Seed","0 or maybe 42",PadamoAppMessage::SetSeed)));
        }


        let run_menu = Item::with_menu(title_menu_button("Run"), Menu::new(run_menu).max_width(150.0).offset(0.0).spacing(5.0));

        let settings_menu = Item::with_menu(title_menu_button("Settings"), Menu::new(vec![
            Item::new(menu_button("Choose workspace directory", PadamoAppMessage::ResetWorkspace)),
        ]).max_width(200.0).offset(0.0).spacing(5.0));

        let mut menu_bar = Vec::new();//vec![file_menu,edit_menu,run_menu,settings_menu]
        menu_bar.push(file_menu);
        #[cfg(feature = "buttons_edit")]
        menu_bar.push(edit_menu);
        menu_bar.push(run_menu);
        #[cfg(feature = "feature_workspace")] // because currently it has only button
        menu_bar.push(settings_menu);
        let menu_bar = MenuBar::new(menu_bar)
            .draw_path(iced_aw::menu::DrawPath::Backdrop)
            .style(|theme:&iced::Theme, status| iced_aw::menu::Style{
                    path_border: iced::Border{
                        radius: iced::border::Radius::new(6.0),
                        ..Default::default()
                    },
                ..primary(theme, status)
            });
        vlist = vlist.push(menu_bar);




        let mut tabs = Tabs::new(PadamoAppMessage::TabSelect)
                    .tab_icon_position(iced_aw::tabs::Position::Bottom);
                    //.on_close(PadamoAppMessage::Noop);

        //let tools = self.tools;
        for (i, tab_obj) in self.tools.iter().enumerate(){
            tabs = tabs.push(i,tab_obj.tab_label(),tab_obj.view(&self.state));
        }
        tabs = tabs.set_active_tab(&self.state.current_page);

        vlist = vlist.push(tabs);

        vlist.into()
        //let underlay:iced::Element<'_,Message> = vlist.into();
        //let overlay:Option<iced::Element<'_,Self::Message>> = None;

        //let popup_window = modal(underlay,overlay).align_x(iced::alignment::Horizontal::Center);
        //popup_window.into()

    }

    pub fn subscription(&self) -> iced::Subscription<Message> {

        iced::Subscription::batch(vec![
            iced::time::every(std::time::Duration::from_millis(33+self.state.add_delay_ms)).map(|_| {
                PadamoAppMessage::Tick
            }),
            iced::keyboard::on_key_press(|key,modifiers|{
                if modifiers.control(){
                    match key {
                        iced::keyboard::key::Key::Character(c)=>{
                            match c.as_str(){
                                "s"=>Some(PadamoAppMessage::Save),
                                "o"=>Some(PadamoAppMessage::Open),
                                "c"=>Some(PadamoAppMessage::Copy),
                                "v"=>Some(PadamoAppMessage::Paste),
                                "a"=>Some(PadamoAppMessage::SelectAll),
                                _=>None
                            }
                        },
                        _=>None
                    }
                }
                else{
                    None
                }

            })
        ])
    }
}

impl Default for Padamo{
    fn default() -> Self {
        Self::new()
    }
}
