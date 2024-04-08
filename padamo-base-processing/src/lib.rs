pub mod ops;
pub mod ops_median;
pub mod moving_median;
pub mod node_reg;
pub mod node_reg_mm;
//pub mod fast_mm;
use abi_stable::prefix_type::PrefixTypeTrait;
use padamo_api::prelude::*;
use abi_stable::std_types::RVec;
use abi_stable::{sabi_extern_fn, export_root_module};
use padamo_api::nodes_vec;
use abi_stable::sabi_trait::prelude::TD_Opaque;

#[sabi_extern_fn]
pub fn nodes()->RVec<CalculationNodeBox>{
    nodes_vec!(
        node_reg_mm::SlidingMedianNode,
        node_reg_mm::SlidingMedianNodeNormalizer,
        node_reg::SlidingQuantileNode,
        node_reg::SlidingQuantileNodeNormalizer,
        node_reg::LazyFlashSuppression
    )
}

#[export_root_module]
pub fn plugin_root()->PadamoModule_Ref{
    PadamoModule{nodes}.leak_into_prefix()
}
