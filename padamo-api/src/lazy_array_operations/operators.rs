use std::ops::{Add, Sub, Mul, Div};

use abi_stable::{StableAbi, std_types::RVec};

use super::ndim_array::ArrayND;


macro_rules! implement_elementwise_ndarray {
    ($traitid:ident, $inner_funcname:ident) => {
        impl<T,U> $traitid<ArrayND<U>> for ArrayND<T>
        where
            T:Clone+StableAbi+$traitid<U, Output=T>,
            U:Clone+StableAbi
        {
            type Output = ArrayND<T>;

            fn $inner_funcname(self, rhs: ArrayND<U>) -> Self::Output {
                if self.is_compatible(&rhs){
                    let mut newdata:RVec<T> = self.flat_data;//RVec::with_capacity(self.flat_data.len());
                    let mut auxdata = rhs.flat_data;
                    for (i,v) in auxdata.drain(..).enumerate(){
                        newdata[i] = $traitid::$inner_funcname (newdata[i].clone(),v);
                        //newdata.push($traitid::$inner_funcname (self.flat_data[i].clone(), rhs.flat_data[i].clone()));
                        //newdata.push(a.flat_data[i].clone()+b.flat_data[i].clone())
                    }
                    return Self::Output{flat_data:newdata, shape:self.shape.clone()};
                }
                else{
                    panic!("Incompatible arrays operation")
                }
            }
        }
    };
}

implement_elementwise_ndarray!{Add, add}
implement_elementwise_ndarray!{Sub, sub}
implement_elementwise_ndarray!{Mul, mul}
implement_elementwise_ndarray!{Div, div}
