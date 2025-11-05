use abi_stable::export_root_module;
use abi_stable::sabi_extern_fn;
use abi_stable::std_types::RString;
use abi_stable::std_types::RVec;
use abi_stable::prefix_type::PrefixTypeTrait;
use padamo_api::lazy_array_operations::ArrayND;
use padamo_api::lazy_array_operations::LazyDetectorSignal;
use padamo_api::{nodes_vec, prelude::CalculationNodeBox, PadamoModule, PadamoModule_Ref};
use padamo_api::prelude::*;
pub mod nodes;
pub mod nodes_geo;

fn matrix_err<T:Into<RString>>(msg:T)->ExecutionError{
    ExecutionError::OtherError(msg.into())
}

fn get_all(x:LazyDetectorSignal)->ArrayND<f64>{
    x.request_range(0,x.length())
}

#[export_root_module]
pub fn plugin_root()->PadamoModule_Ref{
    PadamoModule{nodes}.leak_into_prefix()
}

#[sabi_extern_fn]
pub fn nodes(_library_dir:RString)->RVec<CalculationNodeBox>{
    nodes_vec!(
        nodes::IdentityNode,
        nodes::PositionNode,
        nodes::RotationNode::new("Rotate YZ", "yz", nalgebra::Vector3::new(1.0, 0.0, 0.0)),
        nodes::RotationNode::new("Rotate XZ", "xz", nalgebra::Vector3::new(0.0, 1.0, 0.0)),
        nodes::RotationNode::new("Rotate XY", "xy", nalgebra::Vector3::new(0.0, 0.0, 1.0)),
        nodes::TransformParentNode,
        nodes::ModelViewNode,

        nodes_geo::WGS84PositionNode,
        nodes_geo::DetectorRotatorNode,
    )
}
