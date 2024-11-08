use std::collections::HashMap;

use abi_stable::std_types::{RHashMap, RString, RVec};
#[cfg(feature = "serde")]
use serde::{Serialize,Deserialize};

use crate::{CalculationNodeBox, ConstantContentContainer};

use super::{graph::CalculationSequenceStorage, node::CalculationNodeObject};


#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone,Debug)]
pub struct SmallLink{
    pub output:String,
    pub target_index:usize,
    pub target_input:String,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone,Debug)]
pub struct CompiledNode{
    pub constants:super::content::ConstantContentContainer,
    pub externals:RHashMap<RString,bool>,
    pub identifier:String,
    pub links:Vec<SmallLink>
}

impl CompiledNode {
    pub fn new(constants: super::content::ConstantContentContainer, externals: RHashMap<RString,bool>, identifier: String) -> Self {
        Self { constants, externals, identifier, links:Vec::new() }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone,Debug)]
pub struct CompiledGraph {
    pub nodes:Vec<CompiledNode>
}

impl CompiledGraph{
    pub fn new()->Self{
        Self { nodes: Vec::new() }
    }


    pub fn make_compute_graph(&self, compute_graph:&mut CalculationSequenceStorage, node_registry:&HashMap<String,CalculationNodeBox>){
        compute_graph.clear_graph();
        for node in self.nodes.iter(){

            let calc_node = CalculationNodeObject::new(node_registry[&node.identifier].clone(),Some(node.constants.clone()),Some(node.externals.clone()));
            compute_graph.nodes.push(calc_node);
        }

        for (start_i,node) in self.nodes.iter().enumerate(){
            // let node_ref = node.borrow();
            // for (input_port,_) in node_ref.inputs.iter(){
            //     if let Some(conn) = node_ref.connections.get(input_port){
            //         if let Some(src_node) = conn.node.upgrade(){
            //             let output_port = &conn.port;
            //             if let Some(start_i) = template.lookup_node(&src_node){
            //                 compute_graph.link_fromto(start_i, end_i, &output_port, &input_port);
            //                 println!("{}.{}->{}.{}",start_i,input_port,end_i,output_port);
            //             }
            //         }
            //     }
            //
            //
            // }
            for conn in node.links.iter(){
                compute_graph.link_fromto(start_i, conn.target_index, &conn.output, &conn.target_input);
            }
        }
        //println!("{:?}",compute_graph.nodes);
    }

}
