use abi_stable::std_types::RResult;
use padamo_api::prelude::*;
use padamo_api::{ports,constants};
use abi_stable::std_types::{RVec,RString, ROption};
use abi_stable::rvec;
use super::ops::*;
use abi_stable::sabi_trait::prelude::TD_Opaque;
use padamo_api::lazy_array_operations::{LazyArrayOperationBox, LazyTriSignal};

#[derive(Clone,Debug)]
pub struct PixelThresholdTriggerNode;

impl PixelThresholdTriggerNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>{
        let mut src = inputs.request_detectorfulldata("Signal")?;
        let thresh = constants.request_float("Threshold")?;
        let source = src.0.clone();
        let boxed = LazyArrayOperationBox::from_value(LazyPixelThresholdTrigger::new(source,thresh),TD_Opaque);

        src.2 = ROption::RSome(boxed);

        outputs.set_value("Signal", src.into())?;
        Ok(())
    }
}

impl CalculationNode for PixelThresholdTriggerNode{
    fn name(&self) -> RString { "Pixel threshold node".into() }

    fn category(&self,) -> RVec<RString>where {
        rvec!["Base triggers".into()]
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!(
            ("Threshold", 100.0)
        )
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}



#[derive(Clone,Debug)]
pub struct LCThresholdTriggerNode;

impl LCThresholdTriggerNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>{
        let mut src = inputs.request_detectorfulldata("Signal")?;
        let thresh = constants.request_float("Threshold")?;
        let source = src.0.clone();
        let boxed = LazyArrayOperationBox::from_value(LazyLCThresholdTrigger::new(source,thresh),TD_Opaque);

        src.2 = ROption::RSome(boxed);

        outputs.set_value("Signal", src.into())?;
        Ok(())
    }
}

impl CalculationNode for LCThresholdTriggerNode{
    fn name(&self) -> RString { "Lightcurve threshold node".into() }

    fn category(&self,) -> RVec<RString>where {
        rvec!["Base triggers".into()]
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!(
            ("Threshold", 100.0)
        )
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}
