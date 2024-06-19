use abi_stable::{rvec, std_types::ROption};
use padamo_api::{prelude::*, ports, constants};
use abi_stable::std_types::RVec;
use super::ops::{LazySpaceConverter,LazyTimeConverter};
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
        let signal = LazySpaceConverter::new(divider,trisignal.0,is_sum);
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
            ("Is sum", false)
        )
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,_:&mut RandomState) -> abi_stable::std_types::RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}
