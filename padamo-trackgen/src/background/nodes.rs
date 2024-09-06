use std::default;

use abi_stable::std_types::ROption::RSome;
use padamo_api::lazy_array_operations::LazyTriSignal;
use padamo_api::{constants, ports, prelude::*};
use abi_stable::std_types::{ROption, RResult, RString, RVec, Tuple3};
use abi_stable::rvec;
use serde_json::value;

use crate::ensquared_energy::detector;

#[derive(Debug,Clone)]
pub struct AnyLCLinearTrackGeneratorDynamicGaussNode;

#[derive(Debug,Clone)]
pub struct AnyLCLinearTrackGeneratorDynamicMoffatNode;

fn request_nonnegative(name:&str,constants:&ConstantContentContainer)->Result<f64,ExecutionError>{
    let value = constants.request_float(name)?;
    if value>=0.0{
        Ok(value)
    }
    else {
        Err(ExecutionError::OtherError(format!("Value {} must be nonnegative",name).into()))
    }
}

fn request_nonnegative_input(name:&str,constants:&ContentContainer)->Result<f64,ExecutionError>{
    let value = constants.request_float(name)?;
    if value>=0.0{
        Ok(value)
    }
    else {
        Err(ExecutionError::OtherError(format!("Value {} must be nonnegative",name).into()))
    }
}

fn request_usize(name:&str,constants:&ConstantContentContainer)->Result<usize,ExecutionError>{
    let value = constants.request_integer(name)?;
    if let Ok(v) = usize::try_from(value){
        Ok(v)
    }
    else {
        Err(ExecutionError::OtherError(format!("Value {} must be nonnegative integer (usize)",name).into()))
    }
}


#[derive(Clone, Debug)]
pub struct BlankDataNode;

impl BlankDataNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError> {
        let length = request_usize("length",&constants)?;
        let time_offset = constants.request_float("time_offset")?;
        let time_step = constants.request_float("time_step")?;

        //let shape = constants.request_string("shape")?;
        //let shape = crate::shape_parser::parse_usize_vec(&shape).ok_or_else(|| ExecutionError::OtherError(format!("Cannot parse shape {}",&shape).into()))?;
        let detector_content = environment.request_string("detector")?.to_string();
        let detector: padamo_detectors::polygon::DetectorContent = serde_json::from_str(&detector_content).map_err(|x| ExecutionError::OtherError(format!("{:?}",x).into()))?;
        let shape = detector.compat_shape.clone();

        let temporal = super::ops::ArtificialTime::new(length, time_offset,time_step);
        let spatial = super::ops::ArtificialBlankSignal::new(length, shape);

        let signal:LazyTriSignal = (make_lao_box(spatial),make_lao_box(temporal),ROption::RNone).into();

        outputs.set_value("Signal", signal.into())?;

        Ok(())
    }
}

impl CalculationNode for BlankDataNode{
    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString  {
        "Blank data".into()
    }

    fn category(&self,) -> RVec<RString> {
        rvec!["Artificial data".into()]
    }

    fn old_identifier(&self,) -> ROption<RString>where {
        RSome("Artificial data/Blank data".into())
    }

    fn identifier(&self,) -> RString where {
        "padamotrackgen.blank_data".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self,) -> RVec<CalculationIO> {
        rvec![]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Output definition of node"]
    fn outputs(&self,) -> RVec<CalculationIO> {
        ports![
            ("Signal", ContentType::DetectorFullData)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Constants definition of node with default values."]
    fn constants(&self,) -> RVec<CalculationConstant> {
        constants![
            ("length",100),
            ("time_offset",0.0),
            ("time_step",1.0),
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError> {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}


#[derive(Clone, Debug)]
pub struct AdditiveNormalNoiseNode;

impl AdditiveNormalNoiseNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>{
        let sigma = constants.request_float("sigma")?;
        if sigma<=0.0{
            return Err(ExecutionError::OtherError("Standard deviation must be positive".into()));
        }
        let seed = constants.request_integer("seed")?;
        let mut signal = inputs.request_detectorfulldata("Background")?;
        signal.2 = ROption::RNone;
        signal.0 = make_lao_box(super::ops::LazyAdditiveNormalNoise::new(signal.0, seed, sigma));
        outputs.set_value("Signal", signal.into())
    }
}

impl CalculationNode for AdditiveNormalNoiseNode{
    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self) -> RString  {
        "Additive normal noise".into()
    }

    fn category(&self) -> RVec<RString> {
        rvec!["Artificial data".into()]
    }

    fn identifier(&self,) -> RString where {
        "padamotrackgen.gaussian_noise2".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self) -> RVec<CalculationIO> {
        ports![
            ("Background", ContentType::DetectorFullData),

        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Output definition of node"]
    fn outputs(&self) -> RVec<CalculationIO> {
        ports![
            ("Signal", ContentType::DetectorFullData)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Constants definition of node with default values."]
    fn constants(&self) -> RVec<CalculationConstant> {
        constants![
            ("seed", 0),
            ("sigma", 1.0)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError> {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}
