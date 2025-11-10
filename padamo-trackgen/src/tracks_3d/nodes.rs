
use padamo_api::{constants, ports, prelude::*};
use abi_stable::std_types::{RResult, RString, RVec};
use abi_stable::rvec;



#[derive(Debug,Clone)]
pub struct MeteorTrackNode;

impl MeteorTrackNode{
    fn calculate(&self,args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let detector_name = args.constants.request_string("detector_name")?;
        let mut detector = None;
        for det in args.detectors.iter(){
            if det.get_friendly_name()==detector_name.as_str(){
                detector = Some((det, det.detector_info.clone()));
                break;
            }
        }
        let (detector,detector_info) = detector.ok_or(ExecutionError::OtherError(format!("Detector {} is not found", detector_name).into()))?;
        let detector = crate::ensquared_energy::detector::wireframe(detector.detector.cells.clone());
        let mut data = args.inputs.request_detectorfulldata("Background")?;
        if data.0.length()==0{
            return Err(ExecutionError::OtherError("No background data".into()));
        }
        let probe = data.0.request_range(0,1).take_frame().ok_or(ExecutionError::OtherError("Cannot take test frame".into()))?;

        if !probe.form_compatible(&detector.shape){
            return Err(ExecutionError::OtherError(format!("Background shape {:?} is not compatible with detector {}", probe.shape, detector_name).into()));
        }

        let lc = args.inputs.request_function("Lightcurve")?;
        let psf = args.inputs.request_function("PSF")?;

        let pivot_frame = args.constants.request_float("pivot_frame")?;

        let modify_intensity = args.constants.request_boolean("modify_intensity")?;
        let motion_blur_steps = crate::requesters::request_usize("motion_blur_steps",&args.constants)?;


        let v0 = args.constants.request_float("v0")?;

        let mv = args.inputs.request_detectorsignal("MV Matrix")?;
        let mv = mv.request_range(0,mv.length());

        let mv:nalgebra::Matrix4<f64> = mv.try_into().map_err(|_| ExecutionError::OtherError("Model-view matrix must be 4x4".into()))?;

        data.0 = make_lao_box(super::ops::LazyMeteorTrack{
            motion_blur_steps,
            modify_intensity,
            data:data.0,
            detector,
            detector_info,
            pivot_frame,
            lc,
            psf,
            v0,
            mv,
        });

        args.outputs.set_value("Signal", data.into())?;
        println!("Simulator prepared");
        Ok(())
    }
}


impl CalculationNode for MeteorTrackNode{
    fn name(&self,) -> RString where {
        "Meteor simulator".into()
    }

    fn category(&self,) -> RVec<RString>where {
        rvec!["Artificial data".into(), "3D tracks".into()]
    }

    fn identifier(&self,) -> RString where {
        "padamotrackgen.tracks3d.meteor_track_v2".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Background", ContentType::DetectorFullData),
            ("Lightcurve", ContentType::Function),
            ("PSF", ContentType::Function),
            ("MV Matrix", ContentType::DetectorSignal),
        ]
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Signal", ContentType::DetectorFullData)
        ]
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![
            ("detector_name", "Detector name", ""),
            ("motion_blur_steps","Motion blur steps",5),
            ("modify_intensity", "Follow 1/r^2 falloff", false),
            ("pivot_frame","Zero frame [fr]", 0.0),
            ("v0","v0 [km/fr]",10.0),
        ]
    }

    fn calculate(&self,args:CalculationNodeArguments) -> RResult<(),ExecutionError>{
        self.calculate(args).into()
    }
}
