use abi_stable::{rvec, std_types::{ROption::{self, RSome}, RResult, RString, RVec}};
use padamo_api::{constants, ports, prelude::*};

use crate::ops::{PhysicalFFConstants, ApplyByMap};

pub fn category() -> abi_stable::std_types::RVec<abi_stable::std_types::RString>where {
    rvec!["Flat fielding".into()]
}

pub fn old_id(name:&str)->ROption<RString>{
    RSome(format!("Flat fielding/{}",name).into())
}

fn fun_add(x:f64,y:f64)->f64{
    x+y
}

fn fun_sub(x:f64,y:f64)->f64{
    x-y
}

fn fun_mul(x:f64,y:f64)->f64{
    x*y
}

fn fun_div(x:f64,y:f64)->f64{
    if y==0.0{
        0.0
    }
    else{
        x/y
    }
}

macro_rules! impl_operator {
    ($struct_name:ident, $identifier:expr, $name:expr, $operator:ident) => {
        #[derive(Clone,Debug)]
        pub struct $struct_name;

        impl $struct_name{
            fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
                let mut signal = args.inputs.request_detectorfulldata("Signal")?;
                let coeffs = args.inputs.request_detectorsignal("Coefficients")?;
                let mut coeffs = coeffs.request_range(0,coeffs.length());
                if args.constants.request_boolean("squeeze_map")?{
                    coeffs = coeffs.squeeze();
                }

                if signal.0.length()==0{
                    return Err(ExecutionError::OtherError("Cannot check signal shape compatibility".into()));
                }
                let mut test_data = signal.0.request_range(0,1);
                test_data.shape.drain(..1);

                if !test_data.is_compatible(&coeffs){
                    return Err(ExecutionError::OtherError(format!("coefficient matrix with shape {:?} is not compatible with signal with shape {:?}", coeffs.shape,test_data.shape).into()));
                }

                //if test_data.shape.le
                // signal.0 = make_lao_box($operator::new(signal.0, coeffs));
                signal.0 = make_lao_box(ApplyByMap::new(signal.0, coeffs, $operator));
                args.outputs.set_value("Signal", signal.into())?;
                Ok(())
            }
        }

        impl CalculationNode for $struct_name{
            #[allow(clippy::let_and_return)]
            #[doc = r" Name of node displayed in graph editor or node list"]
            fn name(&self,) -> RString where {
                $name.into()
            }

            fn category(&self,) -> abi_stable::std_types::RVec<abi_stable::std_types::RString>where {
                category()
            }

            fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
                old_id($name)
            }

            fn identifier(&self,) -> RString where {
                $identifier.into()
            }

            #[allow(clippy::let_and_return)]
            #[doc = r" Input definitions of node"]
            fn inputs(&self,) -> RVec<CalculationIO>where {
                ports![
                    ("Signal",ContentType::DetectorFullData),
                    ("Coefficients",ContentType::DetectorSignal)
                ]
            }

            #[allow(clippy::let_and_return)]
            #[doc = r" Output definition of node"]
            fn outputs(&self,) -> RVec<CalculationIO>where {
                ports![
                    ("Signal",ContentType::DetectorFullData)
                ]
            }

            #[allow(clippy::let_and_return)]
            #[doc = r" Constants definition of node with default values."]
            fn constants(&self,) -> RVec<CalculationConstant>where {
                constants!(
                    ("squeeze_map","Squeeze map",false)
                )
            }

            #[allow(clippy::let_and_return)]
            #[doc = r" Main calculation"]
            fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
                self.calculate(args).into()
            }
        }
    };
}

#[derive(Clone,Debug)]
pub struct PhysicalFFNode;

impl PhysicalFFNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let mut signal = args.inputs.request_detectorfulldata("Signal")?;
        let eff_2d = args.inputs.request_detectorsignal("Eff_2D")?;
        let tau = args.inputs.request_detectorsignal("Tau")?;
        let mut eff_2d = eff_2d.request_range(0,eff_2d.length());
        let mut tau = tau.request_range(0,tau.length());
        if args.constants.request_boolean("squeeze_map")?{
            tau = tau.squeeze();
            eff_2d = eff_2d.squeeze();
        }
        if signal.0.length()==0{
            return Err(ExecutionError::OtherError("Cannot check signal shape compatibility".into()));
        }
        let test_data = signal.0.request_range(0,1).squeeze();

        if !test_data.is_compatible(&tau){
            return Err(ExecutionError::OtherError(format!("flat fielding tau {:?} is not compatible with signal {:?}", test_data.shape, tau.shape).into()));
        }
        if !test_data.is_compatible(&eff_2d){
            return Err(ExecutionError::OtherError(format!("flat fielding eff_2d {:?} is not compatible with signal {:?}", test_data.shape, eff_2d.shape).into()));
        }
        //if test_data.shape.le
        let consts = PhysicalFFConstants::from_constlist(&args.constants)?;
        signal.0 = make_lao_box(crate::ops::PhysicalFF::new(signal.0, eff_2d, tau, consts));
        args.outputs.set_value("Signal", signal.into())?;
        Ok(())
    }
}

impl CalculationNode for PhysicalFFNode{
    #[allow(clippy::let_and_return)]
    #[doc = r" Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString where {
        "Pile up flat fielding".into()
    }

    fn category(&self,) -> abi_stable::std_types::RVec<abi_stable::std_types::RString>where {
        category()
    }

    fn old_identifier(&self,) -> abi_stable::std_types::ROption<RString>where {
        old_id("Pile up flat fielding")
    }

    fn identifier(&self,) -> RString where {
        "padamoflatfielding.pile_up_ff".into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Input definitions of node"]
    fn inputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Signal",ContentType::DetectorFullData),
            ("Eff_2D",ContentType::DetectorSignal),
            ("Tau",ContentType::DetectorSignal)

        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Output definition of node"]
    fn outputs(&self,) -> RVec<CalculationIO>where {
        ports![
            ("Signal",ContentType::DetectorFullData)
        ]
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Constants definition of node with default values."]
    fn constants(&self,) -> RVec<CalculationConstant>where {
        let mut consts = PhysicalFFConstants::constlist();
        consts.push(("squeeze_map","Squeeze map",false).into());
        consts
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self, args:CalculationNodeArguments) -> RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}

//crate::ops::MultiplyByMap
impl_operator! (MapMultiplyNode, "padamoflatfielding.map_multiply", "Multiply by map", fun_mul);
impl_operator! (MapDivideNode, "padamoflatfielding.map_divide", "Divide by map", fun_div);
impl_operator! (AddMapNode, "padamoflatfielding.map_add", "Add map", fun_add);
impl_operator! (SubMapNode, "padamoflatfielding.map_sub", "Subtract map", fun_sub);
