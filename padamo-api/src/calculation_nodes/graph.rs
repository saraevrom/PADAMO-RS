use std::borrow::BorrowMut;
use std::collections::{HashMap, VecDeque};
use std::fmt::format;
use abi_stable::StableAbi;
use abi_stable::std_types::{RString, Tuple2};
use topo_sort::{TopoSort,SortResults};

use crate::rng::RandomState;
use crate::ConstantContent;

use super::content::{Content, ContentContainer, ConstantContentContainer};
use super::errors::ExecutionError;
use super::node::CalculationNodeObject;
use super::node::IOData;

use abi_stable::std_types::RHashMap;

#[repr(C)]
#[derive(StableAbi,Clone,Debug,Hash,PartialEq,Eq)]
pub struct PortKey{
    port_name:RString,
    index:usize
}

pub struct CalculationSequenceStorage{
    pub nodes:Vec<CalculationNodeObject>,
    pub nets:HashMap<PortKey,Content>,
    pub environment:ContentContainer,
}

impl CalculationSequenceStorage{
    pub fn new()->Self{
        Self { nodes:Vec::new(), nets: HashMap::new() , environment:ContentContainer::new()}
    }

    pub fn push_node(&mut self,node:CalculationNodeObject){
        self.nodes.push(node);
    }

    fn request_output_net_data(&self,i:usize, name:&str)->Result<Content,ExecutionError>{
        let key = PortKey{index:i, port_name:name.into()};
        //println!("NET REQUEST {:?}",key);
        if let Some(v) = self.nets.get(&key){
            Ok(v.clone())
        }
        else{
            Err(ExecutionError::NotConnected(name.into()))
        }
    }

    pub fn execute_node(&mut self, i:usize,random_state:&mut RandomState)->Result<(),ExecutionError>{
        let node = &self.nodes[i];
        let mut inputs:RHashMap<RString, Content> = RHashMap::new();
        let mut input_mapping:HashMap<_, _> = node.get_connections().into_result()?.into();
        for (port, map)  in input_mapping.drain(){
            let input_value = self.request_output_net_data(map.index, &map.port_name)?;
            inputs.insert(port, input_value);
        }
        drop(input_mapping);
        let inputs = ContentContainer(inputs);
        let mut consts = node.constants.clone();
        //println!("Node inputs: {:?}",inputs);
        for (constant_key, constant_external) in node.constants_external_flags.iter().map(|x| x.into_tuple()){
            if *constant_external{
                let req_key:RString = format!("constant_{}",constant_key).into();
                //println!("override constant {:?} with input {:?}",constant_key,req_key);
                if let Some(v) = inputs.0.get((&req_key).into()){
                    'noset:{
                        let ext_value = match v{
                            Content::Integer(i)=>ConstantContent::Integer(*i),
                            Content::Float(f)=>ConstantContent::Float(*f),
                            Content::Boolean(b)=>ConstantContent::Boolean(*b),
                            Content::String(s)=>ConstantContent::String(s.to_owned()),
                            _=>{break 'noset;}
                        };
                        consts.0.insert(constant_key.to_owned(),ext_value);
                    }

                };
            }
        }
        //println!("Constants set {:?}",consts);

        let output_defs = node.calculator.outputs();
        let mut outputs = IOData::new(output_defs);
        node.calculator.calculate(inputs, &mut outputs, consts,&mut self.environment, random_state).into_result()?;
        let mut explicit_outputs:HashMap<RString,Content> = outputs.clarify()?.into();
        for (port,value) in explicit_outputs.drain(){
            let key = PortKey{
               index:i,
               port_name:port.into()
            };
            //println!("INSERT NET {:?} = {:?}", key, value);
            self.nets.insert(key, value);
        }

        //let outputs_defs = node.calculator.outputs();

        Ok(())
    }

    pub fn clear_graph(&mut self){
        self.nets.clear();
        self.nodes.clear();
    }

    pub fn link_fromto(&mut self, start_i:usize, end_i:usize, output_port:&str, input_port:&str){
        self.nodes[end_i].connect_from(input_port, PortKey { port_name: output_port.into(), index: start_i });
    }

    pub fn edit_constants(&mut self, i:usize)->&mut ConstantContentContainer{
        &mut self.nodes[i].constants
    }

    pub fn execute(&mut self,random_seed:u64)->Result<(),ExecutionError>{
        let mut random_state = RandomState::new(random_seed);
        //println!("EXEC GRAPH {:?}",self.nodes);
        let mut sorter = TopoSort::with_capacity(self.nodes.len());
        let mut nodes_under_processing:VecDeque<usize> = self.nodes.iter()
            .enumerate()
            .filter(|(_,n)| n.calculator.is_primary()).map(|(i,_)| i)
            .collect();

        while let Some(i) = nodes_under_processing.pop_front(){
            let node = &self.nodes[i];
            let conns:HashMap<_, _> = node.get_connections().into_result()?.into();
            let deps:Vec<_> = conns.values().map(|x| x.index).collect();
            //if sorter.
            if let None = sorter.get(&i){
                //println!("{} depends on {:?}", i, deps);
                for dep in deps.iter(){
                    nodes_under_processing.push_back(*dep);
                }
                sorter.insert(i,deps);
            }
        }

        if let SortResults::Full(sorted) = sorter.into_vec_nodes(){
            for i in sorted.iter(){
                //println!("Executing node {}", i);
                let mut state2 = random_state.separate(*i as u64);
                self.execute_node(*i,&mut state2)?;
            }
        }
        else{
            return Err(ExecutionError::CycleError);
        }

        Ok(())
    }
}
