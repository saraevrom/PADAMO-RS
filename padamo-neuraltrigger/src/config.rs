use abi_stable::std_types::RVec;
use padamo_api::{make_node_box, prelude::CalculationNodeBox};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
pub struct PadamoANNConfig{
    pub enabled:bool,
    pub use_cuda_provider:bool,
    pub use_cpu_provider:bool,
    pub networks: Vec<PadamoModelConfig>,
}

impl Default for PadamoANNConfig{
    fn default() -> Self {
        Self {
            enabled: true,
            use_cuda_provider: true,
            use_cpu_provider: true,
            networks: vec![
                PadamoModelConfig::new("ANN trigger Model A", "model_A.onnx", (128,16,16), "concatenate", "model_a", Some("ANN trigger Model A")),
                PadamoModelConfig::new("ANN trigger Model L2 (multiconv)", "model_L2.onnx", (128,8,8), "flatten_1", "model_l2", None),
                PadamoModelConfig::new("ANN trigger Model TE1", "model_te1.onnx", (256,8,8), "Identity:0", "model_te1", None),
            ]
        }
    }
}


#[derive(Serialize, Deserialize)]
pub struct PadamoModelConfig{
    pub node_name:String,
    pub filename:String,
    pub size_hint:(usize,usize,usize),
    pub output_layer:String,
    pub node_id:String,
    pub old_name:Option<String>,

}

impl PadamoModelConfig {
    pub fn new<T1,T2,T3,T4>(node_name: T1, filename: T2, size_hint: (usize,usize,usize), output_layer: T3, node_id: T4, old_name: Option<&str>) -> Self
    where
        T1: Into<String>,
        T2: Into<String>,
        T3: Into<String>,
        T4: Into<String>,
    {
        let old_name = old_name.map(Into::into);
        Self { node_name: node_name.into(), filename:filename.into(), size_hint, output_layer:output_layer.into(), node_id:node_id.into(), old_name }
    }

    pub fn insert_node(&self, library_dir:&str, nodes:&mut RVec<CalculationNodeBox>){
        match crate::nodes::ANN3DNode::new(&self.node_name, &format!("{}/{}",library_dir,self.filename), self.size_hint, (&self.output_layer).into(),
                                    &self.node_id,self.old_name.clone()){
            Ok(v)=>nodes.push(make_node_box(v)),
            Err(e)=>println!("{}",e),
        }
    }
}
