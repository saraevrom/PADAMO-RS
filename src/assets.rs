use std::path::PathBuf;

pub fn get_assets_dir()->std::path::PathBuf{
    let current_exe = std::env::current_exe().unwrap();
    let current_dir = current_exe.parent().unwrap();
    let json_path = current_dir.join("assets");
    json_path
    //std::fs::write(json_path, serde_json::to_string(self).unwrap()).unwrap();
}

pub fn get_asset(asset_name:&str)->std::path::PathBuf{
    let dir = get_assets_dir();
    dir.join(asset_name)
}

pub fn copy_asset_action(src:&str) -> impl Fn(&PathBuf){
    let asset = get_asset(src);
    let src_name:String = src.into();
    move |dst_dir:&PathBuf|{
        if let Err(e) = std::fs::copy(asset.clone(), dst_dir.join(src_name.clone())){
            println!("Copy error: {}", e);
        }
        else{
            println!("{} -> {}",asset.to_str().unwrap(), dst_dir.to_str().unwrap());
        }
    }.into()
}
