use crate::trigger_ops::LazyTriggerExpand;
use abi_stable::{rvec, std_types::{ROption::{self, RSome}, RResult, RString, RVec}};
use padamo_api::{constants, nodes_vec, ports, prelude::*};


fn category() -> RVec<RString>where {
    rvec!["Trigger manipulation".into()]
}

#[derive(Clone,Debug)]
pub struct TriggerExpandNode;

impl TriggerExpandNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let mut data = args.inputs.request_detectorfulldata("Signal")?;
        let expansion_signed = args.constants.request_integer("Expansion")?;
        let expansion:usize = match usize::try_from(expansion_signed) {
            Ok(v)=>{v}
            Err(_)=>{return Err(ExecutionError::OtherError("Cannot convert expansion to unsigned".into()));}
        };


        if let ROption::RSome(v) = data.2.take(){
            let conv = LazyTriggerExpand::new(v, expansion);
            let conv = make_lao_box(conv);
            data.2 = ROption::RSome(conv);
        }
        args.outputs.set_value("Signal", data.into())?;
        Ok(())
    }
}

impl CalculationNode for TriggerExpandNode{
    #[allow(clippy::let_and_return)]
    #[doc = r" Category to place node in node list"]
    fn category(&self,) -> RVec<RString>where {
        category()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString where {
        "Expand trigger".into()
    }

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
        RSome("Trigger manipulation/Expand trigger".into())
    }

    fn identifier(&self,) -> RString where {
        "padamocore.trigger_manipulation.expand_trigger".into()
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
    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }


}

#[derive(Clone,Debug)]
pub struct TriggerExchangeNode;


impl TriggerExchangeNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let mut signal = args.inputs.request_detectorfulldata("Signal")?;
        let trigger_source = args.inputs.request_detectorfulldata("Trigger source")?;

        if let ROption::RSome(trig) = trigger_source.2{
            if signal.0.length()==trig.length(){
                signal.2 = ROption::RSome(trig);
            }
            else{
                return Err(ExecutionError::OtherError(format!("Incompatible signal and trigger sizes: {}!={}",signal.0.length(),trig.length()).into()));
            }
        }

        args.outputs.set_value("Signal", signal.into())
    }
}

impl CalculationNode for TriggerExchangeNode {
    fn category(&self,) -> RVec<RString>{
        category()
    }

    fn name(&self,) -> RString {
        "Exchange trigger".into()
    }


    fn identifier(&self,) -> RString {
        "padamocore.trigger_manipulation.exchange_trigger".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>{
        ports![
            ("Signal", ContentType::DetectorFullData),
            ("Trigger source", ContentType::DetectorFullData),
        ]
    }

    fn outputs(&self,) -> RVec<CalculationIO>{
        ports![
            ("Signal", ContentType::DetectorFullData),
        ]
    }

    fn constants(&self,) -> RVec<CalculationConstant>{
        constants!()
    }

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}



#[derive(Clone,Debug)]
pub struct TriggerNegateNode;

impl TriggerNegateNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let mut signal = args.inputs.request_detectorfulldata("Signal")?;

        if let ROption::RSome(trig) = signal.2{
            let trig = make_lao_box(crate::trigger_ops::LazyTriggerNegate::new(trig));
            signal.2 = ROption::RSome(trig);
        }

        args.outputs.set_value("Signal", signal.into())
    }
}

impl CalculationNode for TriggerNegateNode{
    fn name(&self)->RString {
        "Negate trigger".into()
    }

    fn category(&self)->RVec<RString> {
        category()
    }

    fn identifier(&self)->RString {
        "padamocore.trigger_manipulation.negate_trigger".into()
    }

    fn inputs(&self)->RVec<CalculationIO> {
        ports![
            ("Signal", ContentType::DetectorFullData),
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

    fn calculate(&self, args:CalculationNodeArguments)->RResult<(),ExecutionError> {
        self.calculate(args).into()
    }
}

#[derive(Clone,Debug)]
pub struct TriggerAndNode;

impl TriggerAndNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let mut signal_a = args.inputs.request_detectorfulldata("Main Signal")?;
        let signal_b = args.inputs.request_detectorfulldata("Secondary signal")?;

        if let ROption::RSome(trig_1) = signal_a.2{
            if let ROption::RSome(trig_2) = signal_b.2{
                if trig_1.length()!=trig_2.length(){
                    return Err(ExecutionError::OtherError("Lengths of triggers do not match".into()));
                }

                let trig = make_lao_box(crate::trigger_ops::LazyTriggerAnd::new(trig_1,trig_2));
                signal_a.2 = ROption::RSome(trig);
            }
            else{
                signal_a.2 = ROption::RNone;
            }
        }

        args.outputs.set_value("Signal", signal_a.into())
    }
}

impl CalculationNode for TriggerAndNode{
    fn name(&self)->RString {
        "Trigger AND".into()
    }

    fn category(&self)->RVec<RString> {
        category()
    }

    fn identifier(&self)->RString {
        "padamocore.trigger_manipulation.and_trigger".into()
    }

    fn constants(&self)->RVec<CalculationConstant> {
        constants!()
    }

    fn inputs(&self)->RVec<CalculationIO> {
        ports![
            ("Main Signal",ContentType::DetectorFullData),
            ("Secondary signal",ContentType::DetectorFullData),
        ]
    }

    fn outputs(&self)->RVec<CalculationIO> {
        ports![
            ("Signal",ContentType::DetectorFullData),
        ]
    }

    fn calculate(&self, args:CalculationNodeArguments)->RResult<(),ExecutionError> {
        self.calculate(args).into()
    }
}

#[derive(Clone,Debug)]
pub struct TriggerOrNode;

impl TriggerOrNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let mut signal_a = args.inputs.request_detectorfulldata("Main Signal")?;
        let signal_b = args.inputs.request_detectorfulldata("Secondary signal")?;

        if let ROption::RSome(trig_1) = signal_a.2{
            if let ROption::RSome(trig_2) = signal_b.2{
                if trig_1.length()!=trig_2.length(){
                    return Err(ExecutionError::OtherError("Lengths of triggers do not match".into()));
                }

                // A||B = !(!A && !B)
                let not_1 = make_lao_box(crate::trigger_ops::LazyTriggerNegate::new(trig_1));
                let not_2 = make_lao_box(crate::trigger_ops::LazyTriggerNegate::new(trig_2));
                let and1 = make_lao_box(crate::trigger_ops::LazyTriggerAnd::new(not_1,not_2));
                let trig = make_lao_box(crate::trigger_ops::LazyTriggerNegate::new(and1));
                signal_a.2 = ROption::RSome(trig);
            }
            else{
                signal_a.2 = ROption::RNone;
            }
        }

        args.outputs.set_value("Signal", signal_a.into())
    }
}

impl CalculationNode for TriggerOrNode{
    fn name(&self)->RString {
        "Trigger OR".into()
    }

    fn category(&self)->RVec<RString> {
        category()
    }

    fn identifier(&self)->RString {
        "padamocore.trigger_manipulation.or_trigger".into()
    }

    fn constants(&self)->RVec<CalculationConstant> {
        constants!()
    }

    fn inputs(&self)->RVec<CalculationIO> {
        ports![
            ("Main Signal",ContentType::DetectorFullData),
            ("Secondary signal",ContentType::DetectorFullData),
        ]
    }

    fn outputs(&self)->RVec<CalculationIO> {
        ports![
            ("Signal",ContentType::DetectorFullData),
        ]
    }

    fn calculate(&self, args:CalculationNodeArguments)->RResult<(),ExecutionError> {
        self.calculate(args).into()
    }
}



pub fn nodes()->RVec<CalculationNodeBox>{
    nodes_vec![
        TriggerExpandNode,
        TriggerExchangeNode,
        TriggerNegateNode,
        TriggerAndNode,
        TriggerOrNode
        //StringReplaceRegexNode
    ]
}

