


//Checking before working with libraries

use std::error::Error;

// Code from ort crate
fn dylib_path() -> String {
    match std::env::var("ORT_DYLIB_PATH") {
        Ok(s) if !s.is_empty() => s,
        #[cfg(target_os = "windows")]
        _ => "onnxruntime.dll".to_owned(),
        #[cfg(any(target_os = "linux", target_os = "android"))]
        _ => "libonnxruntime.so".to_owned(),
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        _ => "libonnxruntime.dylib".to_owned()
    }
}


fn lib_handle() ->Result<libloading::Library,Box<dyn Error>> {
    // resolve path relative to executable
    let path: std::path::PathBuf = dylib_path().into();
    let absolute_path = if path.is_absolute() {
        path
    } else {
        let relative = std::env::current_exe()?
            .parent()
            .expect("executable is root?")
            .join(&path);
        if relative.exists() { relative } else { path }
    };
    let lib = unsafe { libloading::Library::new(&absolute_path) }?;
    Ok(lib)
}

pub fn check_dylib() -> bool{
    let h = lib_handle();
    match h{
        Ok(lib)=>{
            unsafe{
                let base_getter: Result<libloading::Symbol<unsafe extern "C" fn() -> *const ort_sys::OrtApiBase>,_> = lib
                            .get(b"OrtGetApiBase");
                match base_getter {
                    Ok(_v)=>{
                        println!("Check passed");
                        true
                    }
                    Err(e)=>{
                        println!("Symbol error: {}",e);
                        false
                    }
                }
            }

        }
        Err(e)=>{
            println!("Library load error: {}", e);
            false
        },
    }
}
