use abi_stable::{StableAbi, std_types::{RVec, Tuple2, ROption}, rvec};
use ndarray::{IxDyn, OwnedRepr};
//use ndarray::{IxDyn, OwnedRepr};
use std::ops::{Add, IndexMut, Index};
//use ndarray::prelude::*;
use crate::lazy_array_operations::merge::Merge;
use crate::lazy_array_operations::cache::Cache;
use super::indexing::ShapeIterator;


/// FFI safe ndarray-like structure
#[repr(C)]
#[derive(Clone,Debug,StableAbi)]
pub struct ArrayND<T>
where
    T: Clone + StableAbi
{
    pub flat_data:RVec<T>,
    pub shape:RVec<usize>,
}


impl<T> ArrayND<T>
where
    T: Clone + StableAbi
{
    pub fn new(shape:Vec<usize>,fill_value:T)->Self{
        let capacity:usize = shape.iter().fold(1, |a,b| a*b);
        let mut flat_data:RVec<T> = RVec::with_capacity(capacity);
        flat_data.resize(capacity, fill_value);
        // for _ in 0..capacity{
        //     flat_data.push(fill_value.clone());
        // }
        //flat_data.fill(fill_value);
        Self { flat_data, shape:shape.into() }
    }

    pub fn try_get<'a>(&self, index:&[usize])->Option<&T>{
        if self.index_compatible(index){
            let index_flat = self.remap_indices(index);
            Some(&self.flat_data[index_flat])
        }
        else{
            None
        }
    }

    pub fn defaults<U:Clone + StableAbi+Default>(shape:Vec<usize>)->ArrayND<U>{
        ArrayND::new(shape,U::default())
    }

    pub fn extract_dataset(&self, reference:&Vec<usize>)->Vec<T>{
        let mut index = vec![0usize];
        index.extend_from_slice(reference);
        let length = self.shape[0];
        let mut res = Vec::with_capacity(length);
        for i in 0..length{
            index[0] = i;
            let v = self[&index].clone();
            res.push(v);
        }
        res
    }

    fn remap_indices(&self,indices:&[usize])->usize{
        if indices.len()!=self.shape.len(){
            panic!("Incompatible shapes")
        }
        let mut multiplier:usize = 1;
        let mut res:usize = 0;
        for (i,index) in indices.iter().enumerate().rev(){
            res += index*multiplier;
            multiplier*=self.shape[i];
        }
        //println!("IN indices {:?} ,out index: {:?}",indices, res);
        res
    }

    pub fn index_compatible(&self, index:&[usize])->bool{
        if index.len()==self.shape.len(){
            index.iter().zip(self.shape.iter()).fold(true,|a,b| a&&(b.0<b.1))
        }
        else{
            false
        }
    }

    pub fn is_compatible<U:Clone+StableAbi>(&self, other:&ArrayND<U>)->bool{
        if self.shape.len()!=other.shape.len(){
            false
        }
        else{
            (0..self.shape.len()).fold(true, |a,b| a&&(self.shape[b]==other.shape[b]))
        }
    }

    pub fn partial_compatible<U:Clone+StableAbi>(&self, other:&ArrayND<U>,start_index:usize)->bool{
        if self.shape.len()!=other.shape.len(){
            false
        }
        else{
            (start_index..self.shape.len()).fold(true, |a,b| a&&(self.shape[b]==other.shape[b]))
        }
    }

    pub fn form_compatible(&self, shape: &Vec<usize>)->bool{
        if self.shape.len()!= shape.len(){
            false
        }
        else{
            (0..self.shape.len()).fold(true, |a,b| a&&(self.shape[b]==shape[b]))
        }
    }

    pub fn enumerate(&self)->ShapeIterator{
        ShapeIterator::new(self.shape.clone().into())
    }

    pub fn set(&mut self, index:&[usize], value:T){
        let index_flat = self.remap_indices(index);
        self.flat_data[index_flat] = value;
    }

    pub fn to_ndarray(self)->ndarray::ArrayBase<OwnedRepr<T>,IxDyn>{
        let shape = IxDyn(&self.shape.to_vec());
        //println!("CONV {:?} {:?}",self.shape,&self.flat_data.len());
        ndarray::ArrayBase::from_shape_vec(shape, self.flat_data.to_vec()).unwrap()
    }

    pub fn flatten(self)->Self{
        let length = self.shape.into_iter().fold(1, |a,b| a*b);
        let shape = rvec![length];
        Self { flat_data: self.flat_data, shape  }
    }

    pub fn squeeze(self)->Self{
        let flat_data = self.flat_data;
        let shape:Vec<usize> = self.shape.into_iter().filter(|x| *x!=1).collect();
        Self { flat_data, shape:shape.into() }
    }


    pub fn assert_shape(&self){
        let capacity:usize = self.shape.iter().fold(1, |a,b| a*b);
        if capacity!=self.flat_data.len(){
            println!("Array assert error {}!={}",capacity,self.flat_data.len());
            panic!("Array assert error");
        }
    }


    pub fn cast<U:From<T>+Clone+abi_stable::StableAbi>(self)->ArrayND<U>{
        let shape = self.shape;
        let mut old_flat_data = self.flat_data;
        let flat_data:RVec<U> = old_flat_data.drain(..).map(|x| x.into()).collect();
        ArrayND { flat_data, shape }
    }

    pub fn stack(data:&[ArrayND<T>])-> ArrayND<T> {
        if data.len()==0{
            ArrayND{flat_data:RVec::new(),shape:rvec![]}
        }
        else {
            let src_shape = data[0].shape.clone();
            let frame_size = src_shape.iter().fold(1,|a,b| a*(*b));
            let mut tgt_shape = RVec::with_capacity(src_shape.len()+1);
            tgt_shape.push(data.len());
            tgt_shape.extend(src_shape);
            let mut flat_data = RVec::with_capacity(data.len()*frame_size);
            for x in data.iter(){
                flat_data.extend(x.flat_data.clone());
            }
            //let flat_data = data.iter().fold(RVec::with_capacity(data.len()), |a,b| a.extend(b.flat_data.clone()));
            ArrayND{flat_data,shape:tgt_shape}
        }
    }

}


impl<T,D> From<ndarray::ArrayBase<OwnedRepr<T>,D>> for ArrayND<T>
where
    T: Clone+StableAbi,
    D: ndarray::Dimension
{
    fn from(value: ndarray::ArrayBase<OwnedRepr<T>,D>) -> Self {
        let shape:Vec<usize> = value.shape().into();
        let flat_data:Vec<T> = value.into_raw_vec();

        Self { flat_data:flat_data.into(), shape: shape.into() }
    }
}


impl<T> Index<&Vec<usize>> for ArrayND<T>
where
    T:Clone+StableAbi
{
    type Output = T;

    fn index(&self, index: &Vec<usize>) -> &Self::Output {
        let index_flat = self.remap_indices(index);
        &self.flat_data[index_flat]
    }
}

impl<T> IndexMut<&Vec<usize>> for ArrayND<T>
where
    T:Clone+StableAbi
{
    fn index_mut(&mut self, index: &Vec<usize>) -> &mut Self::Output {
        let index_flat = self.remap_indices(index);
        &mut self.flat_data[index_flat]
    }
}

impl<T> Index<&[usize]> for ArrayND<T>
where
    T:Clone+StableAbi
{
    type Output = T;

    fn index(&self, index: &[usize]) -> &Self::Output {
        let index_flat = self.remap_indices(index);
        &self.flat_data[index_flat]
    }
}

impl<T> IndexMut<&[usize]> for ArrayND<T>
where
    T:Clone+StableAbi
{
    fn index_mut(&mut self, index: &[usize]) -> &mut Self::Output {
        let index_flat = self.remap_indices(index);
        &mut self.flat_data[index_flat]
    }
}



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

