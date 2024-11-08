use abi_stable::{rvec, std_types::RString};
use abi_stable::std_types::{ROption, RVec};
use padamo_api::lazy_array_operations::LazyTriSignal;
use padamo_api::{constants, ports, prelude::*};


#[derive(Clone,Debug)]
pub struct StretchSignal;

impl StretchSignal{
    fn calculate(&self, inputs:ContentContainer, outputs:&mut IOData, constants:ConstantContentContainer, environment:&mut ContentContainer, rng:&mut RandomState)->Result<(),ExecutionError> {
        let main_signal = inputs.request_detectorfulldata("Signal")?;
        let time_source = inputs.request_detectorfulldata("Time source")?;

        let new_signal = make_lao_box(super::ops_stretch::SyncedSignalStretcher::new(main_signal.0, main_signal.1.clone(), time_source.1.clone()));
        let new_time = time_source.1.clone();
        let new_trig = if let ROption::RSome(trig) = main_signal.2{
            let new_trigger = super::ops_stretch::SyncedTriggerStretcher::new(trig, main_signal.1, time_source.1);
            let new_trigger = make_lao_box(new_trigger);
            ROption::RSome(new_trigger)
        }
        else{
            ROption::RNone
        };

        let new_signal = (new_signal,new_time,new_trig);
        let new_signal:LazyTriSignal = new_signal.into();
        outputs.set_value("Signal", new_signal.into())?;
        Ok(())
    }
}

impl CalculationNode for StretchSignal{
    fn name(&self)->abi_stable::std_types::RString {
        "Stretch signal".into()
    }

    fn category(&self,) -> abi_stable::std_types::RVec<RString> where {
        rvec!["Signal manipulation".into()]
    }

    fn identifier(&self)->abi_stable::std_types::RString {
        "padamosignalmanipulation.stretch_signal".into()
    }

    fn inputs(&self)->RVec<CalculationIO> {
        ports![
            ("Signal", ContentType::DetectorFullData),
            ("Time source", ContentType::DetectorFullData)
        ]
    }

    fn outputs(&self)->RVec<CalculationIO> {
        ports![
            ("Signal", ContentType::DetectorFullData),
        ]
    }

    fn constants(&self)->RVec<CalculationConstant> {
        constants!()
    }

    fn calculate(&self, inputs:ContentContainer, outputs:&mut IOData, constants:ConstantContentContainer, environment:&mut ContentContainer, rng:&mut RandomState)->abi_stable::std_types::RResult<(),ExecutionError> {
        self.calculate(inputs, outputs, constants, environment, rng).into()
    }
}
