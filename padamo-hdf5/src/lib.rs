use abi_stable::std_types::RString;
use padamo_api::prelude::*;
use abi_stable::{std_types::RVec, export_root_module, prefix_type::PrefixTypeTrait};
use padamo_api::nodes_vec;
use abi_stable::sabi_extern_fn;

pub mod compat;
pub mod nodes_mod;
pub mod ops;

#[export_root_module]
pub fn plugin_root()->PadamoModule_Ref{
    PadamoModule{nodes}.leak_into_prefix()
}

#[sabi_extern_fn]
pub fn nodes(_library_dir:RString)->RVec<CalculationNodeBox>{
    nodes_vec!(
        // crate::nodes_mod::LazyHDF5SignalNode,
        padamo_api::SignalTimeEmbeddedMergingNode::new(crate::nodes_mod::LazyHDF5ArrayNode,crate::nodes_mod::LazyHDF5TimeNode,
                                                       "HDF5 Signal node", "padamohdf5.file_reader"),
        crate::nodes_mod::LazyHDF5DirSignalNode,
        crate::nodes_mod::LazyHDF5ArrayNode,
        crate::nodes_mod::LazyHDF5TimeNode,
        crate::nodes_mod::SaveHDF5Node,
        crate::nodes_mod::SaveHDF5ArrayNode
    )
}
