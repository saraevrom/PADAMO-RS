use abi_stable::std_types::RString;
use padamo_api::prelude::*;
use abi_stable::{std_types::RVec, export_root_module, prefix_type::PrefixTypeTrait};
use abi_stable::sabi_extern_fn;



pub mod stft;
pub mod ops;
pub mod main_node;
pub mod nodes_filters;

#[export_root_module]
pub fn plugin_root()->PadamoModule_Ref{
    PadamoModule{nodes}.leak_into_prefix()
}

#[sabi_extern_fn]
pub fn nodes(_library_dir:RString)->RVec<CalculationNodeBox>{
    let mut node_list: RVec<CalculationNodeBox> = RVec::new();
    node_list.extend(main_node::nodes());
    node_list.extend(nodes_filters::nodes());
    node_list
}
