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
