use abi_stable::std_types::RString;
use padamo_api::prelude::*;
use abi_stable::{std_types::RVec, export_root_module, prefix_type::PrefixTypeTrait};
use padamo_api::nodes_vec;
use abi_stable::sabi_extern_fn;
use abi_stable::sabi_trait::prelude::TD_Opaque;
use padamo_api::make_node_box;

pub mod output;
pub mod constants;
pub mod filenames;
pub mod filenames_old;
pub mod trigger_ops;
pub mod trigger_nodes;
pub mod boolconv;
pub mod strings;
pub mod strings_old;

#[export_root_module]
pub fn plugin_root()->PadamoModule_Ref{
    PadamoModule{nodes}.leak_into_prefix()
}

#[sabi_extern_fn]
pub fn nodes(_library_dir:RString)->RVec<CalculationNodeBox>{
    let mut node_list: RVec<CalculationNodeBox> = RVec::new();
    node_list.extend(output::nodes());
    node_list.extend(constants::nodes());
    node_list.extend(filenames::nodes());
    node_list.extend(filenames_old::nodes());
    node_list.extend(strings::nodes());
    node_list.extend(strings_old::nodes());
    node_list.push(make_node_box(trigger_nodes::TriggerExpandNode));
    node_list
}
