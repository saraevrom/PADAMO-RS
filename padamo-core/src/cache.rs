use abi_stable::rvec;
use padamo_api::{constants, ports, prelude::*};


#[derive(Clone,Debug)]
pub struct ForcedCacheNode;

impl ForcedCacheNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let mut signal = args.inputs.request_detectorfulldata("Signal")?;
        signal.0 = signal.0.cached();
        args.outputs.set_value("Signal", signal.into())
    }
}

impl CalculationNode for ForcedCacheNode{
    fn name(&self,) -> abi_stable::std_types::RString where {
        "Cache signal".into()
    }

    fn category(&self,) -> abi_stable::std_types::RVec<abi_stable::std_types::RString>where {
        rvec![]
    }

    fn identifier(&self,) -> abi_stable::std_types::RString where {
        "padamocore.force_cache".into()
    }

    fn constants(&self,) -> abi_stable::std_types::RVec<CalculationConstant>where {
        constants!()
    }

    fn inputs(&self,) -> abi_stable::std_types::RVec<CalculationIO>where {
        ports![
            ("Signal", ContentType::DetectorFullData)
        ]
    }

    fn outputs(&self,) -> abi_stable::std_types::RVec<CalculationIO>where {
        ports![
            ("Signal", ContentType::DetectorFullData)
        ]
    }

    fn calculate(&self, args:CalculationNodeArguments) -> abi_stable::std_types::RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}
