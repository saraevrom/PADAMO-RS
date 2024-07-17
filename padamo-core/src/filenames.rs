use abi_stable::rvec;
use abi_stable::std_types::ROption::RSome;
use abi_stable::std_types::{RString, RVec, RResult};
use padamo_api::{prelude::*, constants};
use padamo_api::{ports,nodes_vec};
use crate::TD_Opaque;


use std::path::Path;

#[derive(Clone,Debug)]
pub struct FileSplit;

impl FileSplit{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>{
        let src:String = inputs.request_string("File path")?.into();
        let fp = Path::new(&src);
        let basename = fp.file_name();
        let dirname= fp.parent();
        if let (Some(b), Some(d)) = (basename, dirname){
            if let (Some(bs), Some(ds)) = (b.to_str(), d.to_str()){
                let bn:String = bs.into();
                let dn:String = ds.into();
                outputs.set_value("Basename",bn.into())?;
                outputs.set_value("Dirname",dn.into())?;
                Ok(())
            }
            else{
                Err(ExecutionError::OtherError("Could not split file path (to_str)".into()))
            }
        }
        else{
            Err(ExecutionError::OtherError("Could not split file path".into()))
        }

    }
}

impl CalculationNode for FileSplit{
    fn name(&self,) -> RString{
        "Split file path".into()
    }

    fn category(&self,) -> RVec<RString> {
        rvec![
            "File path manipulation".into()
        ]
    }

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
        RSome("File path manipulation/Split file path".into())
    }

    fn identifier(&self,) -> RString where {
        "padamocore.file_path.split".into()
    }

    fn is_primary(&self,) -> bool {
        false
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("File path", ContentType::String)
        )
    }

    fn outputs(&self,) -> RVec<CalculationIO>{
        ports!(
            ("Dirname", ContentType::String),
            ("Basename", ContentType::String)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!()
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError>{
        self.calculate(inputs, outputs, constants, environment).into()
    }
}

pub fn nodes()->RVec<CalculationNodeBox>{
    nodes_vec![
        FileSplit,
        FileMerge
    ]

}



#[derive(Clone,Debug)]
pub struct FileMerge;

impl FileMerge{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>{
        let f1:String = inputs.request_string("Path 1")?.into();
        let f2:String = inputs.request_string("Path 2")?.into();
        let fp1 = Path::new(&f1);
        let fp2 = Path::new(&f2);
        let res:String = fp1.join(fp2).into_os_string().into_string().map_err(|x| ExecutionError::OtherError(format!("Could not convert {:?} into path",x).into()))?;
        outputs.set_value("Path", res.into())?;
        Ok(())
    }
}

impl CalculationNode for FileMerge{
    fn name(&self,) -> RString{
        "Merge file path".into()
    }

    fn category(&self,) -> RVec<RString> {
        rvec![
            "File path manipulation".into()
        ]
    }

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
        RSome("File path manipulation/Merge file path".into())
    }

    fn identifier(&self,) -> RString where {
        "padamocore.file_path.merge".into()
        //format!("padamocore.constant.{}",idmark).into()
    }

    fn is_primary(&self,) -> bool {
        false
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Path 1", ContentType::String),
            ("Path 2", ContentType::String)
        )
    }

    fn outputs(&self,) -> RVec<CalculationIO>{
        ports!(
            ("Path", ContentType::String)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!()
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError>{
        self.calculate(inputs, outputs, constants, environment).into()
    }
}
