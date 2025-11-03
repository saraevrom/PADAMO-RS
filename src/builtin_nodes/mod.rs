pub mod viewer;
pub mod viewer_smart;

pub fn register_nodes(nodes:&mut crate::nodes_interconnect::NodesRegistry){
    viewer::register_nodes(nodes);
    viewer_smart::register_nodes(nodes);
}
