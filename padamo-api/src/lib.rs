pub mod calculation_nodes;
pub mod prelude;
pub mod lazy_array_operations;
//pub mod parsers;
pub mod function_operator;

use abi_stable::sabi_trait::TD_Opaque;
use abi_stable::std_types::{RString, RVec};
use abi_stable::{StableAbi, library::RootModule, package_version_strings, sabi_types::VersionStrings, declare_root_module_statics};
use crate::prelude::*;

pub fn make_node_box<T:CalculationNode+'static>(x:T)->CalculationNodeBox{
    CalculationNode_TO::from_value(x,TD_Opaque)
}

#[macro_export]
macro_rules! ports {
    ($( $x:expr ),*) => {
        {
            let mut temp_vec: RVec<CalculationIO> = RVec::new();
            $(
                temp_vec.push($x.into());
            )*
            temp_vec
        }

    };
}

#[macro_export]
macro_rules! constants {
    ($( $x:expr ),*) => {
        {
            let mut temp_vec: RVec<CalculationConstant> = RVec::new();
            $(
                temp_vec.push($x.into());
            )*

            temp_vec
        }
    };
}

#[macro_export]
macro_rules! nodes_vec {
    ($( $x:expr ),*) => {
        {
            let mut temp_vec: RVec<CalculationNodeBox> = RVec::new();
            $(
                temp_vec.push(CalculationNode_TO::from_value($x,TD_Opaque));
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
