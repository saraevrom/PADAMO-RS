use abi_stable::{rvec, std_types::ROption::{self, RSome}};
use padamo_api::{prelude::*, ports, constants};
use abi_stable::std_types::{RResult,RVec,RString};
use super::ops::{LazySpaceConverter,LazyTimeConverter};
use super::tempreduce_performance::LazySpaceConverterPerformant;
use padamo_api::lazy_array_operations::LazyArrayOperationBox;
use abi_stable::sabi_trait::prelude::TD_Opaque;


#[derive(Clone,Debug)]
pub struct TimeResolutionReduceNode;

impl TimeResolutionReduceNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        let trisignal = inputs.request_detectorfulldata("Signal")?;
        let divider = constants.request_integer("Divider")?;
        let is_sum = constants.request_boolean("Is sum")?;
        if divider<=0{
            return Err(ExecutionError::OtherError("Divider must be natural number".into()));
        }
        let divider = divider as usize;
        let signal = if constants.request_boolean("performance_over_memory")?{
            make_lao_box(LazySpaceConverterPerformant::new(divider,trisignal.0,is_sum))
        }
        else{
            make_lao_box(LazySpaceConverter::new(divider,trisignal.0,is_sum))
        };
        let time = LazyTimeConverter::new(divider,trisignal.1);
        let signal = LazyArrayOperationBox::from_value(signal, TD_Opaque);
        let time = LazyArrayOperationBox::from_value(time, TD_Opaque);
        let trisignal = (signal,time,ROption::RNone);
        outputs.set_value("Signal", Content::DetectorFullData(trisignal.into()))?;

        Ok(())
    }
}

impl CalculationNode for TimeResolutionReduceNode{
    fn name(&self,) -> abi_stable::std_types::RString where {
        "Reduce temporal resolution".into()
    }

    fn category(&self,) -> RVec<abi_stable::std_types::RString>where {
        rvec!["Signal manipulation".into()]
    }

    fn identifier(&self,) -> RString where {
        "padamosignalmanipulation.reduce_temp_resolution".into()
    }

    fn old_identifier(&self,) -> ROption<RString>where {
        RSome("Signal manipulation/Reduce temporal resolution".into())
    }

    fn inputs(&self)->RVec<CalculationIO>{
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
            ("Divider", 1000),
            ("Is sum", false),
            ("performance_over_memory", "Performance over memory", false),
        )
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}


#[derive(Clone,Debug)]
pub struct CutterNode;

impl CutterNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> Result<(),ExecutionError>where {
        let mut src = inputs.request_detectorfulldata("Signal")?;
        let start = inputs.request_integer("Start")?;
        let start:usize = start.try_into().map_err(ExecutionError::from_error)?;
        let end = inputs.request_integer("End")?;
        let end:usize = end.try_into().map_err(ExecutionError::from_error)?;
        let l = src.0.length();
        if start>end{
            return Err(ExecutionError::OtherError(format!("Invalid range ({}>{})",start,end).into()));
        }
        if end>l{
            return Err(ExecutionError::OtherError(format!("End point is larget than length of array ({}>{})",end,l).into()));
        }
        src.0 = make_lao_box(crate::ops::CutterOperator::new(start, end, src.0));
        src.1 = make_lao_box(crate::ops::CutterOperator::new(start, end, src.1));
        src.2 = match src.2.into_option() {
            Some(v)=>ROption::RSome(make_lao_box(crate::ops::CutterOperator::new(start, end, v))),
            None=>ROption::RNone,
        };
        outputs.set_value("Signal", src.into())?;
        Ok(())
    }
}

impl CalculationNode for CutterNode{
    fn name(&self,) -> RString where {
        "Cut signal".into()
    }
    fn category(&self,) -> abi_stable::std_types::RVec<RString> where {
        rvec!["Signal manipulation".into()]
    }

    fn old_identifier(&self,) -> ROption<RString>where {
        RSome("Signal manipulation/Cut signal".into())
    }

    fn identifier(&self,) -> RString where {
        "padamosignalmanipulation.cut_signal".into()
    }

    fn inputs(&self)->RVec<CalculationIO>{
        ports!(
            ("Signal", ContentType::DetectorFullData),
            ("Start", ContentType::Integer),
            ("End", ContentType::Integer)
        )
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!()
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment, rng).into()
    }
}
