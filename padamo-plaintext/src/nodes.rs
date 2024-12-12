use abi_stable::{rvec, std_types::{ROption::RNone, RVec}};
use padamo_api::{constants, lazy_array_operations::LazyTriSignal, ports, prelude::*};


#[derive(Clone,Debug)]
pub struct CSVNode;

impl CSVNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> Result<(),ExecutionError>where {
        let start = constants.request_integer("start")?;
        let length = constants.request_integer("length")?;
        let start:usize = start.try_into().ok().unwrap_or(0);
        let length:Option<usize> = length.try_into().ok();
        let tmpbase = constants.request_string("tmpbase")?.into_string();
        let tmpres = constants.request_float("tmpres")?;
        let input_file = inputs.request_string("Filename")?.to_string();
        let separator = constants.request_string("separator")?.to_string();
        let spatial = crate::ops::CSVReader::new(separator, input_file, start, length).map_err(ExecutionError::from_error)?;
        let temporal = pseudotime::ops::AddTime::new(spatial.length, tmpres, &tmpbase).ok_or_else(|| ExecutionError::OtherError("Cannot parse datetime".into()))?;
        let signal:LazyTriSignal = (make_lao_box(spatial),make_lao_box(temporal), RNone).into();
        outputs.set_value("Signal", signal.into())
    }
}

impl CalculationNode for CSVNode{
    fn name(&self,) -> abi_stable::std_types::RString where {
        "CSV".into()
    }

    fn category(&self,) -> abi_stable::std_types::RVec<abi_stable::std_types::RString>where {
        rvec!["Plain text".into()]
    }

    fn identifier(&self,) -> abi_stable::std_types::RString where {
        "padamoplaintext.read_csv".into()
    }


    fn inputs(&self,) -> abi_stable::std_types::RVec<CalculationIO>where {
        ports![
            ("Filename", ContentType::String)
        ]
    }

    fn outputs(&self) -> RVec<CalculationIO> {
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        let now = chrono::Utc::now();
        let dat = now.format("%Y-%m-%d %H:%M:%S.0");
        let formatted = format!("{}", dat);
        constants!(
            ("separator","Separator", r",\s*"),
            ("start","Start line", 0),
            ("length","End line", -1),
            ("tmpbase","Temporal unixtime base", formatted),
            ("tmpres","Temporal resolution", 0.000256),
        )
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> abi_stable::std_types::RResult<(),ExecutionError>where {
        self.calculate(inputs,outputs,constants,environment,rng).into()
    }
}
