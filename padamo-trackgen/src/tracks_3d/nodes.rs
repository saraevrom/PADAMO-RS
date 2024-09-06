use std::default;
use std::f64::consts::PI;

use abi_stable::std_types::ROption::RSome;
use padamo_api::lazy_array_operations::LazyTriSignal;
use padamo_api::{constants, ports, prelude::*};
use abi_stable::std_types::{ROption, RResult, RString, RVec, Tuple3};
use abi_stable::rvec;
use serde_json::value;



#[derive(Debug,Clone)]
pub struct GaussPSFMeteorTrackNode;

impl GaussPSFMeteorTrackNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer, environment: &mut ContentContainer) -> Result<(),ExecutionError>{

        let detector_content = environment.request_string("detector")?.to_string();
        let detector: padamo_detectors::polygon::DetectorContent = serde_json::from_str(&detector_content).map_err(|x| ExecutionError::OtherError(format!("{:?}",x).into()))?;
        let detector = crate::ensquared_energy::detector::wireframe(detector);

        let mut data = inputs.request_detectorfulldata("Background")?;
        let lc = inputs.request_function("Lightcurve")?;

        let pivot_frame = constants.request_float("pivot_frame")?;

        let modify_intensity = constants.request_boolean("modify_intensity")?;
        let motion_blur_steps = crate::requesters::request_usize("motion_blur_steps",&constants)?;

        let theta0 = constants.request_float("theta0")?*PI/180.0;
        let phi0 = constants.request_float("phi0")?*PI/180.0;

        let e0_x = -theta0.sin()*phi0.cos();
        let e0_y = -theta0.sin()*phi0.sin();
        let e0_z = -theta0.cos();


        let x0_planar = constants.request_float("X0")?;
        let y0_planar = constants.request_float("Y0")?;
        let z0 = constants.request_float("z0")?;
        let f = constants.request_float("f")?;
        if f<=0.0{
            return Err(ExecutionError::OtherError("Focal distance must be positive".into()));
        }

        let sigma_x = constants.request_float("sigma_x")?;
        let sigma_y = constants.request_float("sigma_y")?;

        let x0 = x0_planar*z0/f;
        let y0 = y0_planar*z0/f;

        let v0 = constants.request_float("v0")?;
        let a0 = constants.request_float("a0")?;

        data.0 = make_lao_box(super::ops::LazyGaussPSFMeteorTrack{
            motion_blur_steps,
            modify_intensity,
            data:data.0,
            detector,
            pivot_frame,
            lc,
            x0,y0,z0,
            v0_x: v0*e0_x,
            v0_y: v0*e0_y,
            v0_z: v0*e0_z,

            a0_x: a0*e0_x,
            a0_y: a0*e0_y,
            a0_z: a0*e0_z,
            f,
            sigma_x,
            sigma_y
        });

        outputs.set_value("Signal", data.into())?;
        Ok(())
    }
}

impl CalculationNode for GaussPSFMeteorTrackNode{
    fn name(&self,) -> RString where {
        "Gauss PSF Meteor track".into()
    }

    #[doc = r" Category to place node in node list"]
    fn category(&self,) -> RVec<RString> {
        rvec!["Artificial data".into(), "3D tracks".into()]
    }

    fn identifier(&self,) -> RString where {
        "padamotrackgen.tracks3d.meteor_track".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Background", ContentType::DetectorFullData),
            ("Lightcurve", ContentType::Function),
        )
    }

    fn outputs(&self) -> RVec<CalculationIO> {
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![
            ("motion_blur_steps","Motion blur steps",5),
            ("modify_intensity", "Follow 1/r^2 falloff", false),

            ("pivot_frame","Zero frame [fr]", 0.0),

            ("v0","v0 [km/fr]",10.0),
            ("a0","a0 [km/fr^2]",0.0),
            ("z0","z0 [km]",100.0),

            ("theta0","theta0 [deg]",0.0),
            ("phi0","phi0 [deg]",0.0),

            ("X0","X0 [mm]",0.0),
            ("Y0","Y0 [mm]",0.0),
            ("f","Focal distance [mm]",150.0),

            ("sigma_x","Sigma X",1.0),
            ("sigma_y","Sigma Y",1.0)
        ]
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> RResult<(),ExecutionError>{
        self.calculate(inputs, outputs, constants, environment).into()
    }
}
