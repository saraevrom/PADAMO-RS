use abi_stable::std_types::{ROption::RNone, RVec};
use padamo_api::{constants, lazy_array_operations::{LazyArrayOperation, LazyTriSignal}, ports, prelude::*};
use crate::ops::{AddTime,LazyROOTSpatialReader};

#[derive(Clone,Debug)]
pub struct EUSOROOTNode;

impl EUSOROOTNode{
    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> Result<(),ExecutionError>where {
        let filename:String = inputs.request_string("Filename")?.into();
        let tree:String = constants.request_string("Tree")?.into();
        let branch:String = constants.request_string("Branch")?.into();
        let spatial = LazyROOTSpatialReader::new(filename, tree, branch);
        if spatial.length()==0{
            return Err(ExecutionError::OtherError("ROOT file length is zero".into()));
        }
        let length = spatial.length();
        let tmpbase:String = constants.request_string("tmpbase")?.into();
        let tmpres = constants.request_float("tmpres")?;
        let temporal = if let Some(v) = AddTime::new(length, tmpres, &tmpbase) {v}
            else {return Err(ExecutionError::OtherError("Cannot parse datetime".into()));};
        // let dt = if let Some(v) = datetime_parser::parse_datetimes(&tmpbase, chrono::Utc::now()) {v}
        //     else {return Err(ExecutionError::OtherError("Cannot parse datetime".into()));};
        // let tmpbase = (dt.naive_utc().and_utc().timestamp_micros() as f64)*1e-6;
        // let temporal = AddTime::new(length, tmpres, tmpbase);
        let signal:LazyTriSignal = (make_lao_box(spatial), make_lao_box(temporal), RNone).into();
        outputs.set_value("Signal", signal.into())?;
        Ok(())
    }
}

impl CalculationNode for EUSOROOTNode{
    fn name(&self,) -> abi_stable::std_types::RString where {
        "Basic ROOT file reader".into()
    }

    fn identifier(&self,) -> abi_stable::std_types::RString where {
        "padamoroot.basic_root_reader".into()
    }

    fn category(&self,) -> abi_stable::std_types::RVec<abi_stable::std_types::RString>where {
        vec!["ROOT".into()].into()
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
            ("Tree", "tevent"),
            ("Branch", "photon_count_data"),
            ("tmpbase","Temporal unixtime base", formatted),
            ("tmpres","Temporal resolution", 0.000256),
        )
    }

    fn calculate(&self,inputs:ContentContainer,outputs: &mut IOData,constants:ConstantContentContainer,environment: &mut ContentContainer,rng: &mut RandomState,) -> abi_stable::std_types::RResult<(),ExecutionError>where {
        self.calculate(inputs, outputs, constants, environment, rng).into()
    }

}
