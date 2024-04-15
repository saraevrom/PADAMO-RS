use padamo_api::prelude::*;
use abi_stable::{std_types::RVec, export_root_module, prefix_type::PrefixTypeTrait};
use padamo_api::nodes_vec;
use abi_stable::sabi_extern_fn;
use abi_stable::sabi_trait::prelude::TD_Opaque;

pub mod nodes;

use nodes::*;

#[export_root_module]
pub fn plugin_root()->PadamoModule_Ref{
    PadamoModule{nodes}.leak_into_prefix()
}

#[sabi_extern_fn]
pub fn nodes()->RVec<CalculationNodeBox>{
    nodes_vec!(
        ConstantNode,
        LinearNode,
        SquareNode,
        SumNode,
        MultiplyNode,
        ExponentNode,
        LogNode,
        FCalculateNode
    )
}
