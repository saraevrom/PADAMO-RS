use abi_stable::{sabi_trait, std_types::RBox};
use abi_stable::sabi_trait::prelude::TD_Opaque;


#[sabi_trait]
pub trait DoubleFunctionOperator: Clone+Debug+Sync+Send{
    fn calculate(&self, x:f64)->f64;
}


pub type DoubleFunctionOperatorBox = DoubleFunctionOperator_TO<'static,RBox<()>>;


pub fn make_function_box<T,U:DoubleFunctionOperator+'static>(data:U)->DoubleFunctionOperatorBox{
    DoubleFunctionOperatorBox::from_value(data, TD_Opaque)
}
