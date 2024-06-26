use std::fmt::Debug;
use std::clone::Clone;
use dyn_clone::DynClone;
use iced::widget::shader::wgpu::naga::proc::index;
use crate::polygon::DetectorPixel;

pub trait Transformable{
    fn offset(&mut self,offset:(f64,f64));
    fn rotate(&mut self,angle:f64);

}

pub trait Indexed{
    fn set_index(&mut self, index:Vec<usize>);
    fn get_index(&self)->Vec<usize>;
}

pub trait PixelMaker{
    fn get_pixels(&self)->Vec<DetectorPixel>;
}


#[derive(Debug,Clone)]
pub struct PolygonArray(pub Vec<(f64,f64)>);

impl Transformable for PolygonArray{
    fn offset(&mut self,offset:(f64,f64)){
        for pos in self.0.iter_mut(){
            pos.0 += offset.0;
            pos.1 += offset.1;
        }
    }

    fn rotate(&mut self, angle:f64){
        let a11 = angle.cos();
        let a12 = -angle.sin();
        let a21 = angle.sin();
        let a22 = angle.cos();
        for pos in self.0.iter_mut(){
            let x = pos.0;
            let y = pos.1;
            *pos = (x*a11+y*a12,x*a21+y*a22);
        }
    }
}

impl From<Vec<(f64,f64)>> for PolygonArray{
    fn from(value: Vec<(f64,f64)>) -> Self {
        Self(value)
    }
}

#[derive(Debug,Clone)]
pub struct SinglePixel{
    pub index:Vec<usize>,
    pub polygon:PolygonArray,
}

impl PixelMaker for SinglePixel{
    fn get_pixels(&self)->Vec<DetectorPixel> {
        vec![DetectorPixel::new(self.index.clone(), self.polygon.0.clone())]
    }
}

impl Transformable for SinglePixel{
    fn offset(&mut self,offset:(f64,f64)) {
        self.polygon.offset(offset);

    }

    fn rotate(&mut self, angle:f64){
        self.polygon.rotate(angle);
        //self.polygon = self.polygon.rotated(angle);
    }
}

impl Indexed for SinglePixel{
    fn set_index(&mut self, index:Vec<usize>){
        self.index = index;
    }

    fn get_index(&self)->Vec<usize>{
        self.index.clone()
    }
}


pub trait TransformablePixelMaker: PixelMaker+Transformable+Debug+DynClone+Indexed{}
dyn_clone::clone_trait_object!(TransformablePixelMaker);

impl<T: ?Sized> TransformablePixelMaker for T where T: PixelMaker+Transformable+Debug+DynClone+Indexed {}


#[derive(thiserror::Error,Debug)]
pub enum PixelGridError{
    #[error("Invalid X axis indices: {0}:{1}:{2}")]
    XInvalid(usize,usize,usize),
    #[error("Invalid Y axis indices: {0}:{1}:{2}")]
    YInvalid(usize,usize,usize),
}

#[derive(Debug,Clone)]
pub struct PixelGrid{
    pub lowers:(usize,usize),
    pub uppers:(usize,usize),
    pub steps:(usize,usize),
    pub base:(f64,f64),
    pub ax_x:(f64,f64),
    pub ax_y:(f64,f64),
    pub subpixel: Box<dyn TransformablePixelMaker>,
    pub base_index:Vec<usize>,
}


impl PixelMaker for PixelGrid{
    fn get_pixels(&self)->Vec<DetectorPixel> {
        let mut res = vec![];
        for i in (self.lowers.0..self.uppers.0).step_by(self.steps.0){
            for j in (self.lowers.1..self.uppers.1).step_by(self.steps.1){
                let mut pix = self.subpixel.clone();
                let i_delta:f64 = (i-self.lowers.0) as f64 / self.steps.0 as f64;
                let j_delta:f64 = (j-self.lowers.1) as f64 / self.steps.1 as f64;

                let x = self.base.0+self.ax_x.0*(i_delta as f64)+self.ax_y.0*(j_delta as f64);
                let y = self.base.1+self.ax_x.1*(i_delta as f64)+self.ax_y.1*(j_delta as f64);
                let mut index = self.base_index.clone();
                if let Some(v) = index.get_mut(0){
                    *v+=i;
                }
                if let Some(v) = index.get_mut(1){
                    *v+=j;
                }
                pix.offset((x,y));
                pix.set_index(index);
                let pixels = pix.get_pixels();
                res.extend(pixels);
            }
        }
        res
    }
}

impl Transformable for PixelGrid{
    fn offset(&mut self,offset:(f64,f64)) {
        self.base.0 += offset.0;
        self.base.1 += offset.1;
    }

    fn rotate(&mut self,angle:f64) {
        let a11 = angle.cos();
        let a12 = -angle.sin();
        let a21 = angle.sin();
        let a22 = angle.cos();

        let x = self.ax_x.0;
        let y = self.ax_x.1;
        self.ax_x = (x*a11+y*a12,x*a21+y*a22);

        let x = self.ax_y.0;
        let y = self.ax_y.1;
        self.ax_y = (x*a11+y*a12,x*a21+y*a22);
        self.subpixel.rotate(angle);

        let x = self.base.0;
        let y = self.base.1;
        self.base = (x*a11+y*a12,x*a21+y*a22);
    }
}

impl Indexed for PixelGrid{
    fn set_index(&mut self, index:Vec<usize>) {
        self.base_index = index;
    }

    fn get_index(&self)->Vec<usize> {
        self.base_index.clone()
    }

}

impl PixelGrid{
    pub fn new(lowers:(usize,usize),uppers:(usize,usize),steps:(usize,usize),ax_x:(f64,f64),ax_y:(f64,f64),subpixel:Box<dyn TransformablePixelMaker>)->Result<Self,PixelGridError>{
        if lowers.0>=uppers.0 || steps.0==0{
            return Err(PixelGridError::XInvalid(lowers.0,uppers.0,steps.0));
        }
        if lowers.1>=uppers.1 || steps.1==0{
            return Err(PixelGridError::YInvalid(lowers.1,uppers.1,steps.1));
        }
        Ok(Self { lowers, uppers, steps, base: (0.0,0.0), ax_x, ax_y, base_index:subpixel.get_index(), subpixel })
    }
}


#[derive(Clone,Debug)]
pub enum IndexExtension {
    Prepend(Vec<usize>),
    Append(Vec<usize>),
}

impl IndexExtension{
    pub fn modify_index(&self, index:&Vec<usize>)->Vec<usize>{
        match &self{
            IndexExtension::Prepend(pre)=>{
                let mut index2 = pre.clone();
                index2.extend(index.clone());
                index2
            }
            IndexExtension::Append(add)=>{
                let mut index2 = index.clone();
                index2.extend(add.clone());
                index2
            }
        }
    }

    pub fn bite_index(&mut self, index:&Vec<usize>)->Vec<usize>{
        match self{
            IndexExtension::Prepend(ref mut pre)=>{
                let prelen = pre.len();
                *pre = index[0..prelen].to_vec();
                index[prelen..].to_vec()
            }
            IndexExtension::Append(ref mut add)=>{
                let addlen = add.len();
                let index_len = index.len();
                let start_index = index_len-addlen;
                *add = index[start_index..].to_vec();
                index[0..start_index].to_vec()
            }
        }
    }
}

#[derive(Clone,Debug)]
pub struct IndexExtend{
    pub source: Box<dyn TransformablePixelMaker>,
    pub extension:IndexExtension
}

impl IndexExtend{
    pub fn prepend(index:Vec<usize>,source:Box<dyn TransformablePixelMaker>)->Self{
        Self { source, extension: IndexExtension::Prepend(index) }
    }

    pub fn append(index:Vec<usize>,source:Box<dyn TransformablePixelMaker>)->Self{
        Self { source, extension: IndexExtension::Append(index) }
    }
}

impl PixelMaker for IndexExtend{
    fn get_pixels(&self)->Vec<DetectorPixel> {
        let mut pixels = self.source.get_pixels();
        pixels.iter_mut().for_each(|pix|{
            pix.index = self.extension.modify_index(&pix.index)
        });
        pixels
    }
}

impl Transformable for IndexExtend{
    fn offset(&mut self,offset:(f64,f64)) {
        self.source.offset(offset)
    }
    fn rotate(&mut self,angle:f64) {
        self.source.rotate(angle)
    }
}

impl Indexed for IndexExtend{
    fn get_index(&self) ->Vec<usize>{
        self.extension.modify_index(&self.source.get_index())
    }

    fn set_index(&mut self, index:Vec<usize>) {
        let rest = self.extension.bite_index(&index);
        self.source.set_index(rest);
    }
}
