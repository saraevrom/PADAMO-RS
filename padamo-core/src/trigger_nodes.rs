use crate::trigger_ops::{LazyTriggerMerge, LazyTriggerRemoveOverlap};
use abi_stable::{rvec, std_types::{ROption::{self, RSome}, RResult, RString, RVec}};
use padamo_api::{constants, nodes_vec, ports, prelude::*};


fn category() -> RVec<RString>where {
    rvec!["Trigger manipulation".into()]
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
pub struct TriggerMergeNode;


impl TriggerMergeNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let mut signal1 = args.inputs.request_detectorfulldata("Signal 1")?;
        let signal2 = args.inputs.request_detectorfulldata("Signal 2")?;
        if signal1.0.length()!=signal2.0.length(){
            return Err(ExecutionError::OtherError(format!("Incompatible signals lengths: {}!={}",signal1.0.length(),signal2.0.length()).into()));
        }


        signal1.2 = match (signal1.2, signal2.2){
            (ROption::RSome(trig1),ROption::RSome(trig2))=>{
                if trig1.length()==trig2.length(){
                    ROption::RSome(make_lao_box(LazyTriggerMerge::new(trig1, trig2)))
                }
                else{
                    return Err(ExecutionError::OtherError(format!("Incompatible trigger sizes: {}!={}",trig1.length(),trig2.length()).into()));
                }
            }
            (ROption::RSome(x),ROption::RNone)=>{
                ROption::RSome(x)
            },
            (ROption::RNone,ROption::RSome(trig2))=>{
                ROption::RSome(trig2)
            },
            (ROption::RNone, ROption::RNone)=>{
                return Err(ExecutionError::OtherError("Both signals must have at least one trigger to merge".into()));
            }
        };

        args.outputs.set_value("Signal", signal1.into())
    }
}

impl CalculationNode for TriggerMergeNode {
    fn category(&self,) -> RVec<RString>{
        category()
    }

    fn name(&self,) -> RString {
        "Merge triggers".into()
    }


    fn identifier(&self,) -> RString {
        "padamocore.trigger_manipulation.merge_trigger".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>{
        ports![
            ("Signal 1", ContentType::DetectorFullData),
            ("Signal 2", ContentType::DetectorFullData),
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
pub struct TriggerRemoveOverlapNode;


impl TriggerRemoveOverlapNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>{
        let mut signal = args.inputs.request_detectorfulldata("Signal")?;
        let template = args.constants.request_string("Format")?;

        signal.2 = if let ROption::RSome(x) = signal.2{
            ROption::RSome(make_lao_box(LazyTriggerRemoveOverlap::new(x, template.into())))
        }
        else{
            ROption::RNone
        };

        args.outputs.set_value("Signal", signal.into())
    }
}

impl CalculationNode for TriggerRemoveOverlapNode {
    fn category(&self,) -> RVec<RString>{
        category()
    }

    fn name(&self,) -> RString {
        "Remove triggers overlaps".into()
    }


    fn identifier(&self,) -> RString {
        "padamocore.trigger_manipulation.remove_trigger_overlaps".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>{
        ports![
            ("Signal", ContentType::DetectorFullData),
        ]
    }

    fn outputs(&self,) -> RVec<CalculationIO>{
        ports![
            ("Signal", ContentType::DetectorFullData),
        ]
    }

    fn constants(&self,) -> RVec<CalculationConstant>{
        constants![
            ("Format", "{a}")
        ]
    }

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}

pub fn nodes()->RVec<CalculationNodeBox>{
    nodes_vec![
        //TriggerExpandNode,
        TriggerExchangeNode,
        TriggerMergeNode,
        TriggerRemoveOverlapNode,
        //TriggerNegateNode,
        //TriggerAndNode,
        //TriggerOrNode
        //StringReplaceRegexNode
    ]
}

