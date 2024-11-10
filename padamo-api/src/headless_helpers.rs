use std::{collections::HashMap, fs, path::Path};

use abi_stable::library::{lib_header_from_path, RootModule};

use crate::{CalculationNodeBox, PadamoModule_Ref};

fn register_nodes<T:AsRef<Path>>(nodes:&mut HashMap<String,CalculationNodeBox>, seekdir:T, look_in_directories:bool)->anyhow::Result<()>{
    // println!("Seeking for plugins in {}", seekdir.to_str().unwrap());
    let paths = fs::read_dir(seekdir).unwrap();
    for path in paths{
        if let Ok(p_res) = path{
            let p = p_res.path();
            if p.is_dir() && look_in_directories{
                println!("Looking into dir: {:?}",p);

                //NO FULL RECURSIVE SEARCH. Only first layer.
                register_nodes(nodes, &p, false)?;
            }
            else if p.is_file(){

                    let parent = p.parent().ok_or(anyhow::Error::msg("No parent of path"))?;
                    let parent = parent.to_str().ok_or(anyhow::Error::msg("Bad file name"))?;
                    let plugin_f = (||{
                        let header = lib_header_from_path(&p)?;
                        header.init_root_module::<PadamoModule_Ref>()
                    });
                    if let Ok(plugin) = plugin_f(){
                        let nodes_fn = plugin.nodes();
                        let mut add_nodes = nodes_fn(parent.into());
                        add_nodes.drain(..).for_each(|x| {nodes.insert(x.identifier().into(), x);});
                    }


            }
            // if let Err(e) = nodes.load_lib(p.as_path()){
            //     println!("Error reading library: {}",e);
            // }
            else {
                println!("Skipped: {:?}",p);
            }
        }
    }
    Ok(())
}

pub fn load_nodes<T:AsRef<Path>>(seekdir:T)->anyhow::Result<HashMap<String,CalculationNodeBox>>{
    let mut nodes = HashMap::new();
    register_nodes(&mut nodes, seekdir.as_ref(), true)?;
    Ok(nodes)
}


