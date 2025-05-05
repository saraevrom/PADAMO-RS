use abi_stable::std_types::RString;
use padamo_api::prelude::*;
use abi_stable::{std_types::RVec, export_root_module, prefix_type::PrefixTypeTrait};
use padamo_api::nodes_vec;
use padamo_api::SignalTimeEmbeddedMergingNode;
use abi_stable::sabi_extern_fn;

pub mod scalable_array;
pub mod ops;
pub mod nodes;
pub mod nodes_legacy;

#[export_root_module]
pub fn plugin_root()->PadamoModule_Ref{
    PadamoModule{nodes}.leak_into_prefix()
}

#[sabi_extern_fn]
pub fn nodes(_library_dir:RString)->RVec<CalculationNodeBox>{
    nodes_vec!(
        crate::nodes_legacy::EUSOROOTNode,
        crate::nodes::EUSOROOTArrayNode,
        crate::nodes::EUSOROOTTimeNode,
        SignalTimeEmbeddedMergingNode::new(crate::nodes::EUSOROOTArrayNode, crate::nodes::EUSOROOTTimeNode, "ROOT-ROOT signal reader", "padamoroot.root-root"),
        SignalTimeEmbeddedMergingNode::new(crate::nodes::EUSOROOTArrayNode, pseudotime::nodes::PseudoTime, "ROOT-Pseudotime signal reader", "padamoroot.root-pseudo"),
    )
}
