use abi_stable::std_types::RString;
use padamo_api::prelude::*;
use abi_stable::{std_types::RVec, export_root_module, prefix_type::PrefixTypeTrait};
use padamo_api::nodes_vec;
use abi_stable::sabi_extern_fn;

pub mod nodes;
pub mod macros;

//pub use macros::*;

use nodes::*;

#[export_root_module]
pub fn plugin_root()->PadamoModule_Ref{
    PadamoModule{nodes}.leak_into_prefix()
}

#[sabi_extern_fn]
pub fn nodes(_library_dir:RString)->RVec<CalculationNodeBox>{
    nodes_vec!(
        ConstantNode,
        ConstantNode2,
        LinearNode,
        SquareNode,
        SumNode,
        MultiplyNode,
        ExponentNode,
        LogNode,
        FCalculateNode,
        FCalculateNode2,
        MinNode,
        MaxNode,
        AbsNode,
        NegNode,
        InvNode,
        LowerStepNode,
        LinearModificationNode
    )
}
