use std::fmt::Debug;
use abi_stable::sabi_trait::prelude::TD_Opaque;
#[allow(non_local_definitions)]
pub mod traits{
    use abi_stable::{sabi_trait, std_types::RBox};

    #[sabi_trait]
    pub trait DoubleFunctionOperator: Clone+Debug+Sync+Send{
        fn calculate(&self, x:f64)->f64;
    }


    pub type DoubleFunctionOperatorBox = DoubleFunctionOperator_TO<'static,RBox<()>>;
}

pub use traits::{DoubleFunctionOperatorBox,DoubleFunctionOperator, DoubleFunctionOperator_TO};


#[derive(Clone)]
pub struct MapOperator<T:Fn(f64)->f64+Send+Sync+Clone>{
    pub func:T,
    pub parameter:DoubleFunctionOperatorBox
}

#[derive(Clone)]
pub struct InvMapOperator<T:Fn(f64)->f64+Send+Sync+Clone>{
    pub func:T,
    pub parameter:DoubleFunctionOperatorBox
}

#[derive(Clone)]
pub struct Map2Operator<T:Fn(f64,f64)->f64+Send+Sync+Clone>{
    pub func:T,
    pub parameter1:DoubleFunctionOperatorBox,
    pub parameter2:DoubleFunctionOperatorBox,
}


#[derive(Clone)]
pub struct WrappedDoubleFunction<T:Fn(f64)->f64+Send+Sync+Clone>{
    pub func:T
}

impl<T:Fn(f64)->f64+Send+Sync+Clone> Debug for MapOperator<T>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"MapOperator{{parameter: {:?}, func: <nondisplayable>}}",self.parameter)
    }
}

impl<T:Fn(f64)->f64+Send+Sync+Clone> Debug for InvMapOperator<T>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"InvMapOperator{{parameter: {:?}, func: <nondisplayable>}}",self.parameter)
    }
}

impl<T:Fn(f64,f64)->f64+Send+Sync+Clone> Debug for Map2Operator<T>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"MapOperator {{parameter1: {:?}, parameter2: {:?}, func: <nondisplayable>}}",self.parameter1,self.parameter2)
    }
}

impl<T:Fn(f64)->f64+Send+Sync+Clone> Debug for WrappedDoubleFunction<T>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"WrappedDoubleFunction{{ func: <nondisplayable>}}")
    }
}

impl<T:Fn(f64)->f64+Send+Sync+Clone> DoubleFunctionOperator for MapOperator<T>{
    fn calculate(&self,x:f64)->f64{
        let x1 = self.parameter.calculate(x);
        (self.func)(x1)
    }
}

impl<T:Fn(f64)->f64+Send+Sync+Clone> DoubleFunctionOperator for InvMapOperator<T>{
    fn calculate(&self,x:f64)->f64{
        let x1 = (self.func)(x);
        self.parameter.calculate(x1)
    }
}

impl <T:Fn(f64,f64)->f64+Send+Sync+Clone> DoubleFunctionOperator for Map2Operator<T>{
    fn calculate(&self,x:f64)->f64{
        let x1 = self.parameter1.calculate(x);
        let x2 = self.parameter2.calculate(x);
        (self.func)(x1,x2)
    }
}

impl <T:Fn(f64)->f64+Send+Sync+Clone> DoubleFunctionOperator for WrappedDoubleFunction<T> {
    fn calculate(&self,x:f64)->f64{
        (self.func)(x)
    }
}



impl DoubleFunctionOperatorBox{

    /// turns self(x) into f(self(x))
    pub fn map<T:Fn(f64)->f64+Send+Sync+Clone+'static>(self, f:T)->DoubleFunctionOperatorBox{
        make_function_box(MapOperator{func:f, parameter:self})
    }

    /// turns self(x) into self(f(x))
    pub fn invmap<T:Fn(f64)->f64+Send+Sync+Clone+'static>(self, f:T)->DoubleFunctionOperatorBox{
        make_function_box(InvMapOperator{func:f, parameter:self})
    }

    pub fn map2<T:Fn(f64, f64)->f64+Send+Sync+Clone+'static>(self, other:Self, f:T)->DoubleFunctionOperatorBox{
        make_function_box(Map2Operator{func:f, parameter1:self, parameter2:other})
    }
}

pub fn make_function_box<T:DoubleFunctionOperator+'static>(data:T)->DoubleFunctionOperatorBox{
    DoubleFunctionOperatorBox::from_value(data, TD_Opaque)
}


impl<T:Fn(f64)->f64+Send+Sync+Clone+'static> From<T> for DoubleFunctionOperatorBox{
    fn from(value: T) -> Self {
        let f: WrappedDoubleFunction<T> = WrappedDoubleFunction{func:value};
        make_function_box(f)
    }
}
