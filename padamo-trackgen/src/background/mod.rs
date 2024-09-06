use padamo_api::prelude::*;
use padamo_api::nodes_vec;
use abi_stable::std_types::RVec;

pub mod nodes;
pub mod ops;


pub fn nodes()->RVec<CalculationNodeBox>{
    nodes_vec!(
        nodes::AdditiveNormalNoiseNode,
        nodes::BlankDataNode,
    )
}
