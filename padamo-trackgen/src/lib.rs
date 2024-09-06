use abi_stable::std_types::RString;
use padamo_api::prelude::*;
use abi_stable::{std_types::RVec, export_root_module, prefix_type::PrefixTypeTrait};
use padamo_api::nodes_vec;
use abi_stable::sabi_extern_fn;
use abi_stable::sabi_trait::prelude::TD_Opaque;

pub mod ensquared_energy;
pub mod lightcurves;
pub mod tracks_2d;
pub mod legacy;
pub mod background;

mod shape_parser;

#[export_root_module]
pub fn plugin_root()->PadamoModule_Ref{
    PadamoModule{nodes}.leak_into_prefix()
}

#[sabi_extern_fn]
pub fn nodes(_library_dir:RString)->RVec<CalculationNodeBox>{
    let mut res = nodes_vec!();

    res.extend(tracks_2d::nodes());
    res.extend(background::nodes());
    res.extend(lightcurves::nodes());
    res.extend(legacy::nodes());

    res
}
