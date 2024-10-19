use abi_stable::std_types::RString;
use ort::{CPUExecutionProvider, CUDAExecutionProvider};
use padamo_api::{make_node_box, prelude::*};
use abi_stable::{std_types::RVec, export_root_module, prefix_type::PrefixTypeTrait};
use padamo_api::nodes_vec;
use abi_stable::sabi_extern_fn;

pub mod nodes;
pub mod ops;
mod ort_check;

#[export_root_module]
pub fn plugin_root()->PadamoModule_Ref{
    PadamoModule{nodes}.leak_into_prefix()
}

#[sabi_extern_fn]
pub fn nodes(library_dir:RString)->RVec<CalculationNodeBox>{
    let mut res = nodes_vec!();
    if !ort_check::check_dylib(){
        return res;
    }
    match ort::init()
        .with_execution_providers([CUDAExecutionProvider::default().build(),CPUExecutionProvider::default().build()])
        .commit()
    {
        Ok(_)=>{
            match nodes::ANN3DNode::new("ANN trigger Model A", &format!("{}/model_A.onnx",library_dir), (128,16,16), "concatenate".into(),
                                        "model_a","ANN trigger Model A"){
                Ok(v)=>res.push(make_node_box(v)),
                Err(e)=>println!("{}",e),
            }
            match nodes::ANN3DNode::new("ANN trigger Model L2 (multiconv)", &format!("{}/model_L2.onnx",library_dir), (128,8,8), "flatten_1".into(),
                                        "model_l2","ANN trigger Model L2 (multiconv)"){
                Ok(v)=>res.push(make_node_box(v)),
                Err(e)=>println!("{}",e),
            }
        }
        Err(e)=>println!("{}",e),
    }
    res
}
