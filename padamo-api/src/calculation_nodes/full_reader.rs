use std::fmt::Debug;

use crate::constants;
use crate::lazy_array_operations::LazyArrayOperation;
use crate::lazy_array_operations::LazyArrayOperationBox;
use crate::prelude::*;
use crate::ports;
use abi_stable::std_types::RResult;
use abi_stable::std_types::RString;
use abi_stable::std_types::RVec;

#[derive(Clone,Debug)]
pub struct FullReaderNode;

fn read_full<T,U>(arg:LazyArrayOperationBox<U>)->LazyArrayOperationBox<T>
where
    T:Clone+Debug+Send+Sync,
    U:Clone+Debug+Send+Sync+LazyArrayOperation<T>+'static
{
    let len = arg.length();
    let full = arg.request_range(0,len);
    make_lao_box(full)
}

impl FullReaderNode{
    fn calculate(&self,args:CalculationNodeArguments,) -> Result<(),ExecutionError>{
        let mut signal = args.inputs.request_detectorfulldata("Signal")?;
        //signal.0 = make_lao_box()
        signal.0 = read_full(signal.0);
        signal.1 = read_full(signal.1);
        args.outputs.set_value("Signal", signal.into())
    }
}

impl CalculationNode for FullReaderNode{
    #[allow(clippy::let_and_return)]
    #[doc = " Name of node displayed in graph editor or node list"]
    fn name(&self,) -> RString {
        "Full data reader".into()
    }

    fn identifier(&self,) -> RString {
        "builtin.full_reader".into()
    }

    fn category(&self,) -> RVec<RString>where {
        vec![].into()
    }

    #[allow(clippy::let_and_return)]
    #[doc = " Input definitions of node"]
    fn inputs(&self,) -> RVec<CalculationIO>{
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    #[allow(clippy::let_and_return)]
    #[doc = " Output definition of node"]
    fn outputs(&self,) -> RVec<CalculationIO>{
        ports!(
            ("Signal", ContentType::DetectorFullData)
        )
    }

    #[allow(clippy::let_and_return)]
    #[doc = " Constants definition of node with default values."]
    fn constants(&self,) -> RVec<CalculationConstant>{
        constants!()
    }

    #[allow(clippy::let_and_return)]
    #[doc = " Main calculation"]
    fn calculate(&self,args:CalculationNodeArguments,) -> RResult<(),ExecutionError>{
        self.calculate(args).into()
    }
}
