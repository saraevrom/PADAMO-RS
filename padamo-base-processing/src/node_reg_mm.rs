use abi_stable::std_types::ROption::RSome;
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
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let window = args.constants.request_integer("Sliding window")? as usize;

        let source = args.inputs.request_detectorfulldata("Signal")?;
        let trigger = source.2.map(|x| LazyArrayOperationBox::from_value(LazySkipper::new(x, window), TD_Opaque));

        if source.0.length()<window{
            return Err(ExecutionError::OtherError("Signal is too small".into()));
        }

        let bg = LazySlidingMedian::new(source.0.clone(), window);
        //let bg = LazyArrayOperationBox::from_value(bg, TD_Opaque);
        let bg = crate::padding::make_padding(make_lao_box(bg), window/2, window-window/2-1);
        let bg = bg.cached();

        let cut_signal = make_lao_box(source.0);//LazySkipper::new(source.0, window);
        //let cut_signal = LazyArrayOperationBox::from_value(cut_signal, TD_Opaque);
        let detail = LazySubtractor::new(cut_signal, bg.clone());
        let detail = LazyArrayOperationBox::from_value(detail, TD_Opaque);
        let detail = detail.cached();

        let time = source.1;//LazyArrayOperationBox::from_value(LazySkipper::new(source.1, window), TD_Opaque);
        let trisignal:LazyTriSignal = (detail,time.clone(),trigger.clone()).into();
        let bg_out:LazyTriSignal = (bg,time,trigger).into();
        //
        args.outputs.set_value("Detail", Content::DetectorFullData(trisignal))?;
        args.outputs.set_value("Background", Content::DetectorFullData(bg_out))?;
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

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
        RSome("Data Processing/Sliding median".into())
    }

    fn identifier(&self,) -> RString where {
        "padamobasesignalprocessing.sliding_median".into()
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
    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}




#[derive(Clone,Debug)]
pub struct SlidingMedianNodeNormalizer;

impl SlidingMedianNodeNormalizer{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let window = args.constants.request_integer("Sliding window")? as usize;
        let source = args.inputs.request_detectorfulldata("Signal")?;
        let trigger = source.2.map(|x| LazyArrayOperationBox::from_value(LazySkipper::new(x, window), TD_Opaque));
        let gauss = args.constants.request_boolean("Gauss mode")?;
        let variance = args.constants.request_boolean("Use Variance")?;

        if source.0.length()<window{
            return Err(ExecutionError::OtherError("Signal is too small".into()));
        }

        let norm = LazySlidingMedianNormalize::new(source.0.clone(), window, gauss, variance);
        let norm = make_lao_box(norm);
        let norm = crate::padding::make_padding(norm, window/2, window-window/2-1);
        let norm = norm.cached();


        let time = source.1;//LazyArrayOperationBox::from_value(LazySkipper::new(source.1, window), TD_Opaque);
        let trisignal:LazyTriSignal = (norm,time,trigger).into();
        //
        args.outputs.set_value("Normalized", Content::DetectorFullData(trisignal))?;
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

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
        RSome("Data Processing/Sliding median normalize".into())
    }

    fn identifier(&self,) -> RString where {
        "padamobasesignalprocessing.sliding_median_normalize".into()
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
            ("Gauss mode", true),
            ("Use Variance", false)
        )
    }

    #[doc = " Main calculation"]
    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}
