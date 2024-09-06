pub mod errors;

use std::collections::HashMap;
use std::path::Path;

use abi_stable::std_types::RHashMap;
use padamo_api::prelude::{CalculationNodeBox, CalculationNode, CalculationNode_TO};
use padamo_api::PadamoModule_Ref;
use abi_stable::library::{RootModule, lib_header_from_path};

use errors::NodeRegistryError;


use crate::tools::editor::nodes::{GraphNodeStorage, GraphNode};
use padamo_api::calculation_nodes::graph::CalculationSequenceStorage;
use padamo_api::calculation_nodes::node::CalculationNodeObject;
use padamo_api::calculation_nodes::content::{ContentContainer, ConstantContentContainer};

use abi_stable::sabi_trait::prelude::TD_Opaque;


pub struct NodesRegistry{
    nodes:HashMap<String,CalculationNodeBox>,
    legacy_map:HashMap<String,String>
}

impl NodesRegistry{
    pub fn new()->Self{
        Self { nodes: HashMap::new(), legacy_map:HashMap::new() }
    }


    fn register_node_box(&mut self, nodebox:CalculationNodeBox)->Result<(),NodeRegistryError>{
        let key:String = nodebox.identifier().into();
        //self.register_node_by_path(path, nodebox)
        println!("Registering node with key {}", key);
        if self.nodes.contains_key(&key){
            Err(NodeRegistryError::NodeDuplicate(key))
        }
        else {
            if let Some(old_id) = nodebox.old_identifier().into_option(){
                let old_id:String = old_id.into();
                if self.legacy_map.contains_key(&old_id){
                    Err(NodeRegistryError::LegacyDuplicate(old_id))
                }
                else{
                    //println!("Registered legacy mapping {}->{}",old_id,key);
                    self.legacy_map.insert(old_id, key.clone());
                    self.nodes.insert(key, nodebox);
                    Ok(())
                }
            }
            else{
                    self.nodes.insert(key, nodebox);
                    Ok(())
            }
        }
    }

    pub fn register_node<T:CalculationNode + 'static>(&mut self, node:T)->Result<(),NodeRegistryError>{
        self.register_node_box(CalculationNode_TO::from_value(node, TD_Opaque))
    }

    pub fn load_lib(&mut self, path:&Path)->Result<(),NodeRegistryError>{
        //let path = Path::new(path_str);
        /*if let Some(name_os) = path.file_name(){
            if let Some(name) = name_os.to_str(){
                */
        let path_str = path.to_str().ok_or(NodeRegistryError::PathError)?;
        let parent_dir = path.parent().ok_or(NodeRegistryError::PathError)?.to_str().ok_or(NodeRegistryError::PathError)?;
        println!("Loading {}",path_str);
        //let plugin = PadamoModule_Ref::load_from_file(path).map_err(NodeRegistryError::LibraryError)?;

        let plugin = (|| {
            let header = lib_header_from_path(&path)?;
            header.init_root_module::<PadamoModule_Ref>()
        })().map_err(NodeRegistryError::LibraryError)?;

        let nodes_fn = plugin.nodes();
        let mut nodes = nodes_fn(parent_dir.into());
        for node in nodes.iter(){
            println!("Detected node {}",node.name());
        }
        for node in nodes.drain(..){
            self.register_node_box(node)?;
        }
        return Ok(());
    }

    pub fn make_tree(&self)->crate::custom_widgets::treeview::Tree<String>{
        let mut tree = crate::custom_widgets::treeview::Tree::new();
        for (_,node) in self.nodes.iter(){
            let path= node.path();
            let path = path.iter().map(|x| x.as_str()).collect();
            //println!("Insert node {} as{:?}", node.identifier(), path);
            tree.parse_path(path, Some(node.identifier().into()));
        }
        tree.sort_nodes();
        tree
    }

    pub fn make_compute_graph(&self, compute_graph:&mut CalculationSequenceStorage, template:&GraphNodeStorage){
        compute_graph.clear_graph();
        for node in template.nodes.iter(){
            let node_ref = node.borrow();
            let mut const_storage = ConstantContentContainer::new();
            let mut externals = RHashMap::new();
            for (name,con) in node_ref.constants.constants.iter(){
                const_storage.0.insert(name.clone().into(), con.content.clone().into());
                externals.insert(name.clone().into(), con.use_external);
            }
            let calc_node = CalculationNodeObject::new(node_ref.represented_node.0.clone(),Some(const_storage),Some(externals));
            compute_graph.nodes.push(calc_node);
        }
        for (end_i,node) in template.nodes.iter().enumerate(){
            let node_ref = node.borrow();
            for (input_port,_) in node_ref.inputs.iter(){
                if let Some(conn) = node_ref.connections.get(input_port){
                    if let Some(src_node) = conn.node.upgrade(){
                        let output_port = &conn.port;
                        if let Some(start_i) = template.lookup_node(&src_node){
                            compute_graph.link_fromto(start_i, end_i, &output_port, &input_port);
                            println!("{}.{}->{}.{}",start_i,input_port,end_i,output_port);
                        }
                    }
                }


            }
        }
        //println!("{:?}",compute_graph.nodes);
    }

    pub fn create_calculation_node(&self, identifier:String)->Option<GraphNode>{
        let mut true_id = identifier;
        if !self.nodes.contains_key(&true_id){
            if self.legacy_map.contains_key(&true_id){
                print!("Remapping old id \"{}\" ",true_id);
                true_id = self.legacy_map[&true_id].clone();
                println!("to \"{}\"",true_id);
            }
            else{
                return None;
            }
        }
        let identifier = true_id;

        let entry = &self.nodes[&identifier];
        //let display_name = entry.name();
        let mut res = GraphNode::new(entry.clone());
        // let inputs = entry.inputs();
        // for input in inputs.iter(){
        //     res.add_input(&input.name.to_string(), input.port_type);
        // }
        // let outputs = entry.outputs();
        // for output in outputs.iter(){
        //     res.add_output(&output.name.to_string(), output.port_type);
        // }
        let constants = entry.constants();
        for con in constants.iter(){
            res.add_constant(&con.name.to_string(), con.default_value.clone().into(),con.display_name.to_string());
        }
        Some(res)
    }
}

