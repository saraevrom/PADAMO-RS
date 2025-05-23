pub mod calculation_nodes;
pub mod prelude;
pub mod lazy_array_operations;
//pub mod parsers;
pub mod function_operator;
pub mod common_categories;
pub mod rng;

#[cfg(feature = "headless")]
pub mod headless_helpers;

pub use calculation_nodes::ad_hoc_input_node::AdHocInputNode;
pub use calculation_nodes::ad_hoc_output_node::AdHocOutputNode;
pub use calculation_nodes::signal_time_node::SignalTimeEmbeddedMergingNode;

use abi_stable::sabi_trait::TD_Opaque;
use abi_stable::std_types::{RString, RVec};
use abi_stable::{StableAbi, library::RootModule, package_version_strings, sabi_types::VersionStrings, declare_root_module_statics};
use crate::prelude::*;


pub fn make_node_box<T:CalculationNode+'static>(x:T)->CalculationNodeBox{
    CalculationNode_TO::from_value(x,TD_Opaque)
}

#[macro_export]
macro_rules! ports {
    ($( $x:expr ),* $(,)?) => {
        {
            #[allow(unused_mut)]
            let mut temp_vec: abi_stable::std_types::RVec<CalculationIO> = abi_stable::std_types::RVec::new();
            $(
                temp_vec.push($x.into());
            )*
            temp_vec
        }

    };
}

#[macro_export]
macro_rules! constants {
    ($( $x:expr ),* $(,)?) => {
        {
            #[allow(unused_mut)]
            let mut temp_vec: abi_stable::std_types::RVec<CalculationConstant> = abi_stable::std_types::RVec::new();
            $(
                temp_vec.push($x.into());
            )*

            temp_vec
        }
    };
}

#[macro_export]
macro_rules! nodes_vec {
    ($( $x:expr ),* $(,)?) => {
        {
            #[allow(unused_mut)]
            let mut temp_vec: abi_stable::std_types::RVec<CalculationNodeBox> = abi_stable::std_types::RVec::new();
            $(
                temp_vec.push(CalculationNode_TO::from_value($x,abi_stable::sabi_trait::TD_Opaque));
            )*

            temp_vec
        }
    };
}



// #[abi_stable::sabi_trait]
// pub trait PadamoModuleState{
//     fn nodes(&self)->CalculationNodeBox;
// }

//pub type PadamoModuleStateBox = PadamoModuleState_TO<'static, RBox<()>>;



#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix))]
pub struct PadamoModule{
    //pub new: extern "C" fn() -> PadamoModuleStateBox,

    #[sabi(last_prefix_field)]
    pub nodes: extern "C" fn(RString) -> RVec<CalculationNodeBox>,
    //
    //pub indicate: extern "C" fn(&mut State),
}

impl RootModule for PadamoModule_Ref{
    const BASE_NAME: &'static str = "padamo_module";
    // The name of the library for logging and similars
    const NAME: &'static str = "padamo_module";
    // The version of this plugin's crate
    const VERSION_STRINGS: VersionStrings = package_version_strings!();

    // Implements the `RootModule::root_module_statics` function, which is the
    // only required implementation for the `RootModule` trait.
    declare_root_module_statics!{PadamoModule_Ref}
}
