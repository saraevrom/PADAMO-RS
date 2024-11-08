pub mod ops;
pub mod ops_stretch;
pub mod node_reg;
pub mod nodes_stretch;

use abi_stable::prefix_type::PrefixTypeTrait;
use padamo_api::prelude::*;
use abi_stable::std_types::{RString, RVec};
use abi_stable::{sabi_extern_fn, export_root_module};
use padamo_api::nodes_vec;

#[sabi_extern_fn]
pub fn nodes(_library_dir:RString)->RVec<CalculationNodeBox>{
    nodes_vec!(
        node_reg::TimeResolutionReduceNode,
        node_reg::CutterNode,
        nodes_stretch::StretchSignal,
    )
}

#[export_root_module]
pub fn plugin_root()->PadamoModule_Ref{
    PadamoModule{nodes}.leak_into_prefix()
}
