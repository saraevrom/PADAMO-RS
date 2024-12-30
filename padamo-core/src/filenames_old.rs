use abi_stable::rvec;
use abi_stable::std_types::ROption::RSome;
use abi_stable::std_types::{RString, RVec, RResult};
use padamo_api::{prelude::*, constants};
use padamo_api::{ports,nodes_vec};


use std::path::Path;


pub fn nodes()->RVec<CalculationNodeBox>{
    nodes_vec![
        //FileSplit,
        FileMergeOld
    ]

}



#[derive(Clone,Debug)]
pub struct FileMergeOld;

impl FileMergeOld{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let f1:String = args.inputs.request_string("Path 1")?.into();
        let f2:String = args.inputs.request_string("Path 2")?.into();
        let fp1 = Path::new(&f1);
        let fp2 = Path::new(&f2);
        let res:String = fp1.join(fp2).into_os_string().into_string().map_err(|x| ExecutionError::OtherError(format!("Could not convert {:?} into path",x).into()))?;
        args.outputs.set_value("Path", res.into())?;
        Ok(())
    }
}

impl CalculationNode for FileMergeOld{
    fn name(&self,) -> RString{
        "Merge file path (Legacy)".into()
    }

    fn category(&self,) -> RVec<RString> {
        rvec![
            "Legacy".into(),
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

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>{
        self.calculate(args).into()
    }
}
