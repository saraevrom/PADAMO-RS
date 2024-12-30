use abi_stable::rvec;
use abi_stable::std_types::ROption::RSome;
use abi_stable::std_types::{RString, RVec, RResult};
use padamo_api::{prelude::*, constants};
use padamo_api::{ports,nodes_vec};


use std::path::Path;

#[derive(Clone,Debug)]
pub struct FileSplit;

impl FileSplit{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let src:String = args.inputs.request_string("File path")?.into();
        let fp = Path::new(&src);
        let basename = fp.file_name();
        let dirname= fp.parent();
        if let (Some(b), Some(d)) = (basename, dirname){
            if let (Some(bs), Some(ds)) = (b.to_str(), d.to_str()){
                let bn:String = bs.into();
                let dn:String = ds.into();
                args.outputs.set_value("Basename",bn.into())?;
                args.outputs.set_value("Dirname",dn.into())?;
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

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>{
        self.calculate(args).into()
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
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let f1:String = args.constants.request_string("Path 1")?.into();
        let f2:String = args.constants.request_string("Path 2")?.into();
        let fp1 = Path::new(&f1);
        let fp2 = Path::new(&f2);
        let res:String = fp1.join(fp2).into_os_string().into_string().map_err(|x| ExecutionError::OtherError(format!("Could not convert {:?} into path",x).into()))?;
        args.outputs.set_value("Path", res.into())?;
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

    fn identifier(&self,) -> RString where {
        "padamocore.file_path.merge2".into()
        //format!("padamocore.constant.{}",idmark).into()
    }

    fn is_primary(&self,) -> bool {
        false
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            // ("Path 1", ContentType::String),
            // ("Path 2", ContentType::String)
        )
    }

    fn outputs(&self,) -> RVec<CalculationIO>{
        ports!(
            ("Path", ContentType::String)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![
            ("Path 1", ""),
            ("Path 2", "")
        ]
    }

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>{
        self.calculate(args).into()
    }
}
