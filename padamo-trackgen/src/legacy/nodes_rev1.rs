
use abi_stable::std_types::ROption::RSome;
use padamo_api::{constants, nodes_vec, ports, prelude::*};
use abi_stable::std_types::{ROption, RResult, RString, RVec};
use abi_stable::rvec;


#[derive(Debug,Clone)]
pub struct BasicLinearTrackGeneratorNodeOld;


#[derive(Debug,Clone)]
pub struct AnyLCLinearTrackGeneratorNodeOld;

#[derive(Debug,Clone)]
pub struct AnyLCLinearTrackGeneratorDynamicGaussNodeOld;

#[derive(Debug,Clone)]
pub struct AnyLCLinearTrackGeneratorDynamicMoffatNodeOld;

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

impl BasicLinearTrackGeneratorNodeOld{
    fn calculate(&self,args:CalculationNodeArguments) -> Result<(),ExecutionError> {

        let detector_content = args.environment.request_string("detector")?.to_string();
        let detector: padamo_detectors::polygon::Detector = serde_json::from_str(&detector_content).map_err(|x| ExecutionError::OtherError(format!("{:?}",x).into()))?;
        let detector = crate::ensquared_energy::detector::wireframe(detector);

        //let detector = crate::ensquared_energy::load_detector(&detector_path).ok_or_else(|| ExecutionError::OtherError("Could not load detector for track generator".into()))?;
        let pivot_frame = request_nonnegative("pivot_frame", &args.constants)?;
        let attack_frames = request_nonnegative("attack_frames", &args.constants)?;
        let sustain_frames = request_nonnegative("sustain_frames", &args.constants)?;
        let decay_frames = request_nonnegative("decay_frames", &args.constants)?;
        let v0 = request_nonnegative("v0", &args.constants)?;
        let a0 = args.constants.request_float("a0")?;
        let x0 = args.constants.request_float("x0")?;
        let y0 = args.constants.request_float("y0")?;
        let phi0 = args.constants.request_float("phi0")?;
        let e_min = args.constants.request_float("e_min")?;
        let e_max = args.constants.request_float("e_max")?;
        let sigma_x = args.constants.request_float("sigma_x")?;
        let sigma_y = args.constants.request_float("sigma_y")?;
        let motion_blur_steps = request_usize("motion_blur_steps", &args.constants)?;

        let mut signal = args.inputs.request_detectorfulldata("Background")?;
        //let signal =
        signal.0 = padamo_api::lazy_array_operations::make_lao_box(super::ops_rev1::LazyTriangularE0Track{
            data: signal.0,
            detector,pivot_frame,attack_frames,sustain_frames,decay_frames,v0,a0,phi0,x0,y0,e_min,e_max,sigma_x,sigma_y,motion_blur_steps
        });
        signal.2 = ROption::RNone;


        args.outputs.set_value("Signal", signal.into())?;
        Ok(())
    }
}

impl CalculationNode for BasicLinearTrackGeneratorNodeOld{

    #[doc = r" Category to place node in node list"]
    fn category(&self,) -> RVec<RString> {
        rvec!["Legacy".into(),"Artificial data".into()]
    }


    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString  {
        "Basic linear track (Legacy)".into()
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
    fn calculate(&self,args:CalculationNodeArguments) -> RResult<(),ExecutionError> {
        self.calculate(args).into()
    }


}


impl AnyLCLinearTrackGeneratorNodeOld{
    fn calculate(&self,args:CalculationNodeArguments) -> Result<(),ExecutionError> {

        let detector_content = args.environment.request_string("detector")?.to_string();
        let detector: padamo_detectors::polygon::Detector = serde_json::from_str(&detector_content).map_err(|x| ExecutionError::OtherError(format!("{:?}",x).into()))?;
        let detector = crate::ensquared_energy::detector::wireframe(detector);

        //let detector = crate::ensquared_energy::load_detector(&detector_path).ok_or_else(|| ExecutionError::OtherError("Could not load detector for track generator".into()))?;
        let pivot_frame = request_nonnegative("pivot_frame", &args.constants)?;
        let lc = args.inputs.request_function("Lightcurve")?;
        let lc = lc.map(|x| if x>0.0 {x} else {0.0});
        let v0 = request_nonnegative("v0", &args.constants)?;
        let a0 = args.constants.request_float("a0")?;
        let x0 = args.constants.request_float("x0")?;
        let y0 = args.constants.request_float("y0")?;
        let phi0 = args.constants.request_float("phi0")?;
        //let e_min = constants.request_float("e_min")?;
        //let e_max = constants.request_float("e_max")?;
        let sigma_x = args.constants.request_float("sigma_x")?;
        let sigma_y = args.constants.request_float("sigma_y")?;
        let motion_blur_steps = request_usize("motion_blur_steps", &args.constants)?;

        let mut signal = args.inputs.request_detectorfulldata("Background")?;
        //let signal =
        signal.0 = padamo_api::lazy_array_operations::make_lao_box(super::ops_rev1::LazyAnyLCGaussTrack{
            data: signal.0,
            lc,
            detector,pivot_frame,v0,a0,phi0,x0,y0,sigma_x,sigma_y,motion_blur_steps
        });
        signal.2 = ROption::RNone;


        args.outputs.set_value("Signal", signal.into())?;
        Ok(())
    }
}

impl CalculationNode for AnyLCLinearTrackGeneratorNodeOld{

    #[doc = r" Category to place node in node list"]
    fn category(&self,) -> RVec<RString> {
        rvec!["Legacy".into(),"Artificial data".into()]
    }


    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString  {
        "Customizable LC linear track (Legacy)".into()
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
    fn calculate(&self,args:CalculationNodeArguments) -> RResult<(),ExecutionError> {
        self.calculate(args).into()
    }


}



impl AnyLCLinearTrackGeneratorDynamicGaussNodeOld{
    fn calculate(&self,args:CalculationNodeArguments) -> Result<(),ExecutionError> {

        let detector_content = args.environment.request_string("detector")?.to_string();
        let detector: padamo_detectors::polygon::Detector = serde_json::from_str(&detector_content).map_err(|x| ExecutionError::OtherError(format!("{:?}",x).into()))?;
        let detector = crate::ensquared_energy::detector::wireframe(detector);

        //let detector = crate::ensquared_energy::load_detector(&detector_path).ok_or_else(|| ExecutionError::OtherError("Could not load detector for track generator".into()))?;
        let pivot_frame = request_nonnegative_input("pivot_frame", &args.inputs)?;
        let lc = args.inputs.request_function("Lightcurve")?;
        let lc = lc.map(|x| if x>0.0 {x} else {0.0});
        let v0 = request_nonnegative_input("v0", &args.inputs)?;
        let a0 = args.inputs.request_float("a0")?;
        let x0 = args.inputs.request_float("x0")?;
        let y0 = args.inputs.request_float("y0")?;
        let phi0 = args.inputs.request_float("phi0")?;
        //let e_min = constants.request_float("e_min")?;
        //let e_max = constants.request_float("e_max")?;
        let sigma_x = args.inputs.request_float("sigma_x")?;
        let sigma_y = args.inputs.request_float("sigma_y")?;
        let motion_blur_steps = request_usize("motion_blur_steps", &args.constants)?;

        let mut signal = args.inputs.request_detectorfulldata("Background")?;
        //let signal =
        signal.0 = padamo_api::lazy_array_operations::make_lao_box(super::ops_rev1::LazyAnyLCGaussTrack{
            data: signal.0,
            lc,
            detector,pivot_frame,v0,a0,phi0,x0,y0,sigma_x,sigma_y,motion_blur_steps
        });
        signal.2 = ROption::RNone;


        args.outputs.set_value("Signal", signal.into())?;
        Ok(())
    }
}

impl CalculationNode for AnyLCLinearTrackGeneratorDynamicGaussNodeOld{

    #[doc = r" Category to place node in node list"]
    fn category(&self,) -> RVec<RString> {
        rvec!["Legacy".into(),"Artificial data".into()]
    }


    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString  {
        "Gauss PSF Customizable LC linear track (Legacy)".into()
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
    fn calculate(&self,args:CalculationNodeArguments) -> RResult<(),ExecutionError> {
        self.calculate(args).into()
    }


}



impl AnyLCLinearTrackGeneratorDynamicMoffatNodeOld{
    fn calculate(&self,args:CalculationNodeArguments) -> Result<(),ExecutionError> {

        let detector_content = args.environment.request_string("detector")?.to_string();
        let detector: padamo_detectors::polygon::Detector = serde_json::from_str(&detector_content).map_err(|x| ExecutionError::OtherError(format!("{:?}",x).into()))?;
        let detector = crate::ensquared_energy::detector::wireframe(detector);

        //let detector = crate::ensquared_energy::load_detector(&detector_path).ok_or_else(|| ExecutionError::OtherError("Could not load detector for track generator".into()))?;
        let pivot_frame = request_nonnegative_input("pivot_frame", &args.inputs)?;
        let lc = args.inputs.request_function("Lightcurve")?;
        let lc = lc.map(|x| if x>0.0 {x} else {0.0});
        let v0 = request_nonnegative_input("v0", &args.inputs)?;
        let a0 = args.inputs.request_float("a0")?;
        let x0 = args.inputs.request_float("x0")?;
        let y0 = args.inputs.request_float("y0")?;
        let phi0 = args.inputs.request_float("phi0")?;
        //let e_min = constants.request_float("e_min")?;
        //let e_max = constants.request_float("e_max")?;
        let alpha = args.inputs.request_float("alpha")?;
        let beta = args.inputs.request_float("beta")?;
        let motion_blur_steps = request_usize("motion_blur_steps", &args.constants)?;
        let normalize = args.constants.request_boolean("normalize")?;

        if normalize && beta<=1.0{
            return Err(ExecutionError::OtherError(format!("Cannot normalize Moffat with beta={}",beta).into()));
        }

        let mut signal = args.inputs.request_detectorfulldata("Background")?;
        //let signal =
        signal.0 = padamo_api::lazy_array_operations::make_lao_box(super::ops_rev1::LazyAnyLCMoffatTrack{
            data: signal.0,
            lc,
            detector,pivot_frame,v0,a0,phi0,x0,y0,alpha,beta,motion_blur_steps,normalize
        });
        signal.2 = ROption::RNone;


        args.outputs.set_value("Signal", signal.into())?;
        Ok(())
    }
}

impl CalculationNode for AnyLCLinearTrackGeneratorDynamicMoffatNodeOld{

    #[doc = r" Category to place node in node list"]
    fn category(&self,) -> RVec<RString> {
        rvec!["Legacy".into(),"Artificial data".into()]
    }


    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString  {
        "Moffat PSF Customizable LC linear track (Legacy)".into()
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
    fn calculate(&self,args:CalculationNodeArguments) -> RResult<(),ExecutionError> {
        self.calculate(args).into()
    }


}

#[derive(Clone, Debug)]
pub struct AdditiveNormalNoiseNodeOld;

impl AdditiveNormalNoiseNodeOld{
    fn calculate(&self,args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let sigma = args.inputs.request_float("sigma")?;
        if sigma<=0.0{
            return Err(ExecutionError::OtherError("Standard deviation must be positive".into()));
        }
        let seed = args.inputs.request_integer("seed")?;
        let mut signal = args.inputs.request_detectorfulldata("Background")?;
        signal.2 = ROption::RNone;
        signal.0 = make_lao_box(super::ops_rev1::LazyAdditiveNormalNoise::new(signal.0, seed, sigma));
        args.outputs.set_value("Signal", signal.into())
    }
}

impl CalculationNode for AdditiveNormalNoiseNodeOld{
    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self) -> RString  {
        "Additive normal noise (Legacy)".into()
    }

    fn category(&self) -> RVec<RString> {
        rvec!["Legacy".into(),"Artificial data".into()]
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
    fn calculate(&self,args:CalculationNodeArguments) -> RResult<(),ExecutionError> {
        self.calculate(args).into()
    }
}

pub fn nodes()->RVec<CalculationNodeBox>{
    nodes_vec!(
        BasicLinearTrackGeneratorNodeOld,
        AnyLCLinearTrackGeneratorNodeOld,
        AnyLCLinearTrackGeneratorDynamicGaussNodeOld,
        AnyLCLinearTrackGeneratorDynamicMoffatNodeOld,
        AdditiveNormalNoiseNodeOld,
    )
}
