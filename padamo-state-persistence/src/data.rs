/// Gets directory for state json files
pub fn get_state_dir()->std::path::PathBuf{
    let current_exe = std::env::current_exe().unwrap();
    let current_dir = current_exe.parent().unwrap();
    let state = current_dir.join("state");
    let creation = std::fs::create_dir(&state);
    if let Ok(()) = creation{
        println!("Created dir {:?}",state.to_str());
    }
    state
}


