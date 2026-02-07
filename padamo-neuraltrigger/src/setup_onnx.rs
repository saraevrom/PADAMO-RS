use std::{io::Write, path::{Path, PathBuf}};

pub enum PadamoONNXOutcome{
    Ok(PathBuf),
    Failure,
    Disable,
}

fn read_line()->String{
    std::io::stdout().flush().unwrap();
    let mut res = String::new();
    std::io::stdin().read_line(&mut res).unwrap();
    res.trim().to_string()
}

pub fn ask(prompt:&str)->String{
    println!("{}",prompt);
    print!("> ");
    read_line()
}

#[allow(dead_code)]
fn check_answer(response:&str, default_answer:Option<bool>)->Option<bool>{
    let lower = response.to_lowercase();
    match lower.as_str(){
        "" => default_answer,
        "0" | "n" => Some(false),
        "1" | "y" => Some(true),
        _=>None
    }
}

#[allow(dead_code)]
pub fn ask_yesno(prompt:&str, default_answer:Option<bool>)->bool{
    let prompt = match default_answer{
        Some(true) => format!("{} (Y/n) ",prompt),
        Some(false) => format!("{} (y/N) ",prompt),
        None => format!("{} (y/n)",prompt),
    };
    print!("{}",prompt);
    loop{
        let asked = read_line();
        if let Some(v) = check_answer(&asked, default_answer){
            return v;
        }
    }
}

pub fn ask_options(prompt:&str, answers:Vec<&str>)->usize{
    println!("{}", prompt);
    for (i, opt) in answers.iter().enumerate(){
        println!("{}: {}", i+1, opt);
    }
    loop{
        print!("> ");
        let answer = read_line();
        if let Ok(v) = answer.parse::<usize>(){
            if v>=1 && v<=answers.len(){
                return v-1;
            }
        }
        else{
            println!("Wrong option. Choose 1-{}",answers.len());
        }
    }
}



cfg_match::cfg_match! {
    #[cfg(target_os = "linux")]
    fn is_onnxruntime(path:&Path)->bool{
        if let Some(p) = path.file_name().map(|x| x.to_str()).flatten(){
            return p.starts_with("libonnxruntime.so"); // On GNU/Linux file can be named as libonnxruntime.so.1 or whatever. therefore
        }
        false
    }

    #[cfg(target_os = "windows")]
    fn is_onnxruntime(path:&Path)->bool{
        if let Some(".dll") = path.extension().map(|x| x.to_str()).flatten(){
            if let Some(p) = path.file_name().map(|x| x.to_str()).flatten(){
                return p.starts_with("onnxruntime");
            }
        }
        false
    }

    #[cfg(_)]
    fn is_onnxruntime(path:&Path)->bool{
        false
    }
}

// cfg_match::cfg_match! {
//     #[cfg(target_os = "linux")]
//     fn suggest_onnx(){
//         println!("Suggesting ONNX versions for linux:");
//         println!("Official microsoft: https://github.com/microsoft/onnxruntime/releases/download/v1.23.2/onnxruntime-linux-x64-gpu-1.23.2.tgz");
//         println!("Official pykeio (creator of 'ort' crate used in project. Seems to be broken): https://cdn.pyke.io/0/pyke:ort-rs/ms@1.23.2/x86_64-unknown-linux-gnu+cu12.tar.lzma2");
//     }
//
//     #[cfg(target_os = "windows")]
//     fn suggest_onnx(){
//         println!("Suggesting ONNX versions for windows:");
//         println!("Official microsoft (Warning: official builds have telemetry enabled.): https://github.com/microsoft/onnxruntime/releases/download/v1.23.2/onnxruntime-win-x64-gpu-1.23.2.zip");
//     }
//
//     #[cfg(_)]
//     fn suggest_onnx(){
//
//     }
// }

// pub fn check_onnx_dir(path:&str)->bool{
//     let path = PathBuf::from(path);
//     match std::fs::read_dir(path){
//         Ok(v)=>{
//             for entry_res in v{
//                 match entry_res{
//                     Ok(entry)=>{
//                         if is_onnxruntime(&entry.path()){
//                             return true;
//                         }
//                     }
//                     Err(e)=>{
//                         println!("{}", e);
//                     }
//                 }
//             }
//         }
//         Err(e)=>{
//             println!("{}", e);
//         }
//     }
//     false
// }

fn scan_onnx(dir: &Path) -> std::io::Result<Option<PathBuf>> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let scan_res = scan_onnx(&path)?;
                if scan_res.is_some(){
                    return Ok(scan_res);
                }
            } else {
                // println!("{:?}",path);
                if is_onnxruntime(&path){
                    println!("Found ONNX runtime {:?}",path);
                    return Ok(Some(path));
                }
            }
        }
    }
    Ok(None)
}

pub fn get_onnx(maindir:PathBuf)->PadamoONNXOutcome{
    let required_onnx = ort::MINOR_VERSION;
    println!("ORT supports ONNX runtime version 1.{}",required_onnx);
    match ask_options("Setting up ONNX runtime. Pick an option", vec!["Download", "Specify installed directory", "Skip initialization."]){
        0=>{
            // let target_file = maindir.join("downloaded_file.bin");
            let target_dir = maindir.join("onnx_runtime");
            let _ = std::fs::remove_dir_all(&target_dir);
            // if let Err(e) = super::downloader::download_onnx(&target_file){
            //     println!("{}",e);
            //     return PadamoONNXOutcome::Failure;
            // }
            // let new_target_file = maindir.join("downloaded_file.")
            let target_file = match super::downloader::download_onnx(&maindir){
                Ok(v) => {v}
                Err(e) => {
                    println!("{}",e);
                    return PadamoONNXOutcome::Failure;
                }
            };

            let mut archive = match arkiv::Archive::open(&target_file){
                Ok(v) => {v}
                Err(e) => {
                    println!("{}",e);
                    return PadamoONNXOutcome::Failure;
                }
            };
            if let Err(e) = archive.unpack(&target_dir){
                println!("{}",e);
                return PadamoONNXOutcome::Failure;
            }

            //Check subdir
            match scan_onnx(&target_dir){
                Ok(None)=>{
                    println!("ONNX runtime was not found")
                }
                Ok(Some(v))=>{
                    println!("DIFF {:?} {:?}",maindir,v);
                    if let Some(v1) = pathdiff::diff_paths(v,maindir){
                        return PadamoONNXOutcome::Ok(v1);
                    }
                }
                Err(e)=>{
                    println!("{}",e);
                }
            }


            PadamoONNXOutcome::Failure
        },
        1=>{
            loop {
                let path = ask("Specify path to ONNX runtime libraries.");
                let path = PathBuf::from(path);
                match scan_onnx(&path){
                    Ok(None)=>{
                        println!("ONNX runtime was not found")
                    }
                    Ok(Some(v))=>{
                        return PadamoONNXOutcome::Ok(v);
                    }
                    Err(e)=>{
                        println!("{}",e);
                    }
                }
            }
        },
        _=>{
            println!("Skipped initialization. PADAMO neural triggers are disabled. ");
            PadamoONNXOutcome::Disable
        }
    }
}
