use std::{error::Error, sync::Arc};

use abi_stable::{rvec, std_types::{ROption::RSome, RResult, RString, RVec}};
use ort::Session;
use padamo_api::{constants, ports, prelude::*};

#[derive(Clone,Debug)]
pub struct ANN3DNode{
    name:String,
    id:String,
    old_name:String,
    //ann_model_path:String,
    ann_model:Arc<ort::Session>,
    size_hint:(usize,usize,usize),
    output_layer:String,
}

fn request_usize(constants:&ConstantContentContainer,key:&str)->Result<usize,ExecutionError>{
    let i = constants.request_integer("Stride")?;
    if i<0{
        Err(ExecutionError::OtherError(format!("Constant {} must be nonnegative", key).into()))
    }
    else{
        Ok(i as usize)
    }
}


impl ANN3DNode {
    pub fn new(name: &str, ann_model_path: &str, size_hint: (usize,usize,usize), output_layer:String, id:&str,old_name:&str) -> Result<Self,Box<dyn Error>> {
        let ann_model = Arc::new(Session::builder()?.commit_from_file(ann_model_path)?);
        Ok(Self { name:name.into(), ann_model, size_hint, output_layer, id:id.to_owned(), old_name:old_name.to_owned() })
    }

    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let stride = request_usize(&args.constants,"Stride")?;
        let threshold = args.constants.request_float("Threshold")? as f32;
        let squeeze = args.constants.request_boolean("Squeeze")?;
        let mut signal = args.inputs.request_detectorfulldata("Signal")?;

        signal.0 = crate::ops::LazyANNTrigger3D::align_data(signal.0, stride, self.size_hint).map_err(ExecutionError::from_error)?;
        signal.1 = crate::ops::LazyANNTrigger3D::align_data(signal.1, stride, self.size_hint).map_err(ExecutionError::from_error)?;

        signal.2 = RSome(make_lao_box(
            crate::ops::LazyANNTrigger3D::new(self.ann_model.clone(), signal.0.clone(), threshold, stride, self.size_hint,
                                              self.output_layer.clone(),squeeze).map_err(ExecutionError::from_dyn_error)?
        ));

        args.outputs.set_value("Signal",signal.into())?;

        Ok(())
    }
}

impl CalculationNode for ANN3DNode{
    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString where {
        self.name.clone().into()
    }

    fn category(&self,) -> RVec<RString>{
        rvec!["ANN 3D triggers".into()]
    }

    fn identifier(&self,) -> RString where {
        format!("padamoneuraltrigger.{}",self.id).into()
    }

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
        RSome(format!("ANN 3D triggers/{}",self.old_name).into())
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Signal",ContentType::DetectorFullData)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Output definition of node"]
    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Signal",ContentType::DetectorFullData)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Constants definition of node with default values."]
    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![
            ("Threshold",0.5),
            ("Stride",1),
            ("Squeeze", false),
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}
