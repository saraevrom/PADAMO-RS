use padamo_api::{constants, prelude::*};
use abi_stable::{rvec, std_types::RString};
use padamo_api::ports;
use abi_stable::std_types::RVec;
use crate::detector_muxer::{get_signal_var_by_name, get_transform_var_by_name};

#[derive(Clone,Debug)]
pub struct SmartViewNode;

#[derive(Clone,Debug)]
pub struct SmartMaskNode;

#[derive(Clone,Debug)]
pub struct SmartTransformNode;

fn category() -> abi_stable::std_types::RVec<abi_stable::std_types::RString>where {
    rvec![
        "Application".into(),
        "Viewer".into(),
        "Smart".into(),
    ]
}

impl SmartViewNode{
    fn calculate(&self,args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let signal = args.inputs.request_detectorfulldata("Signal")?;
        let detector = args.constants.request_string("detector")?;
        args.environment.0.insert(get_signal_var_by_name(&detector).into(),Content::DetectorFullData(signal));
        Ok(())
    }
}

impl SmartMaskNode{
    fn calculate(&self,args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let detector_name = args.constants.request_string("detector")?;
        let detector = super::find_detector(&args, detector_name.as_str()).ok_or(
            ExecutionError::OtherError(format!("Cannot find detector {}", detector_name).into())
        )?;

        let mask = detector.detector.alive_pixels.clone();
        let mask = make_lao_box(mask.cast::<f64>());

        // let mask= args.environment.request_detectorsignal(&get_mask_var_by_name(&detector))?;
        args.outputs.set_value("Alive pixels", mask.into())?;
        Ok(())
    }
}

impl SmartTransformNode{
    fn calculate(&self,args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let signal = args.inputs.request_detectorsignal("Transform")?;
        let detector_id = args.constants.request_string("detector")?;
        args.environment.0.insert(get_transform_var_by_name(&detector_id).into(),signal.into());
        Ok(())
    }
}

impl CalculationNode for SmartViewNode{
    fn name(&self,) -> RString where {
        "Smart view node".into()
    }

    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn identifier(&self,) -> RString where {
        "builtin.viewer.view_smart".into()
    }

    fn inputs(&self) -> RVec<CalculationIO>{
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!(
            ("detector", "Detector name", "")
        )
    }

    fn calculate(&self,args:CalculationNodeArguments) -> abi_stable::std_types::RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }

    fn is_primary(&self,) -> bool where {
        true
    }
}

impl CalculationNode for SmartMaskNode{
    fn name(&self,) -> RString where {
        "Smart mask node".into()
    }

    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn identifier(&self,) -> RString where {
        "builtin.viewer.mask_smart".into()
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Alive pixels", ContentType::DetectorSignal)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!(
            ("detector", "Detector name", "")
        )
    }

    fn calculate(&self,args:CalculationNodeArguments) -> abi_stable::std_types::RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}

impl CalculationNode for SmartTransformNode{
    fn name(&self,) -> RString where {
        "Smart transform node".into()
    }

    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn identifier(&self,) -> RString where {
        "builtin.viewer.transform_smart".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Transform", ContentType::DetectorSignal)
        ]
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!(
            ("detector", "Detector name", "")
        )
    }

    fn is_primary(&self,) -> bool where {
        true
    }

    fn calculate(&self,args:CalculationNodeArguments) -> abi_stable::std_types::RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}

pub fn register_nodes(nodes:&mut crate::nodes_interconnect::NodesRegistry){
    nodes.register_node(SmartViewNode).unwrap();
    nodes.register_node(SmartMaskNode).unwrap();
    nodes.register_node(SmartTransformNode).unwrap();
}
