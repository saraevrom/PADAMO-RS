use abi_stable::std_types::RString;
use padamo_api::prelude::*;
use abi_stable::{std_types::RVec, export_root_module, prefix_type::PrefixTypeTrait};
use abi_stable::sabi_extern_fn;
use padamo_api::make_node_box;

pub mod output;
pub mod constants;
pub mod filenames;
pub mod filenames_old;

// New event-based triggers have some issues with boolean operators
pub mod trigger_ops;
pub mod trigger_nodes;

pub mod boolconv;
pub mod strings;
pub mod strings_old;
pub mod cache;
pub mod io_nodes;
pub mod temporal;
pub mod remapper;

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
    node_list.extend(trigger_nodes::nodes());
    node_list.extend(io_nodes::nodes());
    node_list.extend(temporal::nodes());
    // node_list.push(make_node_box(trigger_nodes::TriggerExpandNode));
    // node_list.push(make_node_box(trigger_nodes::TriggerExchangeNode));
    node_list.push(make_node_box(cache::ForcedCacheNode));
    node_list.push(make_node_box(remapper::nodes::RemapperNode));
    node_list
}
