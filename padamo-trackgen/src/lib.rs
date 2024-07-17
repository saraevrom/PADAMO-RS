use abi_stable::std_types::RString;
use padamo_api::prelude::*;
use abi_stable::{std_types::RVec, export_root_module, prefix_type::PrefixTypeTrait};
use padamo_api::nodes_vec;
use abi_stable::sabi_extern_fn;
use abi_stable::sabi_trait::prelude::TD_Opaque;


pub mod ops;
pub mod ensquared_energy;
pub mod nodes;
pub mod lc_nodes;

//mod datetime_parser;
mod shape_parser;

#[export_root_module]
pub fn plugin_root()->PadamoModule_Ref{
    PadamoModule{nodes}.leak_into_prefix()
}

#[sabi_extern_fn]
pub fn nodes(_library_dir:RString)->RVec<CalculationNodeBox>{
    nodes_vec!(
        nodes::BasicLinearTrackGeneratorNode,
        nodes::AnyLCLinearTrackGeneratorNode,
        nodes::AnyLCLinearTrackGeneratorDynamicGaussNode,
        nodes::AnyLCLinearTrackGeneratorDynamicMoffatNode,
        nodes::BlankDataNode,
        nodes::AdditiveNormalNoiseNode,
        lc_nodes::LCSwitchNode,
        lc_nodes::LinearLCNode,
        lc_nodes::ExponentLCNode,
        lc_nodes::LCPivotNode,
        lc_nodes::TerminationLCNode,
        lc_nodes::ConstantLCNode,
        lc_nodes::MultiplyByFloatNode
    )
}
