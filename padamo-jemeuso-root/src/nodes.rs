use abi_stable::std_types::RVec;
use padamo_api::{constants, lazy_array_operations::LazyArrayOperation, ports, prelude::*};
use crate::ops::{LazyROOTSpatialReader, LazyROOTTemporalReader};

#[derive(Clone,Debug)]
pub struct EUSOROOTArrayNode;

impl EUSOROOTArrayNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let filename:String = args.inputs.request_string("Filename")?.into();
        let tree:String = args.constants.request_string("Tree")?.into();
        let branch:String = args.constants.request_string("Branch")?.into();
        let spatial = LazyROOTSpatialReader::new(filename, tree, branch);
        if spatial.length()==0{
            return Err(ExecutionError::OtherError("ROOT file length is zero".into()));
        }

        args.outputs.set_value("Array", make_lao_box(spatial).into())?;
        Ok(())
    }
}

impl CalculationNode for EUSOROOTArrayNode{
    fn name(&self,) -> abi_stable::std_types::RString where {
        "Basic ROOT file array reader".into()
    }

    fn identifier(&self,) -> abi_stable::std_types::RString where {
        "padamoroot.basic_root_array_reader".into()
    }

    fn category(&self,) -> abi_stable::std_types::RVec<abi_stable::std_types::RString>where {
        padamo_api::common_categories::array_sources()
    }

    fn inputs(&self,) -> abi_stable::std_types::RVec<CalculationIO>where {
        ports![
            ("Filename", ContentType::String)
        ]
    }

    fn outputs(&self) -> RVec<CalculationIO> {
        ports!(
            ("Array", ContentType::DetectorSignal)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!(
            ("Tree", "Signal tree", "tevent"),
            ("Branch", "Signal branch", "photon_count_data"),
        )
    }

    fn calculate(&self, args:CalculationNodeArguments) -> abi_stable::std_types::RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }

}


#[derive(Clone,Debug)]
pub struct EUSOROOTTimeNode;

impl EUSOROOTTimeNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let filename:String = args.inputs.request_string("Filename")?.into();
        let tree:String = args.constants.request_string("Tree_time")?.into();
        let branch:String = args.constants.request_string("Branch_time")?.into();
        let temporal= LazyROOTTemporalReader::new(filename, tree, branch);
        if temporal.length()==0{
            return Err(ExecutionError::OtherError("ROOT file length is zero".into()));
        }

        args.outputs.set_value("Time", make_lao_box(temporal).into())?;
        Ok(())
    }
}

impl CalculationNode for EUSOROOTTimeNode{
    fn name(&self,) -> abi_stable::std_types::RString where {
        "Basic ROOT file time reader".into()
    }

    fn identifier(&self,) -> abi_stable::std_types::RString where {
        "padamoroot.basic_root_time_reader".into()
    }

    fn category(&self,) -> abi_stable::std_types::RVec<abi_stable::std_types::RString>where {
        padamo_api::common_categories::time_sources()
    }

    fn inputs(&self,) -> abi_stable::std_types::RVec<CalculationIO>where {
        ports![
            ("Filename", ContentType::String)
        ]
    }

    fn outputs(&self) -> RVec<CalculationIO> {
        ports!(
            ("Time", ContentType::DetectorTime)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        constants!(
            ("Tree_time", "Time tree", "tevent"),
            ("Branch_time", "Time branch", "photon_count_data"),
        )
    }

    fn calculate(&self, args:CalculationNodeArguments) -> abi_stable::std_types::RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }

}
