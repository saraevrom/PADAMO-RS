use crate::{constants, trigger_ops::LazyTriggerExpand};
use abi_stable::{rvec, std_types::{ROption, RResult, RString, RVec}};
use padamo_api::{ports, constants, prelude::*};

#[derive(Clone,Debug)]
pub struct TriggerExpandNode;

impl TriggerExpandNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        let mut data = inputs.request_detectorfulldata("Signal")?;
        let expansion_signed = constants.request_integer("Expansion")?;
        let expansion:usize = match usize::try_from(expansion_signed) {
            Ok(v)=>{v}
            Err(_)=>{return Err(ExecutionError::OtherError("Cannot convert expansion to unsigned".into()));}
        };


        if let ROption::RSome(v) = data.2.take(){
            let conv = LazyTriggerExpand::new(v, expansion);
            let conv = make_lao_box(conv);
            data.2 = ROption::RSome(conv);
        }
        outputs.set_value("Signal", data.into())?;
        Ok(())
    }
}

impl CalculationNode for TriggerExpandNode{
    #[allow(clippy::let_and_return)]
    #[doc = r" Category to place node in node list"]
    fn category(&self,) -> RVec<RString>where {
        rvec!["Trigger manipulation".into()]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString where {
        "Expand trigger".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Output definition of node"]
    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Constants definition of node with default values."]
    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!(
            ("Expansion", 0)
        )
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }


}
