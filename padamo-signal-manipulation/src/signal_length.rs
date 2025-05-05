use abi_stable::{rvec, std_types::{RString, RVec}};
use padamo_api::{constants, ports, prelude::*};


#[derive(Clone,Debug)]
pub struct SignalLength;

impl SignalLength{
    fn calculate(&self, args:CalculationNodeArguments)->Result<(),ExecutionError> {
        let signal = args.inputs.request_detectorfulldata("Signal")?;
        let len = signal.0.length();
        args.outputs.set_value("Length", (len as i64).into())?;
        Ok(())
    }
}

impl CalculationNode for SignalLength{
    fn name(&self)->abi_stable::std_types::RString {
        "Get signal length".into()
    }

    fn category(&self,) -> abi_stable::std_types::RVec<RString> where {
        rvec!["Signal manipulation".into(), "Get Length".into()]
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

    fn calculate(&self, args:CalculationNodeArguments)->abi_stable::std_types::RResult<(),ExecutionError> {
        self.calculate(args).into()
    }

    fn constants(&self)->RVec<CalculationConstant> {
        constants!()
    }
}



#[derive(Clone,Debug)]
pub struct SignalArrayLength;

impl SignalArrayLength{
    fn calculate(&self, args:CalculationNodeArguments)->Result<(),ExecutionError> {
        let signal = args.inputs.request_detectorsignal("Signal")?;
        let len = signal.length();
        args.outputs.set_value("Length", (len as i64).into())?;
        Ok(())
    }
}

impl CalculationNode for SignalArrayLength{
    fn name(&self)->abi_stable::std_types::RString {
        "Get signal array length".into()
    }

    fn category(&self,) -> abi_stable::std_types::RVec<RString> where {
        rvec!["Signal manipulation".into(), "Get Length".into()]
    }

    fn identifier(&self)->abi_stable::std_types::RString {
        "padamosignalmanipulation.get_signal_array_length".into()
    }

    fn inputs(&self)->RVec<CalculationIO> {
        ports![
            ("Signal",ContentType::DetectorSignal)
        ]
    }

    fn outputs(&self)->RVec<CalculationIO> {
        ports![
            ("Length",ContentType::Integer)
        ]
    }

    fn calculate(&self, args:CalculationNodeArguments)->abi_stable::std_types::RResult<(),ExecutionError> {
        self.calculate(args).into()
    }

    fn constants(&self)->RVec<CalculationConstant> {
        constants!()
    }
}



#[derive(Clone,Debug)]
pub struct SignalTimeLength;

impl SignalTimeLength{
    fn calculate(&self, args:CalculationNodeArguments)->Result<(),ExecutionError> {
        let signal = args.inputs.request_detectortime("Signal")?;
        let len = signal.length();
        args.outputs.set_value("Length", (len as i64).into())?;
        Ok(())
    }
}

impl CalculationNode for SignalTimeLength{
    fn name(&self)->abi_stable::std_types::RString {
        "Get time length".into()
    }

    fn category(&self,) -> abi_stable::std_types::RVec<RString> where {
        rvec!["Signal manipulation".into(), "Get Length".into()]
    }

    fn identifier(&self)->abi_stable::std_types::RString {
        "padamosignalmanipulation.get_time_length".into()
    }

    fn inputs(&self)->RVec<CalculationIO> {
        ports![
            ("Signal",ContentType::DetectorTime)
        ]
    }

    fn outputs(&self)->RVec<CalculationIO> {
        ports![
            ("Length",ContentType::Integer)
        ]
    }

    fn calculate(&self, args:CalculationNodeArguments)->abi_stable::std_types::RResult<(),ExecutionError> {
        self.calculate(args).into()
    }

    fn constants(&self)->RVec<CalculationConstant> {
        constants!()
    }
}
