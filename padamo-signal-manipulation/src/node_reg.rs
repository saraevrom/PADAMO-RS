use abi_stable::{rvec, std_types::ROption::{self, RSome}};
use padamo_api::{prelude::*, ports, constants};
use abi_stable::std_types::{RResult,RVec,RString};
use padamo_arraynd::ArrayND;
use crate::ops::SignalMux;

use super::ops::{LazySpaceConverter,LazyTimeConverter, TimeShift};
use super::tempreduce_performance::LazySpaceConverterPerformant;
use padamo_api::lazy_array_operations::LazyArrayOperationBox;
use abi_stable::sabi_trait::prelude::TD_Opaque;


#[derive(Clone,Debug)]
pub struct TimeResolutionReduceNode;

impl TimeResolutionReduceNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let trisignal = args.inputs.request_detectorfulldata("Signal")?;
        let divider = args.constants.request_integer("Divider")?;
        let is_sum = args.constants.request_boolean("Is sum")?;
        if divider<=0{
            return Err(ExecutionError::OtherError("Divider must be natural number".into()));
        }
        let divider = divider as usize;
        let signal = if args.constants.request_boolean("performance_over_memory")?{
            make_lao_box(LazySpaceConverterPerformant::new(divider,trisignal.0,is_sum))
        }
        else{
            make_lao_box(LazySpaceConverter::new(divider,trisignal.0,is_sum))
        };
        let time = LazyTimeConverter::new(divider,trisignal.1);
        //let signal = LazyArrayOperationBox::from_value(signal, TD_Opaque);
        let time = LazyArrayOperationBox::from_value(time, TD_Opaque);
        let trisignal = (signal,time,ROption::RNone);
        args.outputs.set_value("Signal", Content::DetectorFullData(trisignal.into()))?;

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
            ("performance_over_memory", "Performance over memory", true),
        )
    }

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}


#[derive(Clone,Debug)]
pub struct CutterNode;

impl CutterNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let mut src = args.inputs.request_detectorfulldata("Signal")?;
        let start = args.inputs.request_integer("Start")?;
        let start:usize = start.try_into().map_err(ExecutionError::from_error)?;
        let end = args.inputs.request_integer("End")?;
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
        args.outputs.set_value("Signal", src.into())?;
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

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}

#[derive(Clone,Debug)]
pub struct TimeOffsetNode;

impl TimeOffsetNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let mut signal = args.inputs.request_detectorfulldata("Signal")?;
        let off = args.constants.request_float("offset")?;
        signal.1 = make_lao_box(TimeShift::new(signal.1, off));
        args.outputs.set_value("Signal", signal.into())
    }
}

impl CalculationNode for TimeOffsetNode{
    fn name(&self,) -> RString where {
        "Time offset".into()
    }

    fn category(&self,) -> RVec<RString>where {
        rvec!["Signal manipulation".into()]
    }

    fn identifier(&self,) -> RString where {
        "padamosignalmanipulation.time_offset".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = " Input definitions of node"]
    fn inputs(&self,) -> RVec<CalculationIO> {
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    #[allow(clippy::let_and_return)]
    #[doc = " Output definition of node"]
    fn outputs(&self,) -> RVec<CalculationIO> {
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    #[allow(clippy::let_and_return)]
    #[doc = " Constants definition of node with default values."]
    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!(
            ("offset", "Offset [s]", 0.0)
        )
    }

    #[allow(clippy::let_and_return)]
    #[doc = " Main calculation"]
    fn calculate(&self,args:CalculationNodeArguments,) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}

#[derive(Clone,Debug)]
pub struct SignalMultiplexerNode;

impl SignalMultiplexerNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let mut signal_false = args.inputs.request_detectorfulldata("Negative")?;
        let mut signal_true = args.inputs.request_detectorfulldata("Positive")?;
        let mask = args.inputs.request_detectorsignal("Mask")?;
        let mask:ArrayND<f64> = mask.request_range(0, mask.length());
        let mask = ArrayND{shape:mask.shape, flat_data:mask.flat_data.iter().map(|x| * x!= 0.0).collect()};

        let rest_mux = args.constants.request_boolean("primary")?;

        let signal = if rest_mux{
            signal_true.0 = make_lao_box(SignalMux::new(signal_false.0, signal_true.0, mask));
            signal_true
        }
        else{
            signal_false.0 = make_lao_box(SignalMux::new(signal_false.0, signal_true.0, mask));
            signal_false
        };

        args.outputs.set_value("Signal", signal.into())
    }
}

impl CalculationNode for SignalMultiplexerNode{
    fn name(&self,) -> RString{
        "Signal mask multiplexer".into()
    }

    fn category(&self,) -> RVec<RString>{
        rvec!["Signal manipulation".into()]
    }

    fn identifier(&self,) -> RString{
        "padamosignalmanipulation.signal_mask_multiplexer".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>{
        ports![
            ("Positive", ContentType::DetectorFullData),
            ("Negative", ContentType::DetectorFullData),
            ("Mask", ContentType::DetectorSignal),
        ]
    }

    fn outputs(&self,) -> RVec<CalculationIO>{
        ports![
            ("Signal", ContentType::DetectorFullData),
        ]
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![
            ("primary", "Time and trigger source", true)
        ]
    }

    fn calculate(&self,args:CalculationNodeArguments,) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}
