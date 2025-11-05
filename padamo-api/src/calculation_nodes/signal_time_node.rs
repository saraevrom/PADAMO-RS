
use abi_stable::std_types::ROption::{self, RNone, RSome};
use abi_stable::std_types::{RResult, RString, RVec};
use crate::lazy_array_operations::{LazyDetectorSignal,LazyTimeSignal, LazyTriSignal};
use crate::prelude::*;

use crate::{constants, ports, CalculationNode, ContentType};


#[derive(Clone,Debug)]
pub struct SignalTimeEmbeddedMergingNode<T:CalculationNode, U:CalculationNode>{
    pub signal:T,
    pub time:U,
    pub name:RString,
    pub identifier:RString,
    pub category:RVec<RString>,
    pub legacy_id:ROption<RString>,
}

impl<T:CalculationNode, U:CalculationNode> SignalTimeEmbeddedMergingNode<T,U>{
    pub fn new<N:Into<RString>,I:Into<RString>>(signal:T, time:U, name:N, identifier:I)->Self{
        Self { signal, time, name:name.into() ,
            identifier:identifier.into(),
            category:crate::common_categories::data_sources(),
            legacy_id:RNone,
        }
    }

    pub fn with_legacy_id<V:Into<RString>>(mut self, legacy_id:V)->Self{
        self.legacy_id = RSome(legacy_id.into());
        self
    }


    fn calculate(&self,args:CalculationNodeArguments,) -> Result<(),ExecutionError>{
        //let inputs = args.inputs;
        let env = args.environment;
        let inputs = args.inputs;
        let consts = args.constants;
        let rng = args.rng;

        let mut outputs_signal = IOData::new(self.signal.outputs());

        let args_signal = CalculationNodeArguments{
            inputs:inputs.clone(),
            outputs:&mut outputs_signal,
            constants:consts.clone(),
            environment:env,
            rng,
            detectors_serialized: args.detectors_serialized
        };

        self.signal.calculate(args_signal).into_result()?;

        let mut outputs_time = IOData::new(self.time.outputs());

        let args_time = CalculationNodeArguments{
            inputs:inputs.clone(),
            outputs:&mut outputs_time,
            constants:consts.clone(),
            environment:env,
            rng,
            detectors_serialized: args.detectors_serialized
        };

        self.time.calculate(args_time).into_result()?;

        //Expecting Output called "Array" or "Signal" in signal results

        let mut signal = outputs_signal.take_value("Array");
        if signal.is_none() {signal = outputs_signal.take_value("Signal")};
        if signal.is_none(){
            return Err(ExecutionError::OtherError(format!("Node {} could not extract \"Array\" or \"Signal\" output from signal part",self.name()).into()));
        }
        let time = outputs_time.take_value("Time");
        if time.is_none(){
            return Err(ExecutionError::OtherError(format!("Node {} could not extract \"Time\" output from temporal part",self.name()).into()));
        }

        //Checked if None before doing anything.
        let signal = signal.unwrap();
        let time = time.unwrap();

        let signal_unwrapped:LazyDetectorSignal;
        if let Content::DetectorSignal(ds) = signal{
            signal_unwrapped = ds;
        }
        else{
            return Err(ExecutionError::OtherError(format!("Node {} detector type is wrong",self.name()).into()));
        }

        let time_unwrapped:LazyTimeSignal;
        if let Content::DetectorTime(dt) = time{
            time_unwrapped = dt;
        }
        else{
            return Err(ExecutionError::OtherError(format!("Node {} detector type is wrong",self.name()).into()));
        }

        let res:LazyTriSignal = (signal_unwrapped, time_unwrapped, RNone).into();
        args.outputs.set_value("Signal", res.into())
    }
}


impl<T:CalculationNode, U:CalculationNode> CalculationNode for SignalTimeEmbeddedMergingNode<T,U>{
    fn name(&self,) -> RString{
        self.name.clone()
    }

    fn category(&self,) -> RVec<RString>{
        self.category.clone()
    }

    fn identifier(&self,) -> RString{
        self.identifier.clone()
    }

    fn old_identifier(&self,) -> ROption<RString>{
        self.legacy_id.clone()
    }

    fn inputs(&self,) -> RVec<CalculationIO>{
        ports!(
            ("Filename", ContentType::String)
        )
    }

    fn outputs(&self) ->RVec<CalculationIO>{
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>{
        let mut c = constants!();
        c.extend(self.signal.constants());
        c.extend(self.time.constants());
        c
    }

    fn calculate(&self,args:CalculationNodeArguments,) -> RResult<(),ExecutionError>{
        self.calculate(args).into()
    }
}
