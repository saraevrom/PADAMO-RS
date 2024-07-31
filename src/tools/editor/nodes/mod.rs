pub mod constants;
pub mod errors;

use std::{cell::RefCell, error::Error, fmt::{write, Display}, rc::{Rc, Weak}};

use iced::widget::{canvas::{Frame, Path, self,Text}, shader::wgpu::core::identity};
use serde_json::Map;

use crate::nodes_interconnect::NodesRegistry;

use self::constants::NodeConstantMessage;

use super::editor_program::EditorCanvasMessage;
use ordered_hash_map::OrderedHashMap;
use constants::NodeConstantStorage;
use errors::NodeError;

const PORT_SIZE:f32 = 20.0;
const PORT_INTERVAL:f32 = 5.0;

pub const PORT_CENTER_OFFSET: iced::Vector = iced::Vector::new(PORT_SIZE*0.5,PORT_SIZE*0.5);

fn is_inside(point:iced::Point,top_left:iced::Point,size:iced::Size)->bool{
    let relative = point-top_left;
    let y_inside = (0.0<=relative.y) && (relative.y<=size.height);
    let x_inside = (0.0<=relative.x) && (relative.x<=size.width);
    x_inside && y_inside
}

pub fn make_iced_color(color:padamo_api::calculation_nodes::content::Color)->iced::Color{
    iced::Color { r: color.r, g: color.g, b: color.b, a: color.a  }
}

#[derive(serde::Serialize, serde::Deserialize, Clone,Copy)]
struct SerdePoint{
    x:f32,
    y:f32
}

impl Into<SerdePoint> for iced::Point{
    fn into(self) -> SerdePoint {
        SerdePoint { x: self.x, y: self.y }
    }
}

impl Into<iced::Point> for SerdePoint{
    fn into(self)->iced::Point{
        iced::Point { x: self.x, y: self.y }
    }
}

pub type PortType = padamo_api::calculation_nodes::content::ContentType;


#[derive(Clone,Debug,Eq,PartialEq)]
pub struct OutputDefinition{
    //pub name:String,
    pub port_type:PortType
}

impl OutputDefinition{
    pub fn new( port_type:PortType)->Self{
        Self { port_type }
    }
}

#[derive(Debug)]
pub struct Connection{
    pub node:Weak<RefCell<GraphNode>>, //source node
    pub port:String                    //output port
}

impl Connection{
    pub fn is_valid(&self)->bool{
        self.node.upgrade().is_some()
    }

}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SerdeConnection{
    pub node_index:usize,              //source node
    pub port:String                    //output port
}



#[derive(Debug)]
pub struct InputDefinition{
    //pub name:String,
    pub port_type:PortType,
    pub connection: Option<Connection>
}

impl InputDefinition {
    pub fn new(port_type:PortType)->Self{
        Self { port_type , connection:None}
    }

    pub fn get_linked_node(&self)->Option<(Rc<RefCell<GraphNode>>,String)>{
        if let Some(conn) = &self.connection{
            return conn.node.upgrade().map(|x| (x,conn.port.clone()));
        }
        None
    }

    pub fn remove_dead_connection(&mut self){
        let mut remove = false;
        if let Some(conn) = &self.connection{
            remove = !conn.is_valid();
        }
        if remove{
            self.connection = None
        }
    }

    pub fn disconnect(&mut self){
        self.connection = None;
    }

    pub fn connect_to(&mut self, node:&Rc<RefCell<GraphNode>>,output_port:&str){
        let conn = Connection{node: Rc::downgrade(node), port:output_port.into()};
        self.connection = Some(conn);
    }
}


#[derive(Debug)]
pub struct GraphNode{
    pub position:iced::Point,
    pub title:String,
    pub identifier:String,
    pub size:iced::Size,
    title_offset:f32,
    pub inputs:OrderedHashMap<String,InputDefinition>,
    pub outputs:OrderedHashMap<String,OutputDefinition>,
    pub constants:NodeConstantStorage
}


pub enum NodeMouseHit{
    MainRect,
    Input(String, iced::Point),
    Output(String, iced::Point)
}



impl GraphNode{
    pub fn new(title:String, identifier:String)->Self{
        let mut res = Self {
            position: iced::Point::new(0.0,0.0),
            title,
            size:iced::Size::new(0.0, 0.0),
            inputs: OrderedHashMap::new(),
            outputs: OrderedHashMap::new(),
            constants: NodeConstantStorage::new(),
            title_offset:0.0,
            identifier
        };
        res.reestimate_size();
        res
    }

    pub fn clone_without_links(&self)->Self{
        let mut inputs = OrderedHashMap::new();
        for (key,value) in self.inputs.iter(){
            inputs.insert(key.clone(), InputDefinition::new(value.port_type));
        }
        Self { position: self.position,
            title: self.title.clone(), identifier: self.identifier.clone(),
            size: self.size, title_offset: self.title_offset,
            inputs,
            outputs: self.outputs.clone(), constants: self.constants.clone()

        }
    }

    pub fn remove_dead_connections(&mut self){
        for (_, port) in self.inputs.iter_mut(){
            port.remove_dead_connection();
        }
    }

    pub fn link_from(&mut self, input_port:&str, dependency: &Rc<RefCell<Self>>, output_port:&str)->Result<(),NodeError>{
        if let Some(input_portdef) = self.inputs.get_mut(input_port){
            if let Some(output_portdef) = dependency.borrow().outputs.get(output_port){
                if output_portdef.port_type==input_portdef.port_type{
                    input_portdef.connect_to(dependency, output_port);
                    self.remove_dead_connections();
                    Ok(())
                }
                else{
                    Err(NodeError::IncompatiblePorts(output_portdef.port_type, input_portdef.port_type))
                }
            }
            else {
                Err(NodeError::NoOutput(output_port.into()))
            }
        }
        else {
            Err(NodeError::NoInput(input_port.into()))
        }
    }

    pub fn unlink(&mut self, input_port:&str)->Result<(),NodeError>{
        if let Some(input_portdef) = self.inputs.get_mut(input_port){
            input_portdef.disconnect();
            Ok(())
        }
        else {
            Err(NodeError::NoInput(input_port.into()))
        }
    }

    pub fn unlink_from(&mut self, source_node:Rc<RefCell<Self>>, output_port: &str){
        for (_, port) in self.inputs.iter_mut(){
            //println!("AAA {}", s);
            if let Some(linked_src) = port.get_linked_node(){
                //println!("AAA");
                if Rc::ptr_eq(&linked_src.0, &source_node){
                    if linked_src.1 == output_port{
                        port.disconnect();
                    }
                }
            }
        }
    }

    fn get_y_pos(&self, index:usize)->f32{
        self.title_offset+PORT_INTERVAL+(index as f32)*(PORT_INTERVAL+PORT_SIZE)
    }

    fn get_input_position(&self, index:usize)->iced::Point{
        self.position + iced::Vector::new(0.0, self.get_y_pos(index))
    }

    fn get_output_position(&self, index:usize)->iced::Point{
        self.position + iced::Vector::new(self.size.width-PORT_SIZE, self.get_y_pos(index))
    }

    // pub fn get_center_input_position(&self, index:usize)->iced::Point{
    //     self.get_input_position(index)+PORT_CENTER_OFFSET
    // }

    pub fn get_center_output_position(&self, index:usize)->iced::Point{
        self.get_output_position(index)+PORT_CENTER_OFFSET
    }

    fn max_input_title_size(&self)->usize{
        self.inputs.iter().map(|x| x.0.len()).max().unwrap_or(0)
    }

    fn max_output_title_size(&self)->usize{
        self.outputs.iter().map(|x| x.0.len()).max().unwrap_or(0)
    }

    fn get_output_index(&self, name:&str)->Option<usize>{
        self.outputs.keys().position(|x| x==name)
    }

    fn make_text(&self)->Text{
        Text{
            content:self.title.clone(),
            font:iced::Font{
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn reestimate_size(&mut self){
        let txt = self.make_text();
        self.title_offset = txt.line_height.to_absolute(txt.size).0;
        let port_chars:f32 = 3.0*PORT_SIZE+((self.max_input_title_size()+self.max_output_title_size()) as f32) * txt.size.0/2.0;
        let width = (self.title.len() as f32)*txt.size.0/2.0;
        let ports_addition = usize::max(self.inputs.len(),self.outputs.len());
        let height = self.title_offset+self.get_y_pos(ports_addition)- PORT_SIZE;
        let width = f32::max(width, port_chars);
        self.size = iced::Size::new(width, height);
    }

    pub fn draw_links(&self, frame:&mut Frame){
        for (i,(_, port)) in self.inputs.iter().enumerate(){
            let pos = self.get_input_position(i);

            if let Some((src,output_port)) = port.get_linked_node(){
                let src_imm = src.borrow();
                if let Some(src_port_id) = src_imm.get_output_index(&output_port){
                    let src_pos = src_imm.get_center_output_position(src_port_id);
                    let tgt_pos = pos + PORT_CENTER_OFFSET;
                    let line = Path::line(src_pos, tgt_pos);
                    frame.stroke(&line, canvas::Stroke::default().with_width(2.0).with_color(iced::Color::BLACK));
                }
            }
        }
    }

    pub fn bottom_right(&self)->iced::Point{
        self.position+self.size.into()
    }

    pub fn draw(&self, frame:&mut Frame, highlight:bool){

        //let txt = Text{content:self.title.clone(),position:self.position,..Default::default()};
        let mut txt = self.make_text();
        txt.position = self.position;
        let main_rect = Path::rectangle(self.position,self.size);
        let linewidth:f32 = if highlight {6.0} else {2.0};
        frame.stroke(&main_rect, canvas::Stroke::default().with_width(linewidth).with_color(iced::Color::BLACK));
        frame.fill(&main_rect, iced::Color::WHITE);
        frame.fill_text(txt);
        let port_size = iced::Size::new(PORT_SIZE, PORT_SIZE);
        for (i,(title, port)) in self.inputs.iter().enumerate(){
            let pos = self.get_input_position(i);
            let port_rect = Path::rectangle(pos, port_size);

            frame.stroke(&port_rect, canvas::Stroke::default().with_width(2.0).with_color(iced::Color::BLACK));
            let color:iced::Color = make_iced_color(port.port_type.get_color());
            frame.fill(&port_rect, color);

            let label = Text{content:title.into(), position: pos+iced::Vector::new(PORT_SIZE, 0.0), ..Default::default()};
            frame.fill_text(label);
        }

        for (i,(title, port)) in self.outputs.iter().enumerate(){
            let pos = self.get_output_position(i);
            let port_rect = Path::rectangle(pos, port_size);
            frame.stroke(&port_rect, canvas::Stroke::default().with_width(2.0).with_color(iced::Color::BLACK));
            frame.fill(&port_rect, make_iced_color(port.port_type.get_color()));
            let label = Text{content:title.into(), position:pos,horizontal_alignment:iced::alignment::Horizontal::Right, ..Default::default()};
            frame.fill_text(label);
        }
    }

    pub fn mouse_event(&self, point:iced::Point)->Option<NodeMouseHit>{
        if is_inside(point, self.position, self.size){
            let port_size = iced::Size::new(PORT_SIZE, PORT_SIZE);
            for (i,(title, port)) in self.inputs.iter().enumerate(){
                let pos = self.get_input_position(i);
                if is_inside(point, pos, port_size){
                    return Some(NodeMouseHit::Input(title.clone(),self.get_input_position(i)+PORT_CENTER_OFFSET));
                }
            }
            for (i,(title, port)) in self.outputs.iter().enumerate(){
                let pos = self.get_output_position(i);
                if is_inside(point, pos, port_size){
                    return Some(NodeMouseHit::Output(title.clone(),self.get_output_position(i)+PORT_CENTER_OFFSET));
                }
            }
            return Some(NodeMouseHit::MainRect);
        }
        None
    }

    pub fn add_input(&mut self, name:&str, port_type:PortType){
        let def = InputDefinition::new(port_type);
        self.inputs.insert(name.into(), def);
        self.reestimate_size();
    }

    pub fn add_output(&mut self, name:&str, port_type:PortType){
        let def = OutputDefinition::new(port_type);
        self.outputs.insert(name.into(), def);
        self.reestimate_size();
    }

    pub fn add_constant(&mut self, key:&str, value: constants::NodeConstantContent){
        self.constants.add_constant(key,value)
    }

    pub fn modify_constant(&mut self, msg:constants::NodeConstantMessage)->Result<(),NodeError>{
        self.constants.modify_constant(msg)
    }


//
}


#[derive(Debug)]
pub struct NodeSelection{
    pub selected_nodes:Vec<Weak<RefCell<GraphNode>>>
}


impl NodeSelection{
    pub fn new()->Self{
        Self { selected_nodes: Vec::new() }
    }

    pub fn simplify(&mut self){
        self.selected_nodes = self.selected_nodes.drain(..).filter(|x| x.upgrade().is_some()).collect();
    }

    pub fn clear(&mut self){
        self.selected_nodes.clear();
    }

    pub fn get_solitary_node(&self)->Option<Rc<RefCell<GraphNode>>>{
        if self.selected_nodes.len()!=1{
            return None;
        }
        if let Some(v) = self.selected_nodes.get(0){
            v.upgrade()
        }
        else{
            None
        }
    }

    pub fn contains_node(&self, node:&Rc<RefCell<GraphNode>>)->bool{
        for r in self.selected_nodes.iter(){
            if let Some(v) = r.upgrade(){
                if Rc::ptr_eq(&v, node){
                    return true;
                }
            }
        }
        false
    }

    pub fn add_to_selection(&mut self, node:&Rc<RefCell<GraphNode>>){
        if self.contains_node(node){
            return;
        }
        self.selected_nodes.push(Rc::downgrade(node));
    }



}


#[derive(Debug)]
pub struct GraphNodeStorage{
    pub nodes:Vec<Rc<RefCell<GraphNode>>>,
    //selected_node: Weak<RefCell<GraphNode>>,
    selection:NodeSelection,
    shift_mod:bool
}

#[derive(Debug)]
pub struct GraphNodeCloneBuffer{
    pub storage:GraphNodeStorage,
    pub offset:iced::Point
}

#[derive(Debug)]
pub enum GraphDeserializationError{
    NodeNotFound(String),
    JsonError(serde_json::Error),
    NotArray,
    NotFound(String),
    WrongFormat(String),
}

impl Display for GraphDeserializationError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            GraphDeserializationError::NodeNotFound(v) => write!(f,"Node {} is not found", v),
            GraphDeserializationError::JsonError(e)=>write!(f, "JSON deserialization error: {}", e),
            GraphDeserializationError::NotArray=>write!(f, "JSON data is not array"),
            GraphDeserializationError::WrongFormat(v)=>write!(f, "Identifier {} has wrong format", v),
            GraphDeserializationError::NotFound(v)=>write!(f, "Identifier {} is not found", v),
        }
    }
}

impl Error for GraphDeserializationError{

}

impl GraphNodeStorage{
    pub fn new()->Self{
        Self { nodes: Vec::new(),
            //selected_node:Weak::new()
            selection:NodeSelection::new(),
            shift_mod:false,
        }
    }

    pub fn clone_selection(&self)->Option<GraphNodeCloneBuffer>{
        if self.selection.selected_nodes.len()==0{
            return None;
        }
        let mut res = Self::new();
        let mut offset = iced::Point::new(-10.0f32, -0.0f32);
        for node_weak in self.selection.selected_nodes.iter(){
            if let Some(node_rc) = node_weak.upgrade(){
                let other = node_rc.borrow().clone_without_links();
                if offset.x<0.0 || offset.x<other.position.x || offset.y<other.position.y{
                    offset = other.position;
                }
                res.insert_node(other);
            }
        }
        Some(GraphNodeCloneBuffer{
            storage:res,
            offset
        })
    }

    pub fn instantiate(&mut self, buffer:&GraphNodeCloneBuffer, position:iced::Point){
        let delta = position - buffer.offset;
        for node in buffer.storage.nodes.iter(){
            let mut newnode = node.borrow().clone_without_links();
            newnode.position = newnode.position+delta;
            self.insert_node(newnode);
        }
    }

    pub fn lookup_node(&self, node_tgt:&Rc<RefCell<GraphNode>>)->Option<usize>{
        for (i,node) in self.nodes.iter().enumerate(){
            if Rc::ptr_eq(node, node_tgt){
                return Some(i);
            }
        }
        None
    }

    pub fn remove_dead_connections(&mut self){
        for node in self.nodes.iter(){
            let mut node_mut = node.borrow_mut();
            node_mut.remove_dead_connections();
        }
    }

    pub fn insert_node(&mut self, node:GraphNode){
        self.nodes.push(Rc::new(RefCell::new(node)));
    }

    pub fn draw(&self,frame: &mut canvas::Frame){
        for (_,node) in self.nodes.iter().enumerate(){
            node.borrow().draw_links(frame);
        }
        for node in self.nodes.iter(){
            let highlight = self.selection.contains_node(node);
            // let highlight:bool = if let Some(s) = self.selected_node.upgrade(){
            //     Rc::ptr_eq(&s, node)
            // }
            // else{
            //     false
            // };
            node.borrow().draw(frame, highlight);
        }
    }

    pub fn get_node_data(&self, point:iced::Point)->Option<(usize, iced::Point, iced::Size, NodeMouseHit)>{
        for (i,node) in self.nodes.iter().enumerate().rev(){
            let node_in = node.borrow();
            if let Some(x) = node_in.mouse_event(point){
                return Some((i, node_in.position, node_in.size,x));
            }
        }
        None
    }

    pub fn handle_message(&mut self, msg:&EditorCanvasMessage){
        match msg {
            EditorCanvasMessage::MoveNode { index, position }=>{
                if let Some(node) = self.nodes.get(*index){
                    let old_position = {
                        node.borrow().position
                    };
                    let mut target_position = *position;
                    // if target_position.x<=0.0{
                    //     target_position.x = 0.0;
                    // }
                    // if target_position.y<=0.0{
                    //     target_position.y = 0.0;
                    // }

                    let mut delta = target_position-old_position;

                    //TODO: needs check if we can move all nodes
                    for selected_node_weak in self.selection.selected_nodes.iter(){
                        if let Some(selected_node) = selected_node_weak.upgrade(){
                            let borrowed_node = selected_node.borrow();
                            let mut newpos = borrowed_node.position + delta;
                            let mut change_delta = false;
                            if newpos.x<=0.0{
                                newpos.x = 0.0;
                                change_delta = true;
                            }

                            if newpos.y<=0.0{
                                newpos.y = 0.0;
                                change_delta = true;
                            }

                            if change_delta{
                                delta = newpos - borrowed_node.position;
                            }
                        }
                    }

                    for selected_node_weak in self.selection.selected_nodes.iter(){
                        if let Some(selected_node) = selected_node_weak.upgrade(){
                            let mut mut_borrowed_node = selected_node.borrow_mut();
                            let newpos = mut_borrowed_node.position + delta;
                            mut_borrowed_node.position = newpos;
                        }
                    }

                    //node.borrow_mut().position = target_position;
                }
            }
            EditorCanvasMessage::LinkNode { from, output_port, to, input_port }=>{
                let link_res = self.link_from_to(*from, output_port,*to, input_port);
                println!("Linking {}.{}->{}.{}",from,output_port,to,input_port);
                match link_res {
                    Ok(())=>{
                        println!("Linked!");
                    }
                    Err(e)=>{
                        println!("{}",e);
                    }
                }
            },
            EditorCanvasMessage::UnlinkOutput { node, output_port } => {
                let res = self.unlink_output(*node, &output_port);
                match res{
                    Ok(())=>println!("Output unlinked"),
                    Err(x)=>println!("{}",x)
                }
            },
            EditorCanvasMessage::UnlinkInput { node, input_port } =>{
                let res = self.unlink_input(*node, &input_port );
                match res{
                    Ok(())=>println!("Output unlinked"),
                    Err(x)=>println!("{}",x)
                }
            }
            EditorCanvasMessage::Unselect=>{
                self.unselect_nodes();
            }
            EditorCanvasMessage::Select(i)=>{
                if !self.shift_mod && !self.is_selected(*i){
                    self.selection.clear();
                }
                self.add_to_selection(*i);
            },
            EditorCanvasMessage::SquareSelect(startpos, endpos)=>{
                if !self.shift_mod{
                    self.selection.clear();
                }
                let minpos_x = startpos.x.min(endpos.x);
                let minpos_y = startpos.y.min(endpos.y);

                let maxpos_x = startpos.x.max(endpos.x);
                let maxpos_y = startpos.y.max(endpos.y);

                let total_width = maxpos_x - minpos_x;
                let total_height = maxpos_y - minpos_y;

                for i in 0..self.nodes.len(){
                    let node = &self.nodes[i];
                    let node_size = node.borrow().size;
                    let truncated_width = total_width - node_size.width;
                    let truncated_height = total_height - node_size.height;
                    let node_pos = node.borrow().position;
                    let truncated_x = node_pos.x-minpos_x;
                    let truncated_y = node_pos.y-minpos_y;
                    if truncated_x>0.0 && truncated_x<truncated_width &&
                        truncated_y>0.0 && truncated_y<truncated_height{
                            self.add_to_selection(i);
                        }
                }
            }
            EditorCanvasMessage::SetShift(shift)=>{
                self.shift_mod = *shift;
            }
            EditorCanvasMessage::DeleteSelectedNode=>{
                self.delete_selected_nodes();
            },
            EditorCanvasMessage::ConstantEdit(v)=>{
                // if let Some(x) = self.selected_node.upgrade(){
                //     let mut selected = x.borrow_mut();
                //     selected.modify_constant(v.clone()).unwrap();
                // }

                if let Some(x) = self.selection.get_solitary_node(){
                    let mut selected = x.borrow_mut();
                    selected.modify_constant(v.clone()).unwrap();
                }
            }
            EditorCanvasMessage::CancelPaste=>(),
            EditorCanvasMessage::CommitPaste(_)=>(),
            //_=>(),
        }
    }

    pub fn link_from_to(&mut self, source_id:usize, output_port:&str, target_id: usize, input_port:&str)->Result<(),NodeError>{
        if source_id==target_id{
            return Err(NodeError::SameNodeLink);
        }
        if let Some(src) = self.nodes.get(source_id){
            if let Some(tgt)  = self.nodes.get(target_id){
                let res = tgt.borrow_mut().link_from(input_port, src, output_port);
                self.remove_dead_connections();
                res
            }
            else{
                Err(NodeError::NodeIndexError(target_id))
            }
        }
        else{
            Err(NodeError::NodeIndexError(source_id))
        }

    }

    pub fn unlink_input(&mut self, node_id:usize, input_port:&str)->Result<(), NodeError>{
        if let Some(node) = self.nodes.get(node_id){
            let res = node.borrow_mut().unlink(input_port);
            self.remove_dead_connections();
            res
        }
        else {
            Err(NodeError::NoInput(input_port.into()))
        }
    }

    pub fn unlink_output(&mut self, node_id:usize, output_port:&str)->Result<(), NodeError>{
        if let Some(src_node) = self.nodes.get(node_id){
            for tgt_node in self.nodes.iter(){
                //println!("TEST");
                if !Rc::ptr_eq(tgt_node, src_node){
                    tgt_node.borrow_mut().unlink_from(src_node.clone(), output_port);
                }
            }
            self.remove_dead_connections();
            Ok(())
        }
        else {
            Err(NodeError::NoOutput(output_port.into()))
        }
    }

    pub fn unselect_nodes(&mut self){
        self.selection.clear()
        //self.selected_node = Weak::new();
    }

    pub fn add_to_selection(&mut self, i:usize){
        if let Some(x) = self.nodes.get(i){
            self.selection.add_to_selection(x);
            //self.selected_node = Rc::downgrade(x);
        }
    }

    pub fn is_selected(&self, i:usize)->bool{
        if let Some(x) = self.nodes.get(i){
            self.selection.contains_node(x)
        }
        else{
            false
        }
    }

    pub fn select_node(&mut self, i:usize){
        self.unselect_nodes();
        self.add_to_selection(i);
        // if let Some(x) = self.nodes.get(i){
        //     self.selected_node = Rc::downgrade(x);
        // }
    }

    // pub fn delete_selected_node(&mut self){
    //     if let Some(sel) = self.selected_node.upgrade(){
    //         let mut rem_id = None;
    //         for (i,x) in self.nodes.iter().enumerate(){
    //             if Rc::ptr_eq(x,& sel){
    //                 rem_id = Some(i);
    //                 break;
    //             }
    //         }
    //         if let Some(i) = rem_id{
    //             self.nodes.remove(i);
    //         }
    //     }
    // }

    pub fn delete_selected_nodes(&mut self){
        for node in self.selection.selected_nodes.iter(){
            if let Some(sel) = node.upgrade(){
                let mut rem_id = None;
                for (i,x) in self.nodes.iter().enumerate(){
                    if Rc::ptr_eq(x,& sel){
                        rem_id = Some(i);
                        break;
                    }
                }
                if let Some(i) = rem_id{
                    self.nodes.remove(i);
                }
            }
        }
        self.selection.clear();
    }

    pub fn view_selected_constants(&self)->Option<constants::NodeConstantStorage>{
        //if let Some(x) = self.selected_node.upgrade(){
        if let Some(x) = self.selection.get_solitary_node(){
            Some(x.borrow().constants.clone())
        }
        else{
            None
        }
    }

    pub fn clear(&mut self){
        self.nodes.clear();
    }


    pub fn full_size(&self)->iced::Size{
        let mut max_x:f32 = 0.0;
        let mut max_y:f32 = 0.0;
        for node in self.nodes.iter(){
            let br = node.borrow().bottom_right();
            if br.x>max_x{
                max_x = br.x;
            }
            if br.y>max_y{
                max_y = br.y;
            }
        }
        iced::Size { width: max_x, height: max_y }
    }

    pub fn serialize(&self)->serde_json::Value{
        let mut values:Vec<serde_json::Value> = Vec::with_capacity(self.nodes.len());
        for node in self.nodes.iter(){
            let node_ref = node.borrow();
            let mut entry = Map::new();
            let pos:SerdePoint = node_ref.position.into();

            entry.insert("position".into(), serde_json::to_value(pos).unwrap());
            entry.insert("identifier".into(), node_ref.identifier.clone().into());

            let mut conns = serde_json::Map::new();
            for (inp_port,inp_def) in node_ref.inputs.iter(){
                let value = if let Some(conn) = &inp_def.connection{
                    if let Some(conn_rc) = conn.node.upgrade(){
                        if let Some(node_index) = self.lookup_node(&conn_rc){

                            let out_port = conn.port.clone();
                            let val = SerdeConnection{node_index, port:out_port};
                            serde_json::to_value(val).unwrap()
                        }
                        else{
                            serde_json::Value::Null
                        }
                    }
                    else{
                        serde_json::Value::Null
                    }
                }
                else{
                    serde_json::Value::Null
                };
                conns.insert(inp_port.clone(), value);
            }
            let mut mapped= serde_json::Map::new();
            for (k,v) in node_ref.constants.constants.iter(){
                mapped.insert(k.clone(), serde_json::to_value(v.content.clone()).unwrap());
            }

            let consts = serde_json::Value::Object(mapped);

            entry.insert("connections".into(), conns.into());
            entry.insert("constants".into(), consts);
            values.push(serde_json::Value::Object(entry));
        }
        serde_json::Value::Array(values)
    }


    pub fn deserialize(&mut self, registry:&NodesRegistry, value:serde_json::Value)->Result<(),GraphDeserializationError>{
        if let serde_json::Value::Array(arr) = value{
            self.clear();
            for value in arr.iter(){
                if let serde_json::Value::Object(obj) = value{
                    'stop: {
                        //let identifier = if let Some(serde_json::Value::String(identifier) )= obj.get("identifier"){identifier}
                        //else { break 'stop; };
                        let identifier = obj.get("identifier").ok_or(GraphDeserializationError::NotFound("identifier".into()))?;
                        let identifier = if let serde_json::Value::String(v) = identifier {v} else {return Err(GraphDeserializationError::WrongFormat("identifier".into()));};

                        // let position = if let Some(position ) = obj.get("position"){
                        //     position
                        // }
                        // else { break 'stop; };
                        let position = obj.get("position").ok_or(GraphDeserializationError::NodeNotFound("position".into()))?;


                        //let pos:SerdePoint = if let Ok(pos) = serde_json::from_value(position.clone()) {pos}
                        //else { break 'stop; };
                        let pos:SerdePoint = serde_json::from_value(position.clone()).map_err(GraphDeserializationError::JsonError)?;

                        // let consts = if let Some(serde_json::Value::Object(c)) = obj.get("constants") {c}
                        // else { break 'stop; };

                        let consts = obj.get("constants").ok_or(GraphDeserializationError::NodeNotFound("constants".into()))?;
                        let consts = if let serde_json::Value::Object(c) = consts {c} else {return Err(GraphDeserializationError::WrongFormat("constants".into()));};


                        let mut node = if let Some(v) = registry.create_calculation_node(identifier.clone()) {v}
                        else{
                            return Err(GraphDeserializationError::NodeNotFound(identifier.clone()));
                        };

                        for (key,con) in consts.iter(){
                            let deserialized_con = serde_json::from_value(con.clone());
                            if let Ok(con_val) = &deserialized_con {
                                println!("Constant deserialize successs");
                                if let Some(entry) = node.constants.constants.get_mut(key){
                                    println!("Entry found");
                                    if entry.content.is_compatible(con_val){
                                        println!("Entry compatible");
                                        (*entry).content = con_val.clone();
                                        entry.update_buffer();
                                    }
                                }
                            }
                        }
                        node.position = pos.into();


                        self.insert_node(node);
                    }
                }
            }

            for (target_node,value) in arr.iter().enumerate(){
                let obj = if let serde_json::Value::Object(obj) = value{obj} else{panic!("Why cannot I look through value again?")};
                'stop: {
                    let conns = if let Some(serde_json::Value::Object(c)) = obj.get("connections") {c} else {break 'stop;};
                    for (input_port, conn) in conns.iter(){
                        let connection:SerdeConnection = if let Ok(c) = serde_json::from_value(conn.clone()) {c} else {break 'stop;};
                        let output_port = connection.port;
                        let start_node = connection.node_index;
                        self.nodes[target_node].borrow_mut().link_from(input_port, &self.nodes[start_node], &output_port).unwrap();
                    }
                }
            }
            Ok(())
        }
        else{
            Err(GraphDeserializationError::NotArray)
        }
    }
}
