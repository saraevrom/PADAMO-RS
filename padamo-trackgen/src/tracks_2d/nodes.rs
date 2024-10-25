
use padamo_api::{constants, ports, prelude::*};
use abi_stable::std_types::{ROption, RResult, RString, RVec};
use abi_stable::rvec;


#[derive(Debug,Clone)]
pub struct AnyLCLinearTrackGeneratorDynamicGaussNode;

#[derive(Debug,Clone)]
pub struct AnyLCLinearTrackGeneratorDynamicMoffatNode;
use crate::requesters::{request_nonnegative,request_usize};
/*
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
}*/


impl AnyLCLinearTrackGeneratorDynamicGaussNode{
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
        signal.0 = padamo_api::lazy_array_operations::make_lao_box(super::ops::LazyAnyLCGaussTrack{
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
        rvec!["Artificial data".into(), "2D tracks".into()]
    }


    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString  {
        "Gauss PSF linear track".into()
    }

    fn identifier(&self,) -> RString where {
        "padamotrackgen.linear_track_gauss_dynamic2".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self) -> RVec<CalculationIO> {
        ports!(
            ("Background", ContentType::DetectorFullData),
            ("Lightcurve", ContentType::Function),
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
            ("motion_blur_steps","Motion blur steps",5),
            ("pivot_frame","Zero frame [fr]", 0.0),
            ("v0","U0 [mm/fr]",1.0),
            ("a0","A0 [mm/fr^2]",0.0),
            ("phi0","Phi0 [deg]",0.0),
            ("x0","X0 [mm]",0.0),
            ("y0","Y0 [mm]",0.0),
            ("sigma_x","Sigma X [mm]",1.0),
            ("sigma_y","Sigma Y [mm]",1.0)
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
        let alpha = constants.request_float("alpha")?;
        let beta = constants.request_float("beta")?;
        let motion_blur_steps = request_usize("motion_blur_steps", &constants)?;
        let normalize = constants.request_boolean("normalize")?;

        if normalize && beta<=1.0{
            return Err(ExecutionError::OtherError(format!("Cannot normalize Moffat with beta={}",beta).into()));
        }

        let mut signal = inputs.request_detectorfulldata("Background")?;
        //let signal =
        signal.0 = padamo_api::lazy_array_operations::make_lao_box(super::ops::LazyAnyLCMoffatTrack{
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
        rvec!["Artificial data".into(), "2D tracks".into()]
    }


    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString  {
        "Moffat PSF linear track".into()
    }

    fn identifier(&self,) -> RString where {
        "padamotrackgen.linear_track_moffat_dynamic2".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self) -> RVec<CalculationIO> {
        ports!(
            ("Background", ContentType::DetectorFullData),
            ("Lightcurve", ContentType::Function),
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

            ("motion_blur_steps","Motion blur steps",5),
            ("pivot_frame","Zero frame [fr]", 0.0),
            ("v0","U0 [mm/fr]",1.0),
            ("a0","A0 [mm/fr^2]",0.0),
            ("phi0","Phi0 [deg]",0.0),
            ("x0","X0 [mm]",0.0),
            ("y0","Y0 [mm]",0.0),

            ("alpha",1.0),
            ("beta",4.765),
            ("normalize","Normalize",true)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError> {
        self.calculate(inputs, outputs, constants, environment).into()
    }


}

