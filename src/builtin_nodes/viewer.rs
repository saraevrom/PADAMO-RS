use padamo_api::prelude::*;
use abi_stable::rvec;
use padamo_api::ports;
use abi_stable::std_types::RVec;

#[derive(Clone,Debug)]
pub struct ViewerNode;

pub const VIEWER_FILENAME_VAR:&'static str = "ViewerOpenedFile";
pub const VIEWER_SIGNAL_VAR:&'static str = "ViewerSignal";

impl ViewerNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>{
        let signal = inputs.request_detectorfulldata("Signal")?;
        environment.0.insert(VIEWER_SIGNAL_VAR.into(),Content::DetectorFullData(signal));
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

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> abi_stable::std_types::RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}

#[derive(Clone,Debug)]
pub struct LoadedFileNode;

impl LoadedFileNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>{
        let filename = environment.request_string(VIEWER_FILENAME_VAR)?;
        outputs.set_value("File path".into(), filename.into())?;
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

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> abi_stable::std_types::RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}
