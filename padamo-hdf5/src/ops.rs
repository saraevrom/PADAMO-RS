use std::cell::RefCell;
use std::sync::Mutex;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;

use abi_stable::std_types::RVec;
use hdf5::{Selection, SliceOrIndex};
use hdf5::Hyperslab;
use padamo_api::lazy_array_operations::{LazyArrayOperation,LazyArrayOperationBox};
use padamo_api::lazy_array_operations::ndim_array::ArrayND;
use std::ops::RangeFull;

fn make_slab(shapelen:usize,indexes:std::ops::Range<usize>)->hdf5::Selection{
    let slab:Vec<SliceOrIndex> = (0..shapelen).map(|i| if i==0 {indexes.clone().into()} else {(..).into()}).collect();
    //Selection::Hyperslab()
    let slab:Hyperslab = slab.into();
    slab.into()
}

#[derive(Clone,Debug)]
pub struct LazyHDF5Reader3D<T>
where
    T:Clone+Debug+abi_stable::StableAbi+hdf5::H5Type+Send+Sync,
    //D:ndarray::Dimension
{
    filename:String,
    dataset_path:String,
    hdf5_file: hdf5::File,
    dataset: hdf5::Dataset,
    _marker:PhantomData<T>,
    //cache:Mutex<Option<(usize,usize,ArrayND<T>)>>,
    //_dim_marker:PhantomData<D>,
}


impl<T> LazyHDF5Reader3D<T>
where
    T:Clone+Debug+Default+abi_stable::StableAbi+hdf5::H5Type+Send+Sync,
{
    pub fn new(filename:String, dataset_path:String)->Result<Self, hdf5::Error>{
        let hdf5_file = hdf5::File::open(&filename)?;
        let dataset = hdf5_file.dataset(&dataset_path)?;
        Ok(Self{filename, dataset_path, hdf5_file, dataset, _marker:PhantomData, })//cache:Mutex::new(None)})
    }

    pub fn check_read(&self)->bool{
        let l = self.dataset.shape().len();
        if let Ok(_) = self.dataset.read_slice::<T, Selection, ndarray::IxDyn>(make_slab(l, 0..1)){
            true
        }
        else{
            false
        }
    }

    pub fn print_dtype(&self){
        if let Ok(v) = self.dataset.dtype(){

            println!("{:?}",v.to_descriptor());
            println!("{:?}",self.dataset.shape());
        }
    }
}

impl<T> LazyArrayOperation<ArrayND<T>> for LazyHDF5Reader3D<T>
where
    T:Clone+Debug+Default+abi_stable::StableAbi+hdf5::H5Type+Send+Sync
{
    fn length(&self) -> usize where {
        let shape = self.dataset.shape();
        if shape.len()==0{
            0
        }
        else{
            shape[0]
        }
    }

    fn request_range(&self,start:usize,end:usize) -> ArrayND<T>{
        // let mut cache = self.cache.borrow_mut();
        // if let Some((old_start,old_end,old_data)) = cache.as_ref(){
        //     if *old_start==start && *old_end==end{
        //         return old_data.clone();
        //     }
        // }
        //let sliced:ndarray::Array3<T> = self.dataset.read_slice((start..end,..,..)).unwrap();
        let sliced = self.dataset.read_slice::<T, Selection, ndarray::IxDyn>(make_slab(self.dataset.shape().len(), start..end)).unwrap();

        let mut shape = vec![end-start];
        let shape_add:Vec<_> = self.dataset.shape().iter().skip(1).map(|x| *x).collect();
        shape.extend(shape_add);
        //let flat:Vec<T> = sliced.into();
        let flat_data:Vec<T> = sliced.into_raw_vec();
        let res = ArrayND{shape:shape.into(), flat_data:flat_data.into()};
        //*cache = Some((start,end,res.clone()));
        res
    }
}



#[derive(Clone,Debug)]
pub struct LazyTimeHDF5Reader<T>
where
    T:Clone+Debug+abi_stable::StableAbi+hdf5::H5Type,
    //D:ndarray::Dimension
{
    filename:String,
    dataset_path:String,
    hdf5_file: hdf5::File,
    dataset: hdf5::Dataset,
    _marker:PhantomData<T>,
    is_matlab:bool,
    //_dim_marker:PhantomData<D>,
}

#[derive(Clone,Debug)]
pub enum ReaderCreationError{
    HDFError(hdf5::Error),
    //CheckError,
    TimeFormatError,
}

impl Display for ReaderCreationError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            Self::HDFError(e)=>{
                std::fmt::Display::fmt(&e, f)
            }
            // Self::CheckError=>{
            //     write!(f, "Could not check shapes")
            // }
            Self::TimeFormatError=>write!(f,"Unsupported time format")
        }
    }
}

impl<T> LazyTimeHDF5Reader<T>
where
    T:Clone+Debug+Default+abi_stable::StableAbi+hdf5::H5Type,
{
    pub fn new(filename:String, dataset_path:String)->Result<Self, ReaderCreationError>{
        let hdf5_file = hdf5::File::open(&filename).map_err(ReaderCreationError::HDFError)?;
        let dataset = hdf5_file.dataset(&dataset_path).map_err(ReaderCreationError::HDFError)?;
        let s = dataset.shape();
        //let totalshape = s.iter().fold(1, |a,b|a*b);
        let mut is_matlab = false;
        if s.len()>2{
            return Err(ReaderCreationError::TimeFormatError);
        }
        if s.len()>1{
            is_matlab = true;
            println!("MATLAB time detected");
        }
        Ok(Self{filename, dataset_path, hdf5_file, dataset, _marker:PhantomData, is_matlab})
    }

    pub fn check_read(&self)->bool{
        if self.is_matlab{
            if let Ok(_)= self.dataset.read_slice_2d::<T, _>((0..1,..)) {true} else {false}
            //let shape:Vec<usize> = self.dataset.shape().iter().skip(1).map(|x| *x).collect();
            //let flat:Vec<T> = sliced.into();

        }
        else{
            if let Ok(_) = self.dataset.read_slice_1d::<T, _>(0..1) {true} else {false}
            //let shape:Vec<usize> = self.dataset.shape().iter().skip(1).map(|x| *x).collect();
            //let flat:Vec<T> = sliced.into();
        }
    }

    pub fn print_dtype(&self){
        println!("{:?}",self.dataset.dtype());
    }
}

impl<T> LazyArrayOperation<RVec<T>> for LazyTimeHDF5Reader<T>
where
    T:Clone+Debug+Default+abi_stable::StableAbi+hdf5::H5Type+Send+Sync
{
    fn length(&self) -> usize where {
        let length = self.dataset.shape().iter().fold(1, |a,b|a*b);
        length
    }


    fn request_range(&self,start:usize, end:usize) -> RVec<T>{
        if self.is_matlab{
            let sliced = self.dataset.read_slice_2d::<T,_>((start..end,..)).unwrap();
            //let shape:Vec<usize> = self.dataset.shape().iter().skip(1).map(|x| *x).collect();
            //let flat:Vec<T> = sliced.into();
            let flat_data = sliced.into_raw_vec();
            flat_data.into()
        }
        else{
            let sliced = self.dataset.read_slice_1d::<T,_>(start..end).unwrap();
            //let shape:Vec<usize> = self.dataset.shape().iter().skip(1).map(|x| *x).collect();
            //let flat:Vec<T> = sliced.into();
            let flat_data = sliced.into_raw_vec();
            flat_data.into()
        }

    }
}

#[derive(Clone,Debug)]
pub struct Caster<T>(LazyArrayOperationBox<T>);

impl<T> Caster<T>{
    pub fn new(src: LazyArrayOperationBox<T>) -> Self{
        Self(src)
    }
}

impl<T: Clone+Debug+Into<U>,U> LazyArrayOperation<U> for Caster<T>{
    fn length(&self) -> usize where {
        self.0.length()
    }

    fn request_range(&self,start:usize, end:usize) -> U{
        let tmp:T = self.0.request_range(start,end);
        return tmp.into();

    }
}


#[derive(Clone,Debug)]
pub struct ArrayCaster<T: abi_stable::StableAbi+Clone>(LazyArrayOperationBox<ArrayND<T>>);

impl<T:Clone+abi_stable::StableAbi> ArrayCaster<T>{
    pub fn new(src: LazyArrayOperationBox<ArrayND<T>>) -> Self{
        Self(src)
    }
}

impl<T: Clone+Debug+abi_stable::StableAbi,U: abi_stable::StableAbi+Clone+From<T>> LazyArrayOperation<ArrayND<U>> for ArrayCaster<T>{
    fn length(&self) -> usize where {
        self.0.length()
    }

    fn request_range(&self,start:usize, end:usize) -> ArrayND<U>{
        let tmp:ArrayND<T> = self.0.request_range(start,end);
        return tmp.cast();

    }
}


#[derive(Clone,Debug)]
pub struct UnsignedToFloatArrayCaster(LazyArrayOperationBox<ArrayND<u64>>);

impl UnsignedToFloatArrayCaster{
    pub fn new(src: LazyArrayOperationBox<ArrayND<u64>>) -> Self{
        Self(src)
    }
}

impl LazyArrayOperation<ArrayND<f64>> for UnsignedToFloatArrayCaster{
    fn length(&self) -> usize where {
        self.0.length()
    }

    fn request_range(&self,start:usize, end:usize) -> ArrayND<f64>{
        let tmp:ArrayND<u64> = self.0.request_range(start,end);
        let shape = tmp.shape;
        let mut old_flat_data = tmp.flat_data;
        let flat_data:RVec<f64> = old_flat_data.drain(..).map(|x| x as f64).collect();
        ArrayND { flat_data, shape }

    }
}
