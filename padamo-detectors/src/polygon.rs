
use padamo_api::lazy_array_operations::ndim_array::ArrayND;
use serde::{Serialize,Deserialize};
use plotters::prelude::*;
use crate::parser::parse_detector;
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


#[derive(Serialize,Deserialize, Debug,Clone, PartialEq)]
pub struct DetectorPixel{
    pub index:Vec<usize>,
    pub vertices:Vec<(f64,f64)>,
}

fn is_ccv(a:(f64,f64),b:(f64,f64),c:(f64,f64))->bool{
    let ab = (b.0-a.0,b.1-a.1);
    let ac = (c.0-a.0,c.1-a.1);
    ab.0*ac.1-ab.1*ac.0 >= 0.0
}

impl DetectorPixel{
    pub fn new(index:Vec<usize>, vertices:Vec<(f64,f64)>)->Self{
        Self { index, vertices}
    }

    // fn is_ccv(&self)->bool{
    //     if self.vertices.len()<3{
    //         true
    //     }
    // }

    pub fn triangles(&self)->Vec<[(f64,f64); 3]>{
        let mut res = Vec::new();
        let mut vertices = self.vertices.clone();
        let mut good = true;
        while vertices.len()>=3 && good{
            good = false;
            'inner: for i in 0..vertices.len(){
                let x1:(f64,f64) = if i==0{
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

    pub fn contains_point(&self,point:(f64,f64))->bool{
        // Let's assume that pixels are convex. Otherwise it will be weird...
        let (x,y) = point;
        if self.vertices.len()<3{
            return false;
        }
        let len = self.vertices.len();
        for i in 0..len{
            let (x1,y1) =  self.vertices[i % len];
            let (x2,y2) =  self.vertices[(i+1) % len];

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

    pub fn rectangle(index:Vec<usize>, start:(f64,f64), size:(f64,f64))->Self{
        let (x,y) = start;
        let (w,h) = size;
        let vertices = vec![
            (x,y),
            (x+w,y),
            (x+w,y+h),
            (x,y+h)
        ];
        Self::new(index, vertices)
    }

    pub fn make_polygon<S:Into<ShapeStyle>>(&self,color:S) -> Polygon<(f64,f64)>{
        Polygon::new(self.vertices.clone(), color)
    }

    pub fn make_outline(&self)->PathElement<(f64, f64)>{
        PathElement::new(self.vertices.clone(), BLACK)
    }

    pub fn into_src(&self, round:Option<usize>)->String{

        let vertices = self.vertices.iter().fold("".to_string(), |a,b| format!("{} {}",a,format_point(*b, round)));
        format!("pixel {:?}\n    polygon {}",self.index, vertices)
    }

    /// Get boundaries min(x,y), max(x,y)
    pub fn boundaries(&self)->((f64,f64),(f64,f64)){
        let mut verts = self.vertices.iter();
        let first = verts.next().unwrap();
        let (mut min_x, mut min_y) = *first;
        let (mut max_x, mut max_y) = *first;
        for vert in verts{
            let (x,y) = *vert;
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
}

#[derive(Clone,Debug,Serialize,Deserialize,PartialEq)]
pub struct DetectorContent{
    pub compat_shape:Vec<usize>,
    pub content:Vec<DetectorPixel>,
    pub name:String,
}

impl DetectorContent{
    pub fn new(compat_shape:Vec<usize>,name:String)->Self{
        let capacity = compat_shape.iter().fold(1, |a,b| a*b);
        Self { compat_shape, content: Vec::with_capacity(capacity), name}
    }


    pub fn shape(&self)->&Vec<usize>{
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


    pub fn position_index(&self, point:(f64,f64))->Option<&Vec<usize>>{
        for pix in self.content.iter(){
            if pix.contains_point(point){
                return Some(&pix.index);
            }
        }
        None
    }

    pub fn default_vtl()->Self{

        let mut vtl = Self::new(vec![16,16], "Verkhnetulomsky".into());
        let PIXEL_SIZE = 2.85;
        let HALF_GAP = 2.0;
        for i in 0..16usize{
            for j in 0..16usize{
                let x_offset = if i<8 {-HALF_GAP} else {HALF_GAP} ;
                let y_offset = if j<8 {-HALF_GAP} else {HALF_GAP};
                let x = x_offset+(i as f64)*PIXEL_SIZE-8.0*PIXEL_SIZE;
                let y = y_offset+(j as f64)*PIXEL_SIZE-8.0*PIXEL_SIZE;
                let index = vec![i,j];
                vtl.content.push(DetectorPixel::rectangle(index, (x,y), (PIXEL_SIZE,PIXEL_SIZE)));
            }
        }
        vtl
    }

    pub fn into_src(&self,round:Option<usize>)->String{
        let header = format!("name \"{}\"\nshape {:?}\n",self.name,self.compat_shape);
        let lines = self.content.iter().fold("".to_string(), |a,b| format!("{}\n{}", a, b.into_src(round)));
        format!("{}\n{}",header ,lines)
    }

    pub fn pixels_values<'a>(&'a self, alive_pixels:&'a ArrayND<bool>, pixels:&'a Option<(&'a ArrayND<f64>,f64)>, scale:super::Scaling)->RectIterator<'a>{
        RectIterator::new(self, alive_pixels, pixels, scale)
    }

    pub fn pixels_colors<'a>(&'a self, pixels:&'a [Vec<usize>], vis:&'a [bool])->ColorIterator<'a>{
        ColorIterator::new(self, pixels, vis)
    }

    pub fn from_specs<'a>(i:&'a str)->Result<Self, nom::Err<nom::error::Error<&'a str>>>{
        parse_detector(i).map(|x| x.1)
    }
}


/// Iterator for signal display value
pub struct RectIterator<'a>{
    pub detector:&'a DetectorContent,
    pub alive_pixels:&'a ArrayND<bool>,
    current_index:usize,
    source:&'a Option<(&'a ArrayND<f64>,f64)>,
    scale:super::Scaling,
}

impl<'a> RectIterator<'a>{
    pub fn new(detector:&'a DetectorContent, alive_pixels: &'a ArrayND<bool>,source:&'a Option<(&'a ArrayND<f64>,f64)>, scale:super::Scaling)->Self{
        Self{detector, current_index:0, source, scale, alive_pixels}
    }


    fn get_current_result(&self)->Polygon<(f64,f64)>{
        // let coords = self.detector.remap_coords((self.i,self.j));
        // let coords = (coords.0-self.size.0/2.0,coords.1-self.size.1/2.0);
        let poly = &self.detector.content[self.current_index];

        let alive = self.alive_pixels.try_get(&poly.index).map(|x| *x).unwrap_or(true);

        let color = if !alive{
            plotters::style::colors::BLACK.filled()
        }
        else if let Some((arr,_)) = self.source{
            let (min_,max_) = self.scale.get_bounds(arr,self.alive_pixels);
            //println!("COORDS {},{}",self.i,self.j);
            if let Some(v) = arr.try_get(&poly.index){
                plotters::style::colors::colormaps::ViridisRGB::get_color_normalized(*v,min_,max_).filled()
            }
            else{
                plotters::style::colors::RED.filled()
            }
        }
        else{
            plotters::style::colors::BLUE.filled()
        };

        poly.make_polygon(color)
    }
}

impl<'a> Iterator for RectIterator<'a>{
    type Item = Polygon<(f64,f64)>;

    fn next(&mut self) -> Option<Polygon<(f64,f64)>> {
        if self.current_index<self.detector.content.len(){
            let res = self.get_current_result();
            self.current_index += 1;
            Some(res)
        }
        else{
            None
        }
    }
}

//pub type StableColorMatrix = ArrayND<abi_stable::std_types::Tuple3<u8,u8,u8>>;

/// Iterator for colored pixelmaps
pub struct ColorIterator<'a>{
    pub detector:&'a DetectorContent,
    current_index:usize,
    source:&'a [Vec<usize>],
    vis:&'a [bool],
}

fn search_vec(haystack:&[Vec<usize>], needle:&Vec<usize>)->Option<usize>{
    for (i,x) in haystack.iter().enumerate(){
        if x==needle{
            return Some(i);
        }
    }
    None
}

impl<'a> ColorIterator<'a>{
    pub fn new(detector:&'a DetectorContent,source:&'a [Vec<usize>], vis:&'a [bool])->Self{
        Self{detector, current_index:0, source, vis}
    }


    fn get_current_result(&self)->(Polygon<(f64,f64)>,PathElement<(f64,f64)>){
        // let coords = self.detector.remap_coords((self.i,self.j));
        // let coords = (coords.0-self.size.0/2.0,coords.1-self.size.1/2.0);
        let poly = &self.detector.content[self.current_index];

        let color = if let Some(i) = search_vec(self.source, &poly.index) {
            if self.vis[i]{
                let rgb = super::colors::get_color_indexed(&poly.index);
                let r = (rgb.0*256.0) as u8;
                let g = (rgb.1*256.0) as u8;
                let b = (rgb.2*256.0) as u8;
                RGBColor(r,g,b).filled()
            }
            else{
                plotters::style::colors::WHITE.filled()
            }
        }
        else{
            plotters::style::colors::WHITE.filled()
        };

        (poly.make_polygon(color),poly.make_outline())
    }
}


impl<'a> Iterator for ColorIterator<'a>{
    type Item = (Polygon<(f64,f64)>,PathElement<(f64,f64)>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index<self.detector.content.len(){
            let res = self.get_current_result();
            self.current_index += 1;
            Some(res)
        }
        else{
            None
        }
    }
}
