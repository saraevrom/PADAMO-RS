use abi_stable::{StableAbi, std_types::RVec, rvec};

#[cfg(feature = "ndarray")]
use ndarray::{IxDyn, OwnedRepr};

use std::fmt::Debug;
//use ndarray::{IxDyn, OwnedRepr};
use std::ops::{IndexMut, Index};
//use ndarray::prelude::*;
use super::indexing::ShapeIterator;

fn calculate_offset(shape: &[usize], indices: &[usize]) -> usize {
    if indices.len()!=shape.len(){
        panic!("Incompatible shapes")
    }
    let mut multiplier:usize = 1;
    let mut res:usize = 0;
    for (i,index) in indices.iter().enumerate().rev(){
        res += index*multiplier;
        multiplier*=shape[i];
    }
    //println!("IN indices {:?} ,out index: {:?}",indices, res);
    res
}

/// FFI safe ndarray-like structure
#[repr(C)]
#[derive(Clone,Debug,StableAbi)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArrayND<T>
where
    T: Clone + StableAbi
{
    pub flat_data:RVec<T>,
    pub shape:RVec<usize>,
    // f_size:usize,
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

    pub fn frame_size(&self)->usize{
        if self.shape.len()>0{
            let new_shape:Vec<usize> = self.shape.as_slice().iter().skip(1).copied().collect();
            new_shape.iter().product()
        }
        else{
            0
        }
    }

    pub fn try_get<'a>(&self, index:&[usize])->Option<&T>{
        if self.index_compatible(index){
            let index_flat = self.remap_indices(index);
            // Some(&self.flat_data[index_flat])
            self.flat_data.get(index_flat)
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
        calculate_offset(&self.shape, indices)
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

    pub fn form_compatible(&self, shape: &[usize])->bool{
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
        if index_flat<self.flat_data.len() {
            self.flat_data[index_flat] = value;
        }
    }


    #[cfg(feature = "ndarray")]
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

    pub fn flip_indices(&self)->ArrayND<T>{
        let shape:RVec<usize> = self.shape.as_ref().iter().cloned().rev().collect::<Vec<usize>>().into();
        let mut res = self.clone();
        res.shape = shape;
        for index in self.enumerate(){
            let target_index:Vec<usize> = index.iter().cloned().rev().collect();
            res.set(&target_index, self[&index].clone());
        }
        res
    }

    pub fn take_frame(&mut self)->Option<Self>{
        if self.shape.len()==0{
            return None;
        }
        if self.shape[0]==0{
            return None;
        }

        let new_shape:Vec<usize> = self.shape.as_slice().iter().skip(1).copied().collect();
        let frame_size:usize = new_shape.iter().product();
        // println!("Frame size {}", frame_size);

        // println!("Frame size {}", frame_size);

        let flat_part:RVec<T> = self.flat_data.drain(..frame_size).collect();
        if flat_part.len()==frame_size{
            self.shape[0] -= 1;
            Some(ArrayND { flat_data:flat_part, shape: new_shape.into() })
        }
        else{
            None
        }
    }

    pub fn make_pixel_iterators<'a>(&'a self)->ArrayND<ArrayStrideIterator<'a, T>>{
        if self.shape.len()<=1{
            panic!("This is a time-like array (1 or less dimensions)");
        }
        let new_shape:Vec<usize> = self.shape.iter().skip(1).map(|x|*x).collect();

        let stride = new_shape.iter().fold(1, |a,b| a*b);
        let mut res = ArrayND::new(new_shape.clone(), ArrayStrideIterator::new(self, 0, 0));
        for pixel_id in res.enumerate(){
            let offset = calculate_offset(&new_shape, &pixel_id);
            res[&pixel_id] = ArrayStrideIterator::new(self, stride, offset);
        }

        // let mut flat_data:Vec<ArrayStrideIterator<'a, T>> = Vec::with_capacity(stride);
        // for pixel_id in super::indexing::ShapeIterator::new(new_shape.clone()){
        //     let offset = calculate_offset(&new_shape, &pixel_id);
        //     flat_data.push(ArrayStrideIterator::new(self, stride, offset));
        // }

        //ArrayND{flat_data:flat_data.into(), shape: new_shape.into() }
        res
    }


}
impl<T> ArrayND<T>
where
    T: Clone + StableAbi + Copy
{
    pub fn fold_frames<F:Clone+Copy+FnMut(T,&T)->T>(&self, initial_value:T, f:F)->Vec<T>{
        if self.shape.is_empty(){
            return vec![];
        }
        let new_shape:Vec<usize> = self.shape.as_slice().iter().skip(1).copied().collect();
        let frame_size:usize = new_shape.iter().product();
        let mut res = Vec::with_capacity(self.shape[0]);
        for i in 0..self.shape[0]{
            res.push(self.flat_data[i*frame_size..(i+1)*frame_size].iter().fold(initial_value, f));
        }
        res
    }

    pub fn apply_on_frames<F:Clone+Copy+FnMut(&[T])->T>(&self, mut f:F)->Vec<T>{
        if self.shape.is_empty(){
            return vec![];
        }
        let new_shape:Vec<usize> = self.shape.as_slice().iter().skip(1).copied().collect();
        let frame_size:usize = new_shape.iter().product();
        let mut res = Vec::with_capacity(self.shape[0]);
        for i in 0..self.shape[0]{
            res.push(f(&self.flat_data[i*frame_size..(i+1)*frame_size]));
        }
        res
    }
}


#[cfg(feature = "ndarray")]
impl<T,D> From<ndarray::ArrayBase<OwnedRepr<T>,D>> for ArrayND<T>
where
    T: Clone+StableAbi,
    D: ndarray::Dimension
{
    fn from(value: ndarray::ArrayBase<OwnedRepr<T>,D>) -> Self {
        let value = value.as_standard_layout().to_owned(); // Standartize layout
        let shape:Vec<usize> = value.shape().into();
        let (flat_data,offset) = value.into_raw_vec_and_offset();
        if let Some(off) = offset{
            let mut flat_data = flat_data;
            flat_data.drain(..off);
            Self { flat_data:flat_data.into(), shape: shape.into() }
        }
        else{
            Self { flat_data:flat_data.into(), shape: shape.into() }
        }

    }
}

#[cfg(feature = "ndarray")]
impl<T> Into<ndarray::ArrayBase<OwnedRepr<T>,IxDyn>> for ArrayND<T>
where
T: Clone+StableAbi,
{
    fn into(self) -> ndarray::ArrayBase<OwnedRepr<T>, IxDyn> {
        self.to_ndarray()
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

#[repr(C)]
#[derive(Clone, StableAbi)]
pub struct ArrayStrideIterator<'a, T:Clone+StableAbi>{
    array: &'a ArrayND<T>,
    stride:usize,
    offset:usize,
}

impl<'a, T: Clone + StableAbi> ArrayStrideIterator<'a, T> {
    pub fn new(array: &'a ArrayND<T>, stride: usize, offset:usize) -> Self {
        Self { array, stride, offset }
    }
}


impl<'a, T: Clone + StableAbi> Iterator for ArrayStrideIterator<'a, T>{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let v = self.array.flat_data.get(self.offset);
        self.offset += self.stride;
        v.map(std::clone::Clone::clone)
    }
}
