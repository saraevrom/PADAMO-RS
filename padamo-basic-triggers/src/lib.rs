pub mod ops;
pub mod node_reg;
use abi_stable::prefix_type::PrefixTypeTrait;
use padamo_api::prelude::*;
use abi_stable::std_types::RVec;
use abi_stable::{sabi_extern_fn, export_root_module};
use padamo_api::nodes_vec;
use abi_stable::sabi_trait::prelude::TD_Opaque;

#[sabi_extern_fn]
pub fn nodes()->RVec<CalculationNodeBox>{
    nodes_vec!(
        node_reg::PixelThresholdTriggerNode,
        node_reg::LCThresholdTriggerNode
    )
}

#[export_root_module]
pub fn plugin_root()->PadamoModule_Ref{
    PadamoModule{nodes}.leak_into_prefix()
}
