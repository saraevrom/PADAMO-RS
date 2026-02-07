use std::path::PathBuf;

use abi_stable::std_types::RString;
use ort::execution_providers::{CPUExecutionProvider, CUDAExecutionProvider};
use padamo_api::prelude::*;
use abi_stable::{std_types::RVec, export_root_module, prefix_type::PrefixTypeTrait};
use padamo_api::nodes_vec;
use abi_stable::sabi_extern_fn;

use crate::setup_onnx::PadamoONNXOutcome;

pub mod nodes;
pub mod ops;
// mod ort_check;
mod setup_onnx;
mod downloader;
pub mod config;

#[export_root_module]
pub fn plugin_root()->PadamoModule_Ref{
    PadamoModule{nodes}.leak_into_prefix()
}

#[sabi_extern_fn]
pub fn nodes(library_dir:RString)->RVec<CalculationNodeBox>{
    let mut res = nodes_vec!();
    let config_path = format!("{}/config.toml", library_dir);

    let mut config:config::PadamoANNConfig = if let Ok(f) = std::fs::read_to_string(&config_path){
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
        let res = config::PadamoANNConfig::default();
        res.attempt_save(&config_path);
        res
    };
    if !config.enabled{
        return res;
    }

    // if !ort_check::check_dylib(){
    //     return res;
    // }
    // let onnx_path = if let Some(s) = config.onnx_dir{
    //     s
    // }
    // else{
    //     let new_onnx = if let Some(s1) = setup_onnx::get_onnx(){
    //         s1
    //     }
    //     else{
    //
    //     };
    //
    //     new_onnx
    // };
    if config.force_onnx_setup || config.onnx_dir.is_none(){
        match setup_onnx::get_onnx(PathBuf::from(library_dir.as_str())){
            PadamoONNXOutcome::Ok(v)=>{
                config.onnx_dir = v.to_str().map(|x| x.to_string());
            }
            PadamoONNXOutcome::Failure=>{},
            PadamoONNXOutcome::Disable=>{
                config.enabled=false;
                config.onnx_dir=None;
                config.attempt_save(&config_path);
                return res;
            }
        }
    }

    let mut onnx_dir:PathBuf = if let Some(v) = &config.onnx_dir{
        PathBuf::from(v)
    }
    else{
        println!("No ONNX runtime is set up. Skipping.");
        return res;
    };

    if onnx_dir.is_relative(){
        onnx_dir = PathBuf::from(library_dir.as_str()).join(onnx_dir);
    }

    config.force_onnx_setup=true;
    config.attempt_save(&config_path);

    let mut providers = vec![];
    if config.use_cuda_provider{
        providers.push(CUDAExecutionProvider::default().build());
    }

    if config.use_cpu_provider{
        providers.push(CPUExecutionProvider::default().build());
    }

    let ort_init = match ort::init_from(onnx_dir){
        Ok(v)=>v,
        Err(e)=>{
            println!("{}",e);
            return res;
        }
    };

    if ort_init
        .with_execution_providers(providers)
        .with_telemetry(false)
        .commit()
    {
        for ann in &config.networks{
            ann.insert_node(&library_dir, &mut res);
        }
    }
    config.force_onnx_setup=false;
    config.attempt_save(&config_path);
    res
}
