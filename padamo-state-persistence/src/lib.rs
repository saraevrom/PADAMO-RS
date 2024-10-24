use std::path::{Path, PathBuf};

pub mod data;

#[derive(Clone,Debug)]
pub struct PersistentState{
    pub state_dir:PathBuf
}


impl Default for PersistentState{
    fn default() -> Self {
        Self::new(data::get_state_dir())
    }
}


impl PersistentState{
    pub fn new(state_dir:PathBuf)->Self{
        Self { state_dir }
    }

    fn get_path(&self, key:&str)->PathBuf{
        let mut filename = key.to_owned();
        filename.push_str(".json");
        self.state_dir.join(filename)
    }

    pub fn write(&self, key:&str, data:&str){
        let path = self.get_path(key);
        if let Ok(_) = std::fs::write(path, data){
            return;
        }
        println!("Error writing state");
    }

    pub fn read(&self, key: &str)->Option<String>{
        let path = self.get_path(key);
        if let Ok(content) = std::fs::read_to_string(path){
            Some(content)
        }
        else{
            None
        }
    }

    pub fn serialize<T:serde::Serialize>(&self, key:&str, data:&T){
        if let Ok(v) = serde_json::to_string(data){
            self.write(key, &v);
        }
        println!("Error serializing state");
    }

    pub fn deserialize<T:serde::de::DeserializeOwned>(&self, key:&str) -> Option<T>{
        if let Some(content) = self.read(key){
            if let Ok(data) = serde_json::from_str(&content){
                return Some(data);
            }
        }
        None
    }
}
