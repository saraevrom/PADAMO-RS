use abi_stable::std_types::RVec;
use padamo_api::{calculation_nodes::node::CalculationNodeBox, nodes_vec};
use padamo_api::prelude::CalculationNode_TO;

pub mod ops;
pub mod nodes;

pub fn nodes()->RVec<CalculationNodeBox>{
    nodes_vec!(
        nodes::GaussPSFMeteorTrackNode,
    )

}
