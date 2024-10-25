use abi_stable::std_types::RString;
use padamo_api::prelude::*;
use abi_stable::{std_types::RVec, export_root_module, prefix_type::PrefixTypeTrait};
use padamo_api::nodes_vec;
use abi_stable::sabi_extern_fn;

pub mod nodes;
// pub mod ops;

#[export_root_module]
pub fn plugin_root()->PadamoModule_Ref{
    PadamoModule{nodes}.leak_into_prefix()
}

#[sabi_extern_fn]
pub fn nodes(_library_dir:RString)->RVec<CalculationNodeBox>{
    nodes_vec!(
        crate::nodes::UniformRandomNode,
        crate::nodes::UUIDRandomNode,
        crate::nodes::RandomIntNode,
        crate::nodes::RandomIntRangeNode
    )
}
