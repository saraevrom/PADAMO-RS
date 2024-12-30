use padamo_api::prelude::*;
use abi_stable::{rvec, std_types::ROption::RSome};
use padamo_api::ports;
use abi_stable::std_types::RVec;

#[derive(Clone,Debug)]
pub struct ViewerNode;

pub const VIEWER_FILENAME_VAR:&'static str = "ViewerOpenedFile";
pub const VIEWER_SIGNAL_VAR:&'static str = "ViewerSignal";
pub const VIEWER_MASK_VAR:&'static str = "alive_pixels";

impl ViewerNode{
    fn calculate(&self,args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let signal = args.inputs.request_detectorfulldata("Signal")?;
        args.environment.0.insert(VIEWER_SIGNAL_VAR.into(),Content::DetectorFullData(signal));
        Ok(())
    }
}

impl CalculationNode for ViewerNode{
    fn name(&self,) -> abi_stable::std_types::RString where {
        "View".into()
    }

    fn category(&self,) -> abi_stable::std_types::RVec<abi_stable::std_types::RString>where {
        rvec![
            "Application".into(),
            "Viewer".into()
        ]
    }

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<abi_stable::std_types::RString>where {
        RSome("Application/Viewer/View".into())
    }

    fn identifier(&self,) -> abi_stable::std_types::RString where {
        "builtin.viewer.view".into()
    }

    fn is_primary(&self,) -> bool where {
        true
    }

    fn inputs(&self) -> RVec<CalculationIO>{
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    fn outputs(&self) -> RVec<CalculationIO>{
        ports!()
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        rvec![]
    }

    fn calculate(&self,args:CalculationNodeArguments) -> abi_stable::std_types::RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}

#[derive(Clone,Debug)]
pub struct LoadedFileNode;

impl LoadedFileNode{
    fn calculate(&self,args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let filename = args.environment.request_string(VIEWER_FILENAME_VAR).unwrap_or("file.h5".into());
        args.outputs.set_value("File path".into(), filename.into())?;
        Ok(())
    }
}

impl CalculationNode for LoadedFileNode{
    fn name(&self,) -> abi_stable::std_types::RString{
        "Opened file".into()
    }

    fn category(&self,) -> abi_stable::std_types::RVec<abi_stable::std_types::RString>where {
        rvec![
            "Application".into(),
            "Viewer".into()
        ]
    }

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<abi_stable::std_types::RString>where {
        RSome("Application/Viewer/Opened file".into())
    }

    fn identifier(&self,) -> abi_stable::std_types::RString where {
        "builtin.viewer.opened_file".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!()
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("File path", ContentType::String)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        rvec![]
    }

    fn calculate(&self,args:CalculationNodeArguments) -> abi_stable::std_types::RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}

#[derive(Clone,Debug)]
pub struct ViewerMaskNode;

impl ViewerMaskNode{
    fn calculate(&self,args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let mask= args.environment.request_detectorsignal(VIEWER_MASK_VAR)?;
        args.outputs.set_value("Alive pixels".into(), mask.into())?;
        Ok(())
    }
}

impl CalculationNode for ViewerMaskNode{
    fn name(&self,) -> abi_stable::std_types::RString{
        "Detector mask".into()
    }

    fn category(&self,) -> abi_stable::std_types::RVec<abi_stable::std_types::RString>where {
        rvec![
            "Application".into(),
            "Viewer".into()
        ]
    }

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<abi_stable::std_types::RString>where {
        RSome("Application/Viewer/Detector mask".into())
    }

    fn identifier(&self,) -> abi_stable::std_types::RString where {
        "builtin.viewer.detector_mask".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!()
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Alive pixels", ContentType::DetectorSignal)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        rvec![]
    }

    fn calculate(&self,args:CalculationNodeArguments) -> abi_stable::std_types::RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}
