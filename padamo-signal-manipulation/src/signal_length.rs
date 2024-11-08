use abi_stable::{rvec, std_types::{RString, RVec}};
use padamo_api::{constants, ports, prelude::*};


#[derive(Clone,Debug)]
pub struct SignalLength;

impl SignalLength{
    fn calculate(&self, inputs:ContentContainer, outputs:&mut IOData, constants:ConstantContentContainer, environment:&mut ContentContainer, rng:&mut RandomState)->Result<(),ExecutionError> {
        let signal = inputs.request_detectorfulldata("Signal")?;
        let len = signal.0.length();
        outputs.set_value("Length", (len as i64).into())?;
        Ok(())
    }
}

impl CalculationNode for SignalLength{
    fn name(&self)->abi_stable::std_types::RString {
        "Get signal length".into()
    }

    fn category(&self,) -> abi_stable::std_types::RVec<RString> where {
        rvec!["Signal manipulation".into()]
    }

    fn identifier(&self)->abi_stable::std_types::RString {
        "padamosignalmanipulation.get_signal_length".into()
    }

    fn inputs(&self)->RVec<CalculationIO> {
        ports![
            ("Signal",ContentType::DetectorFullData)
        ]
    }

    fn outputs(&self)->RVec<CalculationIO> {
        ports![
            ("Length",ContentType::Integer)
        ]
    }

    fn calculate(&self, inputs:ContentContainer, outputs:&mut IOData, constants:ConstantContentContainer, environment:&mut ContentContainer, rng:&mut RandomState)->abi_stable::std_types::RResult<(),ExecutionError> {
        self.calculate(inputs, outputs, constants, environment, rng).into()
    }

    fn constants(&self)->RVec<CalculationConstant> {
        constants!()
    }
}
