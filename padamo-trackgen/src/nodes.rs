use std::default;

use abi_stable::std_types::ROption::RSome;
use padamo_api::lazy_array_operations::LazyTriSignal;
use padamo_api::{constants, ports, prelude::*};
use abi_stable::std_types::{ROption, RResult, RString, RVec, Tuple3};
use abi_stable::rvec;
use serde_json::value;

use crate::ensquared_energy::detector;

#[derive(Debug,Clone)]
pub struct BasicLinearTrackGeneratorNode;


#[derive(Debug,Clone)]
pub struct AnyLCLinearTrackGeneratorNode;

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

impl BasicLinearTrackGeneratorNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError> {

        let detector_content = environment.request_string("detector")?.to_string();
        let detector: padamo_detectors::polygon::DetectorContent = serde_json::from_str(&detector_content).map_err(|x| ExecutionError::OtherError(format!("{:?}",x).into()))?;
        let detector = crate::ensquared_energy::detector::wireframe(detector);

        //let detector = crate::ensquared_energy::load_detector(&detector_path).ok_or_else(|| ExecutionError::OtherError("Could not load detector for track generator".into()))?;
        let pivot_frame = request_nonnegative("pivot_frame", &constants)?;
        let attack_frames = request_nonnegative("attack_frames", &constants)?;
        let sustain_frames = request_nonnegative("sustain_frames", &constants)?;
        let decay_frames = request_nonnegative("decay_frames", &constants)?;
        let v0 = request_nonnegative("v0", &constants)?;
        let a0 = constants.request_float("a0")?;
        let x0 = constants.request_float("x0")?;
        let y0 = constants.request_float("y0")?;
        let phi0 = constants.request_float("phi0")?;
        let e_min = constants.request_float("e_min")?;
        let e_max = constants.request_float("e_max")?;
        let sigma_x = constants.request_float("sigma_x")?;
        let sigma_y = constants.request_float("sigma_y")?;
        let motion_blur_steps = request_usize("motion_blur_steps", &constants)?;

        let mut signal = inputs.request_detectorfulldata("Background")?;
        //let signal =
        signal.0 = padamo_api::lazy_array_operations::make_lao_box(crate::ops::LazyTriangularE0Track{
            data: signal.0,
            detector,pivot_frame,attack_frames,sustain_frames,decay_frames,v0,a0,phi0,x0,y0,e_min,e_max,sigma_x,sigma_y,motion_blur_steps
        });
        signal.2 = ROption::RNone;


        outputs.set_value("Signal", signal.into())?;
        Ok(())
    }
}

impl CalculationNode for BasicLinearTrackGeneratorNode{

    #[doc = r" Category to place node in node list"]
    fn category(&self,) -> RVec<RString> {
        rvec!["Artificial data".into()]
    }


    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString  {
        "Basic linear track".into()
    }

    fn old_identifier(&self,) -> ROption<RString>where {
        RSome("Artificial data/Basic linear track".into())
    }

    fn identifier(&self,) -> RString where {
        "padamotrackgen.basic_linear_track".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self) -> RVec<CalculationIO> {
        ports!(
            ("Background", ContentType::DetectorFullData)
        )
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Output definition of node"]
    fn outputs(&self) -> RVec<CalculationIO> {
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Constants definition of node with default values."]
    fn constants(&self,) -> RVec<CalculationConstant> {
        constants![
            ("pivot_frame", 1.0),
            ("attack_frames",1.0),
            ("sustain_frames",1.0),
            ("decay_frames",1.0),
            ("v0",0.01),
            ("a0",0.0),
            ("phi0",0.0),
            ("x0",0.0),
            ("y0",0.0),
            ("e_min",0.0),
            ("e_max",1.0),
            ("sigma_x",1.2),
            ("sigma_y",1.2),
            ("motion_blur_steps",5),
            ("default_length",100)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError> {
        self.calculate(inputs, outputs, constants, environment).into()
    }


}


impl AnyLCLinearTrackGeneratorNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError> {

        let detector_content = environment.request_string("detector")?.to_string();
        let detector: padamo_detectors::polygon::DetectorContent = serde_json::from_str(&detector_content).map_err(|x| ExecutionError::OtherError(format!("{:?}",x).into()))?;
        let detector = crate::ensquared_energy::detector::wireframe(detector);

        //let detector = crate::ensquared_energy::load_detector(&detector_path).ok_or_else(|| ExecutionError::OtherError("Could not load detector for track generator".into()))?;
        let pivot_frame = request_nonnegative("pivot_frame", &constants)?;
        let lc = inputs.request_function("Lightcurve")?;
        let lc = lc.map(|x| if x>0.0 {x} else {0.0});
        let v0 = request_nonnegative("v0", &constants)?;
        let a0 = constants.request_float("a0")?;
        let x0 = constants.request_float("x0")?;
        let y0 = constants.request_float("y0")?;
        let phi0 = constants.request_float("phi0")?;
        //let e_min = constants.request_float("e_min")?;
        //let e_max = constants.request_float("e_max")?;
        let sigma_x = constants.request_float("sigma_x")?;
        let sigma_y = constants.request_float("sigma_y")?;
        let motion_blur_steps = request_usize("motion_blur_steps", &constants)?;

        let mut signal = inputs.request_detectorfulldata("Background")?;
        //let signal =
        signal.0 = padamo_api::lazy_array_operations::make_lao_box(crate::ops::LazyAnyLCGaussTrack{
            data: signal.0,
            lc,
            detector,pivot_frame,v0,a0,phi0,x0,y0,sigma_x,sigma_y,motion_blur_steps
        });
        signal.2 = ROption::RNone;


        outputs.set_value("Signal", signal.into())?;
        Ok(())
    }
}

impl CalculationNode for AnyLCLinearTrackGeneratorNode{

    #[doc = r" Category to place node in node list"]
    fn category(&self,) -> RVec<RString> {
        rvec!["Artificial data".into()]
    }


    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString  {
        "Customizable LC linear track".into()
    }

    fn old_identifier(&self,) -> ROption<RString>where {
        RSome("Artificial data/Customizable LC linear track".into())
    }

    fn identifier(&self,) -> RString where {
        "padamotrackgen.customizable_lc_linear_track".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self) -> RVec<CalculationIO> {
        ports!(
            ("Background", ContentType::DetectorFullData),
            ("Lightcurve", ContentType::Function)
        )
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Output definition of node"]
    fn outputs(&self) -> RVec<CalculationIO> {
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Constants definition of node with default values."]
    fn constants(&self,) -> RVec<CalculationConstant> {
        constants![
            ("pivot_frame", 1.0),
            ("v0",0.01),
            ("a0",0.0),
            ("phi0",0.0),
            ("x0",0.0),
            ("y0",0.0),
            ("sigma_x",1.2),
            ("sigma_y",1.2),
            //("e_min",0.0),
            //("e_max",1.0),
            ("motion_blur_steps",5)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError> {
        self.calculate(inputs, outputs, constants, environment).into()
    }


}



impl AnyLCLinearTrackGeneratorDynamicGaussNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError> {

        let detector_content = environment.request_string("detector")?.to_string();
        let detector: padamo_detectors::polygon::DetectorContent = serde_json::from_str(&detector_content).map_err(|x| ExecutionError::OtherError(format!("{:?}",x).into()))?;
        let detector = crate::ensquared_energy::detector::wireframe(detector);

        //let detector = crate::ensquared_energy::load_detector(&detector_path).ok_or_else(|| ExecutionError::OtherError("Could not load detector for track generator".into()))?;
        let pivot_frame = request_nonnegative_input("pivot_frame", &inputs)?;
        let lc = inputs.request_function("Lightcurve")?;
        let lc = lc.map(|x| if x>0.0 {x} else {0.0});
        let v0 = request_nonnegative_input("v0", &inputs)?;
        let a0 = inputs.request_float("a0")?;
        let x0 = inputs.request_float("x0")?;
        let y0 = inputs.request_float("y0")?;
        let phi0 = inputs.request_float("phi0")?;
        //let e_min = constants.request_float("e_min")?;
        //let e_max = constants.request_float("e_max")?;
        let sigma_x = inputs.request_float("sigma_x")?;
        let sigma_y = inputs.request_float("sigma_y")?;
        let motion_blur_steps = request_usize("motion_blur_steps", &constants)?;

        let mut signal = inputs.request_detectorfulldata("Background")?;
        //let signal =
        signal.0 = padamo_api::lazy_array_operations::make_lao_box(crate::ops::LazyAnyLCGaussTrack{
            data: signal.0,
            lc,
            detector,pivot_frame,v0,a0,phi0,x0,y0,sigma_x,sigma_y,motion_blur_steps
        });
        signal.2 = ROption::RNone;


        outputs.set_value("Signal", signal.into())?;
        Ok(())
    }
}

impl CalculationNode for AnyLCLinearTrackGeneratorDynamicGaussNode{

    #[doc = r" Category to place node in node list"]
    fn category(&self,) -> RVec<RString> {
        rvec!["Artificial data".into()]
    }


    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString  {
        "Gauss PSF Customizable LC linear track".into()
    }

    fn old_identifier(&self,) -> ROption<RString>where {
        RSome("Artificial data/Dynamic Customizable LC linear track".into())
    }

    fn identifier(&self,) -> RString where {
        "padamotrackgen.linear_track_gauss_dynamic".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self) -> RVec<CalculationIO> {
        ports!(
            ("Background", ContentType::DetectorFullData),
            ("Lightcurve", ContentType::Function),
            ("pivot_frame", ContentType::Float),
            ("v0",ContentType::Float),
            ("a0",ContentType::Float),
            ("phi0",ContentType::Float),
            ("x0",ContentType::Float),
            ("y0",ContentType::Float),
            ("sigma_x",ContentType::Float),
            ("sigma_y",ContentType::Float)
            //("e_min",0.0),
            //("e_max",1.0),
        )
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Output definition of node"]
    fn outputs(&self) -> RVec<CalculationIO> {
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Constants definition of node with default values."]
    fn constants(&self,) -> RVec<CalculationConstant> {
        constants![
            ("motion_blur_steps",5)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError> {
        self.calculate(inputs, outputs, constants, environment).into()
    }


}



impl AnyLCLinearTrackGeneratorDynamicMoffatNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError> {

        let detector_content = environment.request_string("detector")?.to_string();
        let detector: padamo_detectors::polygon::DetectorContent = serde_json::from_str(&detector_content).map_err(|x| ExecutionError::OtherError(format!("{:?}",x).into()))?;
        let detector = crate::ensquared_energy::detector::wireframe(detector);

        //let detector = crate::ensquared_energy::load_detector(&detector_path).ok_or_else(|| ExecutionError::OtherError("Could not load detector for track generator".into()))?;
        let pivot_frame = request_nonnegative_input("pivot_frame", &inputs)?;
        let lc = inputs.request_function("Lightcurve")?;
        let lc = lc.map(|x| if x>0.0 {x} else {0.0});
        let v0 = request_nonnegative_input("v0", &inputs)?;
        let a0 = inputs.request_float("a0")?;
        let x0 = inputs.request_float("x0")?;
        let y0 = inputs.request_float("y0")?;
        let phi0 = inputs.request_float("phi0")?;
        //let e_min = constants.request_float("e_min")?;
        //let e_max = constants.request_float("e_max")?;
        let alpha = inputs.request_float("alpha")?;
        let beta = inputs.request_float("beta")?;
        let motion_blur_steps = request_usize("motion_blur_steps", &constants)?;
        let normalize = constants.request_boolean("normalize")?;

        if normalize && beta<=1.0{
            return Err(ExecutionError::OtherError(format!("Cannot normalize Moffat with beta={}",beta).into()));
        }

        let mut signal = inputs.request_detectorfulldata("Background")?;
        //let signal =
        signal.0 = padamo_api::lazy_array_operations::make_lao_box(crate::ops::LazyAnyLCMoffatTrack{
            data: signal.0,
            lc,
            detector,pivot_frame,v0,a0,phi0,x0,y0,alpha,beta,motion_blur_steps,normalize
        });
        signal.2 = ROption::RNone;


        outputs.set_value("Signal", signal.into())?;
        Ok(())
    }
}

impl CalculationNode for AnyLCLinearTrackGeneratorDynamicMoffatNode{

    #[doc = r" Category to place node in node list"]
    fn category(&self,) -> RVec<RString> {
        rvec!["Artificial data".into()]
    }


    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString  {
        "Moffat PSF Customizable LC linear track".into()
    }

    fn old_identifier(&self,) -> ROption<RString>where {
        RSome("Artificial data/Moffat PSF Customizable LC linear track".into())
    }

    fn identifier(&self,) -> RString where {
        "padamotrackgen.linear_track_moffat_dynamic".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self) -> RVec<CalculationIO> {
        ports!(
            ("Background", ContentType::DetectorFullData),
            ("Lightcurve", ContentType::Function),
            ("pivot_frame", ContentType::Float),
            ("v0",ContentType::Float),
            ("a0",ContentType::Float),
            ("phi0",ContentType::Float),
            ("x0",ContentType::Float),
            ("y0",ContentType::Float),
            ("alpha", ContentType::Float),
            ("beta", ContentType::Float)
            //("sigma_x",ContentType::Float),
            //("sigma_y",ContentType::Float)
            //("e_min",0.0),
            //("e_max",1.0),
        )
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Output definition of node"]
    fn outputs(&self) -> RVec<CalculationIO> {
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Constants definition of node with default values."]
    fn constants(&self,) -> RVec<CalculationConstant> {
        constants![
            ("motion_blur_steps",5),
            ("normalize", true)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError> {
        self.calculate(inputs, outputs, constants, environment).into()
    }


}




#[derive(Clone, Debug)]
pub struct BlankDataNode;

impl BlankDataNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError> {
        let length = request_usize("length",&constants)?;
        let time_offset = constants.request_float("time_offset")?;

        //let shape = constants.request_string("shape")?;
        //let shape = crate::shape_parser::parse_usize_vec(&shape).ok_or_else(|| ExecutionError::OtherError(format!("Cannot parse shape {}",&shape).into()))?;
        let detector_content = environment.request_string("detector")?.to_string();
        let detector: padamo_detectors::polygon::DetectorContent = serde_json::from_str(&detector_content).map_err(|x| ExecutionError::OtherError(format!("{:?}",x).into()))?;
        let shape = detector.compat_shape.clone();

        let temporal = crate::ops::ArtificialTime::new(length, time_offset);
        let spatial = crate::ops::ArtificialBlankSignal::new(length, shape);

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
            ("time_offset",0.0)
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
        let sigma = inputs.request_float("sigma")?;
        if sigma<=0.0{
            return Err(ExecutionError::OtherError("Standard deviation must be positive".into()));
        }
        let seed = inputs.request_integer("seed")?;
        let mut signal = inputs.request_detectorfulldata("Background")?;
        signal.2 = ROption::RNone;
        signal.0 = make_lao_box(crate::ops::LazyAdditiveNormalNoise::new(signal.0, seed, sigma));
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

    fn old_identifier(&self,) -> ROption<RString>where {
        RSome("Artificial data/Additive normal noise".into())
    }

    fn identifier(&self,) -> RString where {
        "padamotrackgen.gaussian_noise".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self) -> RVec<CalculationIO> {
        ports![
            ("Background", ContentType::DetectorFullData),
            ("seed", ContentType::Integer),
            ("sigma", ContentType::Float)
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
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError> {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}
