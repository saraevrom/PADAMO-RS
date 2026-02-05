use abi_stable::std_types::ROption::{RNone, RSome};
use abi_stable::std_types::{ROption, RString, RVec, Tuple2, Tuple3};
use serde::{Serialize,Deserialize};
use plotters::prelude::*;
use crate::{parser::parse_detector, scripted::parse_scripted};
use rhai::{serde::from_dynamic, CustomType, EvalAltResult, TypeBuilder};
use abi_stable::{rvec, StableAbi};

//use super::colors::



fn format_double(x:f64,round:Option<usize>)->String{
    match round {
        Some(v)=>format!("{:.1$}",x,v),
        None=>format!("{}",x),
    }
}

fn format_point(x:(f64,f64), round:Option<usize>)->String{
    format!("({}, {})",format_double(x.0,round),format_double(x.1,round))
}

fn do_vecs_match<T: PartialEq>(a: &[T], b: &[T]) -> bool {
    let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
    matching == a.len() && matching == b.len()
}

fn segment_intestects_rect(mut start:(f64,f64), mut end:(f64,f64), left:f64, right:f64, top:f64, bottom:f64)->bool{
    if start.0>end.0{
        (start.0, end.0) = (end.0, start.0);
    }

    if start.1>end.1{
        (start.1, end.1) = (end.1, start.1);
    }


    // Test if horizontal part if *NOT* intersected
    if start.0>right || end.0<left{
        return false;
    }

    // Test if vertical part is *NOT* intersected too
    if start.1>top || end.1<bottom{
        return false;
    }

    true
}

#[derive(Debug,Clone, PartialEq, CustomType, StableAbi)]
#[rhai_type(extra = Self::build_extra)]
#[repr(C)]
pub struct DetectorPixel{
    pub index:RVec<usize>,
    pub vertices:RVec<Tuple2<f64,f64>>,
    pub color:ROption<Tuple3<f32, f32, f32>>
}

#[derive(Serialize,Deserialize, )]
struct DetectorPixelProxy{
    pub index:Vec<usize>,
    pub vertices:Vec<(f64,f64)>,
    pub color:Option<(f32, f32, f32)>
}

impl Into<DetectorPixelProxy> for DetectorPixel{
    fn into(self) -> DetectorPixelProxy {
        DetectorPixelProxy {
            index: self.index.into_vec(),
            vertices: self.vertices.iter().map(|x| x.into_tuple()).collect(),
            color: self.color.into_option().map(|x| x.into_tuple())
        }
    }
}

impl Into<DetectorPixel> for DetectorPixelProxy{
    fn into(self) -> DetectorPixel {
        DetectorPixel {
            index: self.index.into(),
            vertices: self.vertices.iter().map(|x| (*x).into()).collect(),
            color: self.color.map(Into::into).into()
        }
    }
}

impl Serialize for DetectorPixel{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        let proxy:DetectorPixelProxy = self.clone().into();
        proxy.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DetectorPixel{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        DetectorPixelProxy::deserialize(deserializer).map(|x| x.into())
    }
}


fn is_ccv(a:Tuple2<f64,f64>,b:Tuple2<f64,f64>,c:Tuple2<f64,f64>)->bool{
    let ab = (b.0-a.0,b.1-a.1);
    let ac = (c.0-a.0,c.1-a.1);
    ab.0*ac.1-ab.1*ac.0 >= 0.0
}


fn convert_array<'a,T:serde::Deserialize<'a>>(arr_in:&'a rhai::Array)->Result<RVec<T>,Box<EvalAltResult>>{
    let mut arr_out:RVec<T> = RVec::with_capacity(arr_in.len());
    for x in arr_in.iter(){
        arr_out.push(from_dynamic(x)?);
    };
    Ok(arr_out)
}

use crate::rotate;

impl DetectorPixel{
    pub fn new(index:RVec<usize>, vertices:RVec<Tuple2<f64,f64>>)->Self{
        Self { index, vertices, color:ROption::RNone}
    }

    pub fn new_rhai(index:rhai::Array, vertices:rhai::Array)->Result<Self, Box<EvalAltResult>>{
        let index = convert_array::<usize>(&index)?;
        let vertices = convert_array::<rhai::Array>(&vertices)?;
        let mut verts:RVec<Tuple2<f64,f64>> = RVec::with_capacity(vertices.len());
        for vert in vertices.iter(){
            let new_vert = convert_array::<f64>(vert)?;
            if new_vert.len()!=2{
                return Err("Pixel vertex must be 2D".into());
            }
            verts.push((new_vert[0],new_vert[1]).into());
        }

        Ok(Self::new(index, verts))
    }

    // fn is_ccv(&self)->bool{
    //     if self.vertices.len()<3{
    //         true
    //     }
    // }

    pub fn triangles(&self)->RVec<[Tuple2<f64,f64>; 3]>{
        let mut res = RVec::new();
        let mut vertices = self.vertices.clone();
        let mut good = true;
        while vertices.len()>=3 && good{
            good = false;
            'inner: for i in 0..vertices.len(){
                let x1:Tuple2<f64,f64> = if i==0{
                    vertices[vertices.len()-1]
                }
                else{
                    vertices[i-1]
                };
                let x2 = vertices[i];
                let x3 = vertices[(i+1)%vertices.len()];
                if is_ccv(x1, x2, x3){
                    vertices.remove(i);
                    res.push([x1,x2,x3]);
                    good = true;
                    break 'inner;
                }
            }
        }
        res
    }

    pub fn set_color(&mut self, r:f32,g:f32,b:f32){
        self.color = RSome((r,g,b).into());
    }

    pub fn clear_color(&mut self){
        self.color = RNone;
    }

    pub fn contains_point(&self,point:(f64,f64))->bool{
        // Let's assume that pixels are convex. Otherwise it will be weird...
        let (x,y) = point;
        if self.vertices.len()<3{
            return false;
        }
        let len = self.vertices.len();
        for i in 0..len{
            let (x1,y1) =  self.vertices[i % len].into_tuple();
            let (x2,y2) =  self.vertices[(i+1) % len].into_tuple();

            let dx2 = x-x1;
            let dy2 = y-y1;

            let dx1 = x2-x1;
            let dy1 = y2-y1;

            if dx1*dy2-dx2*dy1 <= 0.0{
                return false;
            }

        }
        true
    }

    pub fn intersects_or_inside_rectangle(&self, mut left:f64, mut right:f64, mut top:f64, mut bottom:f64)->bool{
        if left>right{
            (left, right) = (right, left);
        }
        if bottom>top{
            (bottom, top) = (top, bottom);
        }

        for (start, end) in self.vertices.iter().zip(self.vertices.iter().skip(1).cycle()){
            if segment_intestects_rect(start.into_tuple(), end.into_tuple(), left, right, top, bottom){
                return true;
            }
        }

        false
    }

    pub fn rectangle(index:Vec<usize>, start:(f64,f64), size:(f64,f64))->Self{
        let (x,y) = start;
        let (w,h) = size;
        let vertices = rvec![
            (x,y).into(),
            (x+w,y).into(),
            (x+w,y+h).into(),
            (x,y+h).into()
        ];
        Self::new(index.into(), vertices)
    }

    pub fn rectangle_centered(index:Vec<usize>, center:(f64,f64), size:(f64,f64))->Self{
        let start_x = center.0-size.0/2.;
        let start_y = center.1-size.1/2.;
        Self::rectangle(index, (start_x,start_y), size)
    }

    pub fn rectangle_centered_rhai(index:rhai::Array, center_x:f64, center_y:f64, size_x:f64, size_y:f64)->Result<Self, Box<EvalAltResult>>{
        Self::rectangle_rhai(index, center_x-size_x/2., center_y-size_y/2.,size_x, size_y)
    }

    pub fn square(index:Vec<usize>, center:(f64,f64), size:f64)->Self{
        Self::rectangle_centered(index, center, (size,size))
    }

    pub fn square_rhai(index:rhai::Array, center_x:f64, center_y:f64, size:f64)->Result<Self, Box<EvalAltResult>>{
        Self::rectangle_centered_rhai(index, center_x, center_y, size, size)
    }


    pub fn rectangle_rhai(index:rhai::Array, start_x:f64, start_y:f64, size_x:f64, size_y:f64)->Result<Self, Box<EvalAltResult>>{
        let index = convert_array(&index)?;
        Ok(Self::rectangle(index.into(), (start_x, start_y), (size_x,size_y)))
    }

    fn build_extra(builder: &mut TypeBuilder<Self>) {
        builder
            .with_name("DetectorPixel")
            .with_fn("new_pixel", Self::new_rhai)
            .with_fn("rectangle", Self::rectangle_rhai)
            .with_fn("rectangle_centered", Self::rectangle_centered_rhai)
            .with_fn("square", Self::square_rhai)
            .with_fn("set_color", Self::set_color)
            .with_fn("clear_color", Self::clear_color);
    }

    pub fn make_polygon<S:Into<ShapeStyle>>(&self,color:S, rotation:f64) -> Polygon<(f64,f64)>{
        Polygon::new(self.vertices.iter().map(|x| x.into_tuple()).map(|x| rotate(x,rotation)).collect::<Vec<(f64,f64)>>(), color)
    }

    pub fn make_outline(&self, rotation:f64)->PathElement<(f64,f64)>{
        let mut verts = self.vertices.iter().map(|x| x.into_tuple()).map(|x| rotate(x,rotation)).collect::<Vec<(f64,f64)>>();
        if let Some(x) = verts.get(0){
            verts.push(*x); // To make a closed loop
        }
        PathElement::new(verts, BLACK)
    }

    pub fn into_src(&self, round:Option<usize>)->String{

        let vertices = self.vertices.iter().fold("".to_string(), |a,b| format!("{} {}",a,format_point(b.into_tuple(), round)));
        format!("pixel {:?}\n    polygon {}",self.index, vertices)
    }

    /// Get boundaries min(x,y), max(x,y)
    pub fn boundaries(&self)->((f64,f64),(f64,f64)){
        let mut verts = self.vertices.iter();
        let first = verts.next().unwrap();
        let (mut min_x, mut min_y) = (*first).into_tuple();
        let (mut max_x, mut max_y) = (*first).into_tuple();
        for vert in verts{
            let (x,y) = (*vert).into_tuple();
            if x>max_x{
                max_x = x;
            }

            if x<min_x{
                min_x = x;
            }

            if y>max_y{
                max_y = y;
            }

            if y<min_y{
                min_y = y;
            }
        }
        // /println!("{:?}", ((min_x, min_y), (max_x, max_y)));
        ((min_x, min_y), (max_x, max_y))
    }

    pub fn get_color(&self)->(f32,f32,f32){
        if let RSome(c) = self.color{
            c.into_tuple()
        }
        else{
            super::colors::get_color_indexed(self.index.as_ref())
        }
    }
}

#[derive(Clone,Debug,Serialize,Deserialize,PartialEq, CustomType, StableAbi)]
#[rhai_type(extra = Self::build_extra)]
#[repr(C)]
pub struct Detector{
    pub compat_shape:RVec<usize>,
    pub content:RVec<DetectorPixel>,
    pub name:RString,
}

impl Detector{
    pub fn new(compat_shape:RVec<usize>,name:RString)->Self{
        let capacity = compat_shape.iter().fold(1, |a,b| a*b);
        Self { compat_shape, content: RVec::with_capacity(capacity), name}
    }

    pub fn set_name(&mut self, name:&str){
        self.name = name.into();
    }

    pub fn set_shape(&mut self, shape:rhai::Array) -> Result<(), Box<EvalAltResult>> {
        self.compat_shape = convert_array::<usize>(&shape)?;
        Ok(())
    }

    pub fn clear(&mut self){
        self.content.clear();
    }

    pub fn add_pixel(&mut self, pixel:DetectorPixel){
        self.content.push(pixel);
    }

    pub fn shape(&self)->&[usize]{
        &self.compat_shape
    }

    pub fn size(&self)->((f64,f64),(f64,f64)){
        if self.content.is_empty(){
            return ((0.0,0.0),(0.0,0.0));
        }
        let mut cells = self.content.iter();
        let ((mut min_x, mut min_y), (mut  max_x, mut max_y)) = cells.next().unwrap().boundaries();
        for pix in cells{
            let ((p_min_x, p_min_y), (p_max_x, p_max_y)) = pix.boundaries();
            if p_min_x<min_x{
                min_x = p_min_x;
            }
            if p_min_y<min_y{
                min_y = p_min_y;
            }

            if p_max_x>max_x{
                max_x = p_max_x;
            }
            if p_max_y>max_y{
                max_y = p_max_y;
            }
        }
        ((min_x, min_y), (max_x, max_y))
    }


    pub fn position_index(&self, point:(f64,f64))->Option<&[usize]>{
        for pix in self.content.iter(){
            if pix.contains_point(point){
                return Some(&pix.index);
            }
        }
        None
    }

    pub fn select_indices_in_rectangle(&self, left:f64, right:f64, top:f64, bottom:f64) -> Vec<Vec<usize>>{
        let mut res = Vec::new();
        for pix in self.content.iter(){
            if pix.intersects_or_inside_rectangle(left,right,top,bottom){
                res.push(pix.index.to_vec());
            }
        }
        res
    }

    pub fn default_vtl()->Self{

        let mut vtl = Self::new(rvec![16,16], "Verkhnetulomsky".into());
        let pixel_size = 2.85;
        let half_gap = 2.0;
        for i in 0..16usize{
            for j in 0..16usize{
                let x_offset = if i<8 {-half_gap} else {half_gap} ;
                let y_offset = if j<8 {-half_gap} else {half_gap};
                let x = x_offset+(i as f64)*pixel_size-8.0*pixel_size;
                let y = y_offset+(j as f64)*pixel_size-8.0*pixel_size;
                let index = vec![i,j];
                vtl.content.push(DetectorPixel::rectangle(index, (x,y), (pixel_size,pixel_size)));
            }
        }
        vtl
    }

    pub fn into_src(&self,round:Option<usize>)->String{
        let header = format!("name \"{}\"\nshape {:?}\n",self.name,self.compat_shape);
        let lines = self.content.iter().fold("".to_string(), |a,b| format!("{}\n{}", a, b.into_src(round)));
        format!("{}\n{}",header ,lines)
    }



    pub fn from_specs<'a>(i:&'a str)->Result<Self, nom::Err<nom::error::Error<&'a str>>>{
        parse_detector(i).map(|x| x.1)
    }

    pub fn from_script<'a>(i:&'a str)->Result<Self, Box<rhai::EvalAltResult>>{
        parse_scripted(i)
    }

    pub fn find_color(&self, index:&Vec<usize>)->Option<(f32,f32,f32)>{
        for pixel in &self.content{
            if do_vecs_match(&pixel.index, index){
                return Some(pixel.get_color());
            }
        }
        None
    }

    fn build_extra(builder: &mut TypeBuilder<Self>) {
        builder
            .with_name("DetectorContent")
            .with_fn("new_detector", Self::default)
            .with_fn("clear", Self::clear)
            .with_fn("set_name", Self::set_name)
            .with_fn("set_shape", Self::set_shape)
            .with_fn("add_pixel", Self::add_pixel);
    }
}


impl Default for Detector{
    fn default() -> Self {
        Self::new(rvec![1], "untitled".into())
    }
}



//pub type StableColorMatrix = ArrayND<abi_stable::std_types::Tuple3<u8,u8,u8>>;


