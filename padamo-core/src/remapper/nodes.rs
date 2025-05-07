use abi_stable::{rvec, std_types::ROption::RSome};
use padamo_api::{constants, ports, prelude::*};
use super::ops::LazyRemapper;
use index_remapper::{parse_f64_remapper, parse_bool_remapper};

#[derive(Clone,Debug)]
pub struct RemapperNode;

impl RemapperNode{
    fn calculate(&self,args:CalculationNodeArguments,) -> Result<(),ExecutionError>{
        let mut src_signal = args.inputs.request_detectorfulldata("Signal")?;
        if src_signal.0.length()==0{
            return Err(ExecutionError::OtherError("Cannot remap empty data".into()))
        }
        let mut testframe = src_signal.0.request_range(0,1);
        testframe.shape.drain(..1);


        let remapper_src = args.constants.request_string("remapper_f64")?;
        if !remapper_src.is_empty(){
            let remapper = parse_f64_remapper(&remapper_src, testframe.shape.clone().into(), 0.0).map_err(ExecutionError::from_error)?;
            src_signal.0 = make_lao_box(LazyRemapper::new(src_signal.0, remapper));
        }

        if let RSome(trigger) = src_signal.2.take(){
            let mut testframe = src_signal.0.request_range(0,1);
            testframe.shape.drain(..1);

            let remapper_src = args.constants.request_string("remapper_trigger")?;
            if !remapper_src.is_empty(){
                let remapper = parse_bool_remapper(&remapper_src, testframe.shape.clone().into(), false).map_err(ExecutionError::from_error)?;
                src_signal.2 = RSome(make_lao_box(LazyRemapper::new(trigger, remapper)));
            }
            else{
                src_signal.2 = RSome(trigger);
            }
        }


        args.outputs.set_value("Signal", src_signal.into())
    }
}

impl CalculationNode for RemapperNode{
    fn name(&self,) -> abi_stable::std_types::RString {
        "Remap indices".into()
    }

    fn category(&self,) -> abi_stable::std_types::RVec<abi_stable::std_types::RString>{
        rvec!["Signal manipulation".into()]
    }

    fn identifier(&self,) -> abi_stable::std_types::RString {
        "padamocore.remapper".into()
    }

    fn inputs(&self,) -> abi_stable::std_types::RVec<CalculationIO>{
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    fn outputs(&self,) -> abi_stable::std_types::RVec<CalculationIO>{
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    fn constants(&self,) -> abi_stable::std_types::RVec<CalculationConstant>{
        constants!(
            ("remapper_f64", "Pixels remap", "0.0; i0, i1"),
            ("remapper_trigger", "Trigger remap", "")
        )
    }

    fn calculate(&self,args:CalculationNodeArguments,) -> abi_stable::std_types::RResult<(),ExecutionError>{
        self.calculate(args).into()
    }
}
