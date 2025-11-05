// pub mod ndim_array;
// pub mod indexing;
// pub use ndim_array::ArrayND;
pub mod lao;
//
// #[cfg(feature = "nalgebra")]
// pub mod nalgebra_support;

use abi_stable::StableAbi;
pub use padamo_arraynd::ArrayND;
pub use padamo_arraynd::indexing::ShapeIterator;

use crate::lazy_array_operations::{cache::Cache, merge::Merge};


impl<T> Merge for ArrayND<T>
where
T:StableAbi+Clone
{
    fn merge(self,other:Self)->Self {
        if !self.partial_compatible(&other,1){
            panic!("Cannot merge arrays");
        }
        let mut new_shape = self.shape;
        new_shape[0]+= other.shape[0];
        let mut flat_data = self.flat_data;
        flat_data.extend(other.flat_data);
        let res = Self { flat_data, shape: new_shape };
        //res.assert_shape();
        res
    }
}


impl<T> Cache for ArrayND<T>
where
T:StableAbi+Clone
{

    fn cut_front(self,count:usize)->Self {
        let mut new_shape = self.shape.clone();
        new_shape[0] = new_shape[0]-count;
        let step = self.shape.iter().skip(1).fold(1, |a,b| {a*b});
        let start_remap = step*count;
        let mut new_flat = self.flat_data;
        new_flat.drain(..start_remap);
        let res = ArrayND{flat_data:new_flat, shape:new_shape};
        res.assert_shape();
        res
    }

    fn cut_end(self,count:usize)->Self {
        let mut new_shape = self.shape.clone();
        new_shape[0] = new_shape[0]-count;
        let step = self.shape.iter().skip(1).fold(1, |a,b| {a*b});
        let end_remap = new_shape[0]*step;
        let mut new_flat = self.flat_data;
        new_flat.drain(end_remap..);
        let res = ArrayND{flat_data:new_flat, shape:new_shape};
        res.assert_shape();
        res
    }

    fn prepend(self,data:Self)->Self {
        data.merge(self)
    }

    fn append(self,data:Self)->Self {
        self.merge(data)
    }

}
