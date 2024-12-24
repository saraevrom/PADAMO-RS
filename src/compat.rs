use abi_stable::StableAbi;
use padamo_api::lazy_array_operations::ArrayND;
use ndarray::OwnedRepr;
use ndarray::IxDyn;

// Old implementations of conversion functions.
// They will not be used when HDF 5 crate updates to ndarray 0.16.

#[allow(dead_code)]
pub fn ndarray_to_arraynd<T,D>(value: ndarray::ArrayBase<OwnedRepr<T>,D>) -> ArrayND<T>
where
    T: Clone+StableAbi,
    D: ndarray::Dimension
{
    let shape:Vec<usize> = value.shape().into();
    let flat_data= value.into_raw_vec();
    ArrayND { flat_data:flat_data.into(), shape: shape.into() }
}


#[allow(dead_code)]
pub fn arraynd_to_ndarray<T>(value:ArrayND<T>)->ndarray::ArrayBase<OwnedRepr<T>,IxDyn>
where
    T: Clone+StableAbi,
{
    let shape = IxDyn(&value.shape.to_vec());
    //println!("CONV {:?} {:?}",self.shape,&self.flat_data.len());
    ndarray::ArrayBase::from_shape_vec(shape, value.flat_data.to_vec()).unwrap()
}
