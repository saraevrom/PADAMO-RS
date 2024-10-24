use std::{fs::File, io::Read, path::PathBuf};

use serde::{Serialize,Deserialize};

fn rewrap_pathbuf(path_opt:Option<PathBuf>)->Option<String>{
        if let Some(filename) =  path_opt{
            if let Ok(s) = filename.into_os_string().into_string(){
                Some(s)
            }
            else{
                None
            }
        }
        else{
            None
        }
    }

#[derive(Serialize,Deserialize,Clone)]
pub struct PadamoWorkspace{
    path:Option<String>
}


impl PadamoWorkspace{
    pub fn new(path:Option<String>)->Self{
        Self{path}
    }

    pub fn initialize()->Self{
        if let Some(opened) = Self::open(){
            opened
        }
        else{
            Self::create()
        }
    }

    pub fn open()->Option<Self>{
        let current_exe = std::env::current_exe().unwrap();
        let current_dir = current_exe.parent().unwrap();
        let json_path = current_dir.join("workspace.json");
        if let Ok(mut f) = File::open(json_path){
            let mut buf:String = String::new();
            if let Ok(_) = f.read_to_string(&mut buf){
                if let Ok(r) = serde_json::from_str(&buf){
                    Some(r)
                }
                else{
                    None
                }
            }
            else{
                None
            }
        }
        else{
            None
        }
    }

    pub fn save(&self){
        let current_exe = std::env::current_exe().unwrap();
        let current_dir = current_exe.parent().unwrap();
        let json_path = current_dir.join("workspace.json");
        std::fs::write(json_path, serde_json::to_string(self).unwrap()).unwrap();
    }

    pub fn create()->Self{
        let mut res = Self::new(None);
        res.recreate();
        res
    }

    pub fn workspace<'a>(&'a self, subdir:&'a str)->PadamoSubWorkspace<'a>{
        PadamoSubWorkspace{
            workspace:self,
            subdir,
            default_operations:Vec::new()
        }
    }

    pub fn recreate(&mut self){
        let path = if let Some(s) = &self.path {Some(s.as_str())} else {None};
        let mut dialog = rfd::FileDialog::new();
        if let Some(p) = path{
            dialog = dialog.set_directory(p);
        }
        let sres = dialog.pick_folder();

        self.path = rewrap_pathbuf(sres);
        self.save();


        // if let Ok(pres) = nfd::open_pick_folder(path){
        //     if let nfd::Response::Okay(p) = pres{
        //         self.path = Some(p);
        //     }
        //     else{
        //         self.path = None;
        //     };
        //     self.save();
        // }
    }
}


pub struct PadamoSubWorkspace<'a>{
    workspace:&'a PadamoWorkspace,
    subdir:&'a str,
    default_operations:Vec<Box<dyn Fn(&PathBuf)->()>>
}

pub type FilenameFilter = Vec<(&'static str, Vec<&'static str>)>;

impl<'a> PadamoSubWorkspace<'a>{
    pub fn subdir(&self)->Option<PathBuf>{
        if let Some(p) = &self.workspace.path{
            let p = std::path::Path::new(p).join(self.subdir);
            let creation = std::fs::create_dir(&p);
            if let Ok(()) = creation{
                println!("Created dir {:?}",p.to_str());
            }
            for action in self.default_operations.iter(){
                action(&p)
            }
            Some(p)
        }
        else{
            None
        }
    }

    fn make_dialog(&self, filter_list: FilenameFilter)->rfd::FileDialog{
        let mut dialog = rfd::FileDialog::new();
        let p = self.subdir();
        if let Some(pb) = &p{
            dialog = dialog.set_directory(pb);
        }
        for (name, extensions) in filter_list{
            dialog = dialog.add_filter(name,&extensions);
        }
        dialog
    }

    pub fn with_action<T:Fn(&PathBuf)->()+'static>(mut self, action:T)->Self{
        self.default_operations.push(Box::new(action));
        self
    }

    pub fn save_dialog(&self, filter_list: FilenameFilter) ->  Option<String>{
        let dialog = self.make_dialog(filter_list);
        rewrap_pathbuf(dialog.save_file())

        //nfd::open_save_dialog(filter_list,initd)
    }

    pub fn open_dialog(&self, filter_list: FilenameFilter) ->  Option<String>{
        let dialog = self.make_dialog(filter_list);
        rewrap_pathbuf(dialog.pick_file())
    }
}
