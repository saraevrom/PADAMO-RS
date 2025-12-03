use abi_stable::std_types::RString;
use ort::execution_providers::{CPUExecutionProvider, CUDAExecutionProvider};
use padamo_api::prelude::*;
use abi_stable::{std_types::RVec, export_root_module, prefix_type::PrefixTypeTrait};
use padamo_api::nodes_vec;
use abi_stable::sabi_extern_fn;

pub mod nodes;
pub mod ops;
mod ort_check;
pub mod config;

#[export_root_module]
pub fn plugin_root()->PadamoModule_Ref{
    PadamoModule{nodes}.leak_into_prefix()
}

#[sabi_extern_fn]
pub fn nodes(library_dir:RString)->RVec<CalculationNodeBox>{
    let mut res = nodes_vec!();
    let config_path = format!("{}/config.toml", library_dir);

    let config:config::PadamoANNConfig = if let Ok(f) = std::fs::read_to_string(&config_path){
        let subconf = toml::from_str(&f);
        match subconf {
            Ok(v)=>v,
            Err(e)=>{
                eprintln!("Could not read ANN plugin config: {}", e);
                return res;
            }
        }
    }
    else{
        let res = Default::default();
        let s = toml::to_string(&res).unwrap();
        if let Err(e) = std::fs::write(&config_path, s){
            eprintln!("Could not write new config file for ANN plugin: {}", e);
        }
        else{
            eprintln!("Created new ANN plugin configuration file");
        }
        res
    };
    if !config.enabled{
        return res;
    }

    if !ort_check::check_dylib(){
        return res;
    }

    let mut providers = vec![];
    if config.use_cuda_provider{
        providers.push(CUDAExecutionProvider::default().build());
    }

    if config.use_cpu_provider{
        providers.push(CPUExecutionProvider::default().build());
    }

    if ort::init()
        .with_execution_providers(providers)
        .commit()
    {
        for ann in config.networks{
            ann.insert_node(&library_dir, &mut res);
        }
        // match nodes::ANN3DNode::new("ANN trigger Model A", &format!("{}/model_A.onnx",library_dir), (128,16,16), "concatenate".into(),
        //                             "model_a","ANN trigger Model A"){
        //     Ok(v)=>res.push(make_node_box(v)),
        //     Err(e)=>println!("{}",e),
        // }
        // match nodes::ANN3DNode::new("ANN trigger Model L2 (multiconv)", &format!("{}/model_L2.onnx",library_dir), (128,8,8), "flatten_1".into(),
        //                             "model_l2","ANN trigger Model L2 (multiconv)"){
        //     Ok(v)=>res.push(make_node_box(v)),
        //     Err(e)=>println!("{}",e),
        // }
        // match nodes::ANN3DNode::new("ANN trigger Model TE1", &format!("{}/model_te1.onnx",library_dir), (256,8,8), "Identity:0".into(),
        //                             "model_te1","ANN trigger Model TE1"){
        //     Ok(v)=>res.push(make_node_box(v)),
        //     Err(e)=>println!("{}",e),
        // }
    }
    res
}
