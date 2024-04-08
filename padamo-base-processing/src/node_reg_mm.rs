use abi_stable::std_types::RResult;
use padamo_api::prelude::*;
use padamo_api::{ports,constants};
use abi_stable::std_types::{RVec,RString};
use abi_stable::rvec;
use super::ops_median::*;
use crate::ops::{LazySkipper,LazySubtractor};
use abi_stable::sabi_trait::prelude::TD_Opaque;
use padamo_api::lazy_array_operations::{LazyArrayOperationBox, LazyTriSignal};

#[derive(Clone,Debug)]
pub struct SlidingMedianNode;


impl SlidingMedianNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>{
        let window = constants.request_integer("Sliding window")? as usize;

        let source = inputs.request_detectorfulldata("Signal")?;
        let trigger = source.2.map(|x| LazyArrayOperationBox::from_value(LazySkipper::new(x, window), TD_Opaque));

        if source.0.length()<window{
            return Err(ExecutionError::OtherError("Signal is too small".into()));
        }

        let bg = LazySlidingMedian::new(source.0.clone(), window);
        let bg = LazyArrayOperationBox::from_value(bg, TD_Opaque);
        let bg = bg.cached();

        let cut_signal = LazySkipper::new(source.0, window);
        let cut_signal = LazyArrayOperationBox::from_value(cut_signal, TD_Opaque);
        let detail = LazySubtractor::new(cut_signal, bg.clone());
        let detail = LazyArrayOperationBox::from_value(detail, TD_Opaque);
        let detail = detail.cached();

        let time = LazyArrayOperationBox::from_value(LazySkipper::new(source.1, window), TD_Opaque);
        let trisignal:LazyTriSignal = (detail,time.clone(),trigger.clone()).into();
        let bg_out:LazyTriSignal = (bg,time,trigger).into();
        //
        outputs.set_value("Detail", Content::DetectorFullData(trisignal))?;
        outputs.set_value("Background", Content::DetectorFullData(bg_out))?;
        Ok(())
    }
}

impl CalculationNode for SlidingMedianNode {
    #[doc = " Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString where {
        "Sliding median".into()
    }

    fn category(&self,) -> RVec<RString>where {
        rvec!["Data Processing".into()]
    }

    #[doc = " Input definitions of node"]
    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    #[doc = " Output definition of node"]
    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Detail", ContentType::DetectorFullData),
            ("Background", ContentType::DetectorFullData)
        )
    }

    #[doc = " Constants definition of node with default values."]
    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!(
            ("Sliding window", 64)
        )
    }

    #[doc = " Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}




#[derive(Clone,Debug)]
pub struct SlidingMedianNodeNormalizer;

impl SlidingMedianNodeNormalizer{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>{
        let window = constants.request_integer("Sliding window")? as usize;
        let source = inputs.request_detectorfulldata("Signal")?;
        let trigger = source.2.map(|x| LazyArrayOperationBox::from_value(LazySkipper::new(x, window), TD_Opaque));
        let gauss = constants.request_boolean("Gauss mode")?;

        if source.0.length()<window{
            return Err(ExecutionError::OtherError("Signal is too small".into()));
        }

        let norm = LazySlidingMedianNormalize::new(source.0.clone(), window, gauss);
        let norm = LazyArrayOperationBox::from_value(norm, TD_Opaque);
        let norm = norm.cached();


        let time = LazyArrayOperationBox::from_value(LazySkipper::new(source.1, window), TD_Opaque);
        let trisignal:LazyTriSignal = (norm,time,trigger).into();
        //
        outputs.set_value("Normalized", Content::DetectorFullData(trisignal))?;
        Ok(())
    }
}

impl CalculationNode for SlidingMedianNodeNormalizer {
    #[doc = " Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString where {
        "Sliding median normalize".into()
    }

    fn category(&self,) -> RVec<RString>where {
        rvec!["Data Processing".into()]
    }

    #[doc = " Input definitions of node"]
    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    #[doc = " Output definition of node"]
    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Normalized", ContentType::DetectorFullData)
        )
    }

    #[doc = " Constants definition of node with default values."]
    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!(
            ("Sliding window", 64),
            ("Gauss mode", true)
        )
    }

    #[doc = " Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}
