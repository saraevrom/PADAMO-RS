use abi_stable::{sabi_trait, std_types::RBox};


#[sabi_trait]
pub trait DoubleFunctionOperator: Clone+Debug+Sync+Send{
    fn calculate(&self, x:f64)->f64;
}


pub type DoubleFunctionOperatorBox = DoubleFunctionOperator_TO<'static,RBox<()>>;


