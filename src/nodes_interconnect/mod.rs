pub mod errors;

use std::collections::HashMap;
use std::path::Path;

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
    nodes:HashMap<String,CalculationNodeBox>
}

impl NodesRegistry{
    pub fn new()->Self{
        Self { nodes: HashMap::new() }
    }

    fn register_node_by_path(&mut self, path:Vec<String>,nodebox:CalculationNodeBox)->Result<(),NodeRegistryError>{
        let key:String = path.join("/");
        println!("Registering node {}", key);
        if self.nodes.contains_key(&key){
            Err(NodeRegistryError::NodeDuplicate(key))
        }
        else {
            self.nodes.insert(key, nodebox);
            Ok(())
        }
    }

    fn register_node_box(&mut self, nodebox:CalculationNodeBox)->Result<(),NodeRegistryError>{
        let path:Vec<String> = nodebox.path().to_vec().iter().map(|x| x.to_string()).collect();
        self.register_node_by_path(path, nodebox)
    }

    pub fn register_node<T:CalculationNode + 'static>(&mut self, node:T)->Result<(),NodeRegistryError>{
        self.register_node_box(CalculationNode_TO::from_value(node, TD_Opaque))
    }

    pub fn load_lib(&mut self, path:&Path)->Result<(),NodeRegistryError>{
        //let path = Path::new(path_str);
        /*if let Some(name_os) = path.file_name(){
            if let Some(name) = name_os.to_str(){
                */
        println!("Loading {}",path.to_str().unwrap());
        //let plugin = PadamoModule_Ref::load_from_file(path).map_err(NodeRegistryError::LibraryError)?;

        let plugin = (|| {
            let header = lib_header_from_path(&path)?;
            header.init_root_module::<PadamoModule_Ref>()
        })().map_err(NodeRegistryError::LibraryError)?;

        let nodes_fn = plugin.nodes();
        let mut nodes = nodes_fn();
        for node in nodes.iter(){
            println!("Detected node {}",node.name());
        }
        for node in nodes.drain(..){
            self.register_node_box(node)?;
        }
        return Ok(());
    }

    pub fn make_tree(&self)->crate::custom_widgets::treeview::Tree{
        let mut tree = crate::custom_widgets::treeview::Tree::new();
        for (_,node) in self.nodes.iter(){
            let path= node.path();
            let path = path.iter().map(|x| x.as_str()).collect();
            tree.parse_path(path);
        }
        tree
    }

    pub fn make_compute_graph(&self, compute_graph:&mut CalculationSequenceStorage, template:&GraphNodeStorage){
        compute_graph.clear_graph();
        for node in template.nodes.iter(){
            let node_ref = node.borrow();
            let mut const_storage = ConstantContentContainer::new();
            for (name,con) in node_ref.constants.constants.iter(){
                const_storage.0.insert(name.clone().into(), con.content.clone().into());
            }
            let calc_node = CalculationNodeObject::new(self.nodes[&node_ref.identifier].clone(),Some(const_storage));


            compute_graph.nodes.push(calc_node);
        }
        for (end_i,node) in template.nodes.iter().enumerate(){
            let node_ref = node.borrow();
            for (input_port,port) in node_ref.inputs.iter(){
                if let Some((src_node,output_port)) = &port.get_linked_node(){
                    if let Some(start_i) = template.lookup_node(src_node){
                        compute_graph.link_fromto(start_i, end_i, &output_port, &input_port);
                        println!("{}.{}->{}.{}",start_i,input_port,end_i,output_port);
                    }
                }
            }
        }
        //println!("{:?}",compute_graph.nodes);
    }

    pub fn create_calculation_node(&self, path:String)->GraphNode{
        let entry = &self.nodes[&path];
        let display_name = entry.name();
        let mut res = GraphNode::new(display_name.into(), path);
        let inputs = entry.inputs();
        for input in inputs.iter(){
            res.add_input(&input.name.to_string(), input.port_type);
        }
        let outputs = entry.outputs();
        for output in outputs.iter(){
            res.add_output(&output.name.to_string(), output.port_type);
        }
        let constants = entry.constants();
        for con in constants.iter(){
            res.add_constant(&con.name.to_string(), con.default_value.clone().into());
        }
        res
    }
}

