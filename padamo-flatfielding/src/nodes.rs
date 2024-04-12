use abi_stable::{rvec, std_types::{RResult, RString, RVec}};
use padamo_api::{constants, ports, prelude::*};


#[derive(Clone,Debug)]
pub struct PhysicalFFNode;

impl PhysicalFFNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> Result<(),ExecutionError>where {
        let mut signal = inputs.request_detectorfulldata("Signal")?;
        let eff_2d = inputs.request_detectorsignal("Eff_2D")?;
        let tau = inputs.request_detectorsignal("Tau")?;
        let eff_2d = eff_2d.request_range(0,eff_2d.length());
        let tau = tau.request_range(0,tau.length());
        if signal.0.length()==0{
            return Err(ExecutionError::OtherError("Cannot check signal shape compatibility".into()));
        }
        let test_data = signal.0.request_range(0,1).squeeze();

        if !test_data.is_compatible(&tau){
            return Err(ExecutionError::OtherError("flat fielding tau is not compatible with signal".into()));
        }
        if !test_data.is_compatible(&eff_2d){
            return Err(ExecutionError::OtherError("flat fielding eff_2d is not compatible with signal".into()));
        }
        //if test_data.shape.le
        signal.0 = make_lao_box(crate::ops::PhysicalFF::new(signal.0, eff_2d, tau));
        outputs.set_value("Signal", signal.into())?;
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
        rvec!["Flat fielding".into()]
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
        constants!()
    }

    #[allow(clippy::let_and_return)]
    #[doc = r" Main calculation"]
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,) -> RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment).into()
    }
}
