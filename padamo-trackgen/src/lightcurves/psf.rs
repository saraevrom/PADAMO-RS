use std::f64::consts::PI;

use abi_stable::rvec;
use abi_stable::std_types::{RResult, RString, RVec};
use padamo_api::function_operator::{DoubleFunctionOperator,DoubleFunctionOperatorBox};
use padamo_api::{constants, ports, prelude::*};

fn category() -> RVec<RString>where {
    rvec!["Functions".into(), "PSF".into()]
}

#[derive(Clone,Debug)]
pub struct GaussianPSF;

impl GaussianPSF{
    fn calculate(&self,args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let e0 = args.constants.request_float("E0")?;
        let sigma = args.constants.request_float("Sigma")?;
        if sigma<=0.0{
            return Err(ExecutionError::OtherError("Gaussian sigma coefficient must be positive".into()));
        }
        let f:DoubleFunctionOperatorBox = (move |r:f64| {
            e0*f64::exp(-r*r/(2.0*PI*sigma*sigma))
        }).into();
        args.outputs.set_value("F", f.into())
    }
}

impl CalculationNode for GaussianPSF{
    fn name(&self,) -> RString where {
        "Gaussian PSF".into()
    }

    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn identifier(&self,) -> RString where {
        "padamotrackgen.psf.gauss".into()
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("F", ContentType::Function)
        ]
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![
            ("E0", 1.0),
            ("Sigma", 1.0),
        ]
    }

    fn calculate(&self,args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}

#[derive(Clone,Debug)]
pub struct MoffatPSF;

impl MoffatPSF{
    fn calculate(&self,args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let e0 = args.constants.request_float("E0")?;
        // let sigma = args.constants.request_float("Sigma")?;
        let alpha = args.constants.request_float("Alpha")?;
        if alpha<=0.0{
            return Err(ExecutionError::OtherError("Moffat alpha coefficient must be positive".into()));
        }
        let beta = args.constants.request_float("Beta")?;
        let f:DoubleFunctionOperatorBox = (move |r:f64| {
            e0*(1.0+(r*r/(alpha*alpha))).powf(-beta)
        }).into();
        args.outputs.set_value("F", f.into())
    }
}

impl CalculationNode for MoffatPSF{
    fn name(&self,) -> RString where {
        "Moffat PSF".into()
    }

    fn category(&self,) -> RVec<RString>where {
        category()
    }

    fn identifier(&self,) -> RString where {
        "padamotrackgen.psf.moffat".into()
    }

    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("F", ContentType::Function)
        ]
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants![
            ("E0", 1.0),
            ("Alpha",1.0),
            ("Beta",4.765),
        ]
    }

    fn calculate(&self,args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}
