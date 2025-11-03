pub mod viewer;

pub fn register_nodes(nodes:&mut crate::nodes_interconnect::NodesRegistry){
    viewer::register_nodes(nodes);
}
