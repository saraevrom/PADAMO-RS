use abi_stable::std_types::{RResult, RString, RVec};
use abi_stable::rvec;
use padamo_api::{constants, nodes_vec, ports, prelude::*};
use padamo_api::function_operator::DoubleFunctionOperatorBox;


fn category() -> RVec<RString>where {
    rvec!["Functions".into(), "FFT filters".into()]
}

fn default_ports() ->RVec<CalculationIO>{
    ports![
        ("Filter", ContentType::Function)
    ]
}

#[derive(Clone,Debug)]
pub struct InvertFilterNode;

impl InvertFilterNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let inner = args.inputs.request_function("Filter")?;
        let output = inner.map(move |x| 1.0-x);
        args.outputs.set_value("Filter", output.into())?;
        Ok(())
    }
}

impl CalculationNode for InvertFilterNode{
    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn name(&self,) -> RString where {
        "Inverse filter".into()
    }

    fn identifier(&self,) -> RString where {
        "padamosfft.filters.inverse".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        default_ports()
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        default_ports()
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![]
    }

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}


#[derive(Clone,Debug)]
pub struct CombineANDFilterNode;

impl CombineANDFilterNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let inner1 = args.inputs.request_function("Filter 1")?;
        let inner2 = args.inputs.request_function("Filter 2")?;
        let output:DoubleFunctionOperatorBox = (move |x| {inner1.calculate(x)*inner2.calculate(x)}).into();
        args.outputs.set_value("Filter", output.into())?;
        Ok(())
    }
}

impl CalculationNode for CombineANDFilterNode{
    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn name(&self,) -> RString where {
        "AND filter".into()
    }

    fn identifier(&self,) -> RString where {
        "padamosfft.filters.combine.and".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Filter 1", ContentType::Function),
            ("Filter 2", ContentType::Function),
        ]
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        default_ports()
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![]
    }

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}


#[derive(Clone,Debug)]
pub struct CombineORFilterNode;

impl CombineORFilterNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let inner1 = args.inputs.request_function("Filter 1")?;
        let inner2 = args.inputs.request_function("Filter 2")?;
        let output:DoubleFunctionOperatorBox = (move |x| {
            1.0-(1.0-inner1.calculate(x))*(1.0-inner2.calculate(x))

        }).into();
        args.outputs.set_value("Filter", output.into())?;
        Ok(())
    }
}

impl CalculationNode for CombineORFilterNode{
    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn name(&self,) -> RString where {
        "OR filter".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Filter 1", ContentType::Function),
            ("Filter 2", ContentType::Function),
        ]
    }

    fn identifier(&self,) -> RString where {
        "padamosfft.filters.combine.or".into()
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        default_ports()
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![]
    }

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}


#[derive(Clone,Debug)]
pub struct BandPassFilter;

impl BandPassFilter{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let freq = args.constants.request_float("frequency")?;
        let band = args.constants.request_float("band_width")?;
        let output:DoubleFunctionOperatorBox = (move |x:f64| {
            if (x.abs()-freq).abs()*2.0<band{
                1.0
            }
            else{
                0.0
            }
        }).into();
        args.outputs.set_value("Filter", output.into())?;
        Ok(())
    }
}

impl CalculationNode for BandPassFilter{
    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn name(&self,) -> RString where {
        "Band pass (hard)".into()
    }

    fn identifier(&self,) -> RString where {
        "padamosfft.filters.band_pass".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!()
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        default_ports()
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![
            ("frequency", "Frequency", 0.0),
            ("band_width", "Band width", 0.0),
        ]
    }

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}



#[derive(Clone,Debug)]
pub struct ButtleworthLowPassFilter;

impl ButtleworthLowPassFilter{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let freq = args.constants.request_float("frequency")?;
        let f_cutoff = freq;
        let power = args.constants.request_float("power")?;
        let output:DoubleFunctionOperatorBox = (move |x:f64| {
            let f = x.abs();
            1./(1.+(f/f_cutoff).powf(2.*power)).sqrt()
        }).into();
        args.outputs.set_value("Filter", output.into())?;
        Ok(())
    }
}

impl CalculationNode for ButtleworthLowPassFilter{
    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn name(&self,) -> RString where {
        "Buttleworth low pass".into()
    }


    fn identifier(&self,) -> RString where {
        "padamosfft.filters.buttleworth".into()
    }

    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports!()
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        default_ports()
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![
            ("frequency", "Cutoff frequency", 0.0),
            ("power", "Power", 1.0),
        ]
    }

    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}


pub fn nodes()->RVec<CalculationNodeBox>{
    nodes_vec![
        InvertFilterNode,
        CombineANDFilterNode,
        CombineORFilterNode,
        BandPassFilter,
        ButtleworthLowPassFilter
    ]
}
