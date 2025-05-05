use abi_stable::{rvec, std_types::{ROption::RNone, RVec}};
use padamo_api::{constants, lazy_array_operations::{LazyDetectorSignal, LazyTriSignal}, ports, prelude::*};

#[derive(Clone,Debug)]
pub struct CSVArrayNode;


impl CSVArrayNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {
        let start = args.constants.request_integer("start")?;
        let length = args.constants.request_integer("length")?;
        let start:usize = start.try_into().ok().unwrap_or(0);
        let length:Option<usize> = length.try_into().ok();
        let input_file = args.inputs.request_string("Filename")?.to_string();
        let separator = args.constants.request_string("separator")?.to_string();
        let start_column:Option<usize> = args.constants.request_integer("col_start")?.try_into().ok();
        let end_column:Option<usize> = args.constants.request_integer("col_end")?.try_into().ok();

        let spatial = if args.constants.request_boolean("transpose")?{
            let sp = crate::ops_transposed::CSVReaderTransposed::new(separator, input_file, start, length,start_column, end_column).map_err(ExecutionError::from_error)?;
            if sp.frame_size==0{
                return Err(ExecutionError::OtherError("No spatial data".into()));
            }
            make_lao_box(sp)
        }
        else{
            let sp = crate::ops::CSVReader::new(separator, input_file, start, length,start_column, end_column).map_err(ExecutionError::from_error)?;
            if sp.frame_size==0{
                return Err(ExecutionError::OtherError("No spatial data".into()));
            }
            make_lao_box(sp)
        };

        let signal:LazyDetectorSignal = spatial.into();
        args.outputs.set_value("Signal", signal.into())
    }
}

impl CalculationNode for CSVArrayNode{
    fn name(&self,) -> abi_stable::std_types::RString where {
        "CSV Array".into()
    }

    fn category(&self,) -> abi_stable::std_types::RVec<abi_stable::std_types::RString>where {
        padamo_api::common_categories::array_sources()
    }

    fn identifier(&self,) -> abi_stable::std_types::RString where {
        "padamoplaintext.read_csv_array".into()
    }


    fn inputs(&self,) -> abi_stable::std_types::RVec<CalculationIO>where {
        ports![
            ("Filename", ContentType::String)
        ]
    }

    fn outputs(&self) -> RVec<CalculationIO> {
        ports!(
            ("Signal", ContentType::DetectorSignal)
        )
    }

    fn constants(&self,) -> RVec<CalculationConstant>where {
        // let now = chrono::Utc::now();
        // let dat = now.format("%Y-%m-%d %H:%M:%S.0");
        // let formatted = format!("{}", dat);
        constants!(
            ("transpose", "Read transposed",false),
            ("separator","Separator", r",\s*"),
            ("start","Start line", 0),
            ("length","Length", -1),

            ("col_start","Start column",0),
            ("col_end","End column", -1),

            // ("tmpbase","Temporal unixtime base", formatted),
            // ("tmpres","Temporal resolution", 0.000256),
        )
    }

    fn calculate(&self, args:CalculationNodeArguments) -> abi_stable::std_types::RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}


#[derive(Clone,Debug)]
pub struct CSVTimeNode;


impl CSVTimeNode{
    fn calculate(&self, args:CalculationNodeArguments) -> Result<(),ExecutionError>where {

        let skip = args.constants.request_integer("skip_time")?;
        let skip:usize = skip.try_into().ok().unwrap_or(0);

        let length = args.constants.request_integer("length_time")?;
        let length:Option<usize> = length.try_into().ok();

        let input_file = args.inputs.request_string("Filename")?.to_string();
        let separator = args.constants.request_string("separator_time")?.to_string();

        let item = args.constants.request_integer("item_time")?;
        let item :usize = item .try_into().ok().unwrap_or(0);

        let time = if args.constants.request_boolean("transpose_time")?{
            let lower_bound = Some(skip);
            let upper_bound = length.map(|x|x+skip);
            let t = crate::ops_temporal::CSVTimeRowReader::new(separator, input_file, item, lower_bound, upper_bound)
                .map_err(ExecutionError::from_error)?;
            make_lao_box(t)
        }
        else{
            let t = crate::ops_temporal::CSVTimeColumnReader::new(separator, input_file, skip, length, item)
                .map_err(ExecutionError::from_error)?;
            make_lao_box(t)
        };
        args.outputs.set_value("Time", time.into())

    }
}

impl CalculationNode for CSVTimeNode{
    fn name(&self,) -> abi_stable::std_types::RString where {
        "CSV Time".into()
    }

    fn category(&self,) -> abi_stable::std_types::RVec<abi_stable::std_types::RString>where {
        padamo_api::common_categories::time_sources()
    }

    fn identifier(&self,) -> abi_stable::std_types::RString where {
        "padamoplaintext.read_csv_time".into()
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
            ("transpose_time", "(Time) Read row instead of column",false),
            ("item_time", "(Time) Column (or row)",0),
            ("separator_time","(Time) Separator", r",\s*"),
            ("skip_time", "(Time) Skip items",0),
            ("length_time", "(Time) Length", -1),
        )
    }

    fn calculate(&self, args:CalculationNodeArguments) -> abi_stable::std_types::RResult<(),ExecutionError>where {
        self.calculate(args).into()
    }
}
