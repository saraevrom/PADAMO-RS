use abi_stable::std_types::ROption::RSome;
use abi_stable::std_types::RResult;
use padamo_api::prelude::*;
use padamo_api::{ports,constants};
use abi_stable::std_types::{RVec,RString};
use abi_stable::rvec;
use super::ops::*;
use abi_stable::sabi_trait::prelude::TD_Opaque;
use padamo_api::lazy_array_operations::{LazyArrayOperationBox, LazyTriSignal};

#[derive(Clone,Debug)]
pub struct SlidingQuantileNode;

fn check_quantile(q:f64)->Result<(),ExecutionError>{
    if q<0.0 || q>1.0{
        Err(ExecutionError::OtherError(format!("Invalid quantile {}",q).into()))
    }
    else{
        Ok(())
    }
}

impl SlidingQuantileNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let window = args.constants.request_integer("Sliding window")? as usize;
        let quantile = args.constants.request_float("Quantile")?;
        check_quantile(quantile)?;
        let source = args.inputs.request_detectorfulldata("Signal")?;
        let trigger = source.2.map(|x| LazyArrayOperationBox::from_value(LazySkipper::new(x, window), TD_Opaque));

        if source.0.length()<window{
            return Err(ExecutionError::OtherError("Signal is too small".into()));
        }

        let bg = LazySlidingQuantile::new(source.0.clone(), window, quantile);
        let bg = LazyArrayOperationBox::from_value(bg, TD_Opaque);
        let bg = crate::padding::make_padding(bg, window/2, window-window/2-1);
        let bg = bg.cached();

        //let cut_signal = LazySkipper::new(source.0, window);
        let cut_signal = source.0.clone();
        let cut_signal = LazyArrayOperationBox::from_value(cut_signal, TD_Opaque);
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

impl CalculationNode for SlidingQuantileNode {
    #[doc = " Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString where {
        "Sliding quantile".into()
    }

    fn category(&self,) -> RVec<RString>where {
        rvec!["Data Processing".into()]
    }

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
        RSome("Data Processing/Sliding quantile".into())
    }

    fn identifier(&self,) -> RString where {
        "padamobasesignalprocessing.sliding_quantile".into()
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
            ("Sliding window", 64),
            ("Quantile", 0.5)
        )
    }

    #[doc = " Main calculation"]
    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}




#[derive(Clone,Debug)]
pub struct SlidingQuantileNodeNormalizer;

impl SlidingQuantileNodeNormalizer{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let window = args.constants.request_integer("Sliding window")? as usize;
        let quantile = args.constants.request_float("Quantile")?;
        check_quantile(quantile)?;
        let source = args.inputs.request_detectorfulldata("Signal")?;
        let trigger = source.2.map(|x| LazyArrayOperationBox::from_value(LazySkipper::new(x, window), TD_Opaque));
        let gauss = args.constants.request_boolean("Gauss mode")?;
        let variance = args.constants.request_boolean("Use Variance")?;

        if source.0.length()<window{
            return Err(ExecutionError::OtherError("Signal is too small".into()));
        }

        let norm = LazySlidingQuantileNormalize::new(source.0.clone(), window, quantile,gauss, variance);
        let norm = crate::padding::make_padding(make_lao_box(norm), window/2, window-window/2-1);
        let norm = norm.cached();


        let time = source.1;//LazyArrayOperationBox::from_value(LazySkipper::new(source.1, window), TD_Opaque);
        let trisignal:LazyTriSignal = (norm,time,trigger).into();
        //
        args.outputs.set_value("Normalized", Content::DetectorFullData(trisignal))?;
        Ok(())
    }
}

impl CalculationNode for SlidingQuantileNodeNormalizer {
    #[doc = " Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString where {
        "Sliding quantile normalize".into()
    }

    fn category(&self,) -> RVec<RString>where {
        rvec!["Data Processing".into()]
    }

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
        RSome("Data Processing/Sliding quantile normalize".into())
    }

    fn identifier(&self,) -> RString where {
        "padamobasesignalprocessing.sliding_quantile_normalize".into()
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
            ("Quantile", 0.5),
            ("Gauss mode", true),
            ("Use Variance", false)
        )
    }

    #[doc = " Main calculation"]
    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}

#[derive(Clone,Debug)]
pub struct LazyFlashSuppression;

impl LazyFlashSuppression{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let src = args.inputs.request_detectorfulldata("Signal")?;
        let q = args.constants.request_float("Quantile")?;
        check_quantile(q)?;
        let newsignal = LazyFlashSuppress::new(src.0, q);
        let newsignal = LazyArrayOperationBox::from_value(newsignal, TD_Opaque);
        let tgt = (newsignal,src.1,src.2).into();
        args.outputs.set_value("Signal", Content::DetectorFullData(tgt))?;
        Ok(())
    }
}

impl CalculationNode for LazyFlashSuppression{
    fn name(&self,) -> RString where {
        "Flash suppression".into()
    }

    fn category(&self,) -> RVec<RString>where {
        rvec!["Data Processing".into()]
    }

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
        RSome("Data Processing/Flash suppression".into())
    }

    fn identifier(&self,) -> RString where {
        "padamobasesignalprocessing.flash_suppression".into()
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
            ("Quantile", 0.5)
        )
    }

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}


#[derive(Clone,Debug)]
pub struct LazyThresholdNode;

impl LazyThresholdNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let mut src = args.inputs.request_detectorfulldata("Signal")?;
        let threshold_value = args.constants.request_float("threshold_value")?;
        let blank_value = args.constants.request_float("blank_value")?;
        let invert = args.constants.request_boolean("invert")?;
        src.0 = make_lao_box(LazyThreshold{
            source: src.0,
            threshold_value,
            blank_value,
            invert,
        });
        args.outputs.set_value("Signal", Content::DetectorFullData(src))
    }
}

impl CalculationNode for LazyThresholdNode{
    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString where {
        "Threshold replace".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn category(&self,) -> RVec<RString>where {
        rvec!["Data Processing".into()]
    }

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
        RSome("Data Processing/Threshold replace".into())
    }

    fn identifier(&self,) -> RString where {
        "padamobasesignalprocessing.threshold_replace".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Output definition of node"]
    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Constants definition of node with default values."]
    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!(
            ("threshold_value",0.0),
            ("blank_value",0.0),
            ("invert",false)
        )
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}
