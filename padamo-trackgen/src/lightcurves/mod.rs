use padamo_api::prelude::*;
use padamo_api::nodes_vec;
use abi_stable::std_types::RVec;

pub mod lc_nodes;
pub mod psf;
// pub mod lc_nodes_old;


pub fn nodes()->RVec<CalculationNodeBox>{
    nodes_vec!(
        lc_nodes::LCSwitchNode,
        lc_nodes::LinearLCNode,
        lc_nodes::ExponentLCNode,
        lc_nodes::LCPivotNode,
        lc_nodes::TerminationLCNode,
        lc_nodes::ConstantLCNode,
        lc_nodes::MultiplyByFloatNode,

        psf::GaussianPSF,
        psf::MoffatPSF,
    )
}
