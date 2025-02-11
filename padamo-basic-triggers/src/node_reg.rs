use abi_stable::std_types::ROption::RSome;
use abi_stable::std_types::RResult;
use padamo_api::prelude::*;
use padamo_api::{ports,constants};
use abi_stable::std_types::{RVec,RString, ROption};
use abi_stable::rvec;
use super::ops::*;
use abi_stable::sabi_trait::prelude::TD_Opaque;
use padamo_api::lazy_array_operations::LazyArrayOperationBox;

#[derive(Clone,Debug)]
pub struct PixelThresholdTriggerNode;

impl PixelThresholdTriggerNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let mut src = args.inputs.request_detectorfulldata("Signal")?;
        let thresh = args.constants.request_float("Threshold")?;
        let source = src.0.clone();
        let boxed = LazyArrayOperationBox::from_value(LazyPixelThresholdTrigger::new(source,thresh),TD_Opaque);

        src.2 = ROption::RSome(boxed);

        args.outputs.set_value("Signal", src.into())?;
        Ok(())
    }
}

impl CalculationNode for PixelThresholdTriggerNode{
    fn name(&self) -> RString { "Pixel threshold node".into() }

    fn category(&self,) -> RVec<RString>where {
        rvec!["Base triggers".into()]
    }

    fn old_identifier(&self,) -> ROption<RString>where {
        RSome("Base triggers/Pixel threshold node".into())
    }

    fn identifier(&self,) -> RString where {
        "padamobasictriggers.pixel_threshold_trigger".into()
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

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}



#[derive(Clone,Debug)]
pub struct LCThresholdTriggerNode;

impl LCThresholdTriggerNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let mut src = args.inputs.request_detectorfulldata("Signal")?;
        let thresh = args.constants.request_float("Threshold")?;
        let source = src.0.clone();
        let boxed = LazyArrayOperationBox::from_value(LazyLCThresholdTrigger::new(source,thresh),TD_Opaque);

        src.2 = ROption::RSome(boxed);

        args.outputs.set_value("Signal", src.into())?;
        Ok(())
    }
}

impl CalculationNode for LCThresholdTriggerNode{
    fn name(&self) -> RString { "Lightcurve threshold node".into() }

    fn category(&self,) -> RVec<RString>where {
        rvec!["Base triggers".into()]
    }

    fn old_identifier(&self,) -> ROption<RString>where {
        RSome("Base triggers/Lightcurve threshold node".into())
    }

    fn identifier(&self,) -> RString where {
        "padamobasictriggers.lightcurve_threshold_trigger".into()
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

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}


#[derive(Clone,Debug)]
pub struct MedianThresholdTriggerNode;

impl MedianThresholdTriggerNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let mut src = args.inputs.request_detectorfulldata("Signal")?;
        let thresh = args.constants.request_float("Threshold")?;
        let source = src.0.clone();
        let boxed = LazyArrayOperationBox::from_value(LazyMedianTrigger::new(source,thresh),TD_Opaque);

        src.2 = ROption::RSome(boxed);

        args.outputs.set_value("Signal", src.into())?;
        Ok(())
    }
}

impl CalculationNode for MedianThresholdTriggerNode{
    fn name(&self) -> RString { "Median threshold node".into() }

    fn category(&self,) -> RVec<RString>where {
        rvec!["Base triggers".into()]
    }

    fn old_identifier(&self,) -> ROption<RString>where {
        RSome("Base triggers/Median threshold node".into())
    }

    fn identifier(&self,) -> RString where {
        "padamobasictriggers.median_threshold_trigger".into()
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

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}
