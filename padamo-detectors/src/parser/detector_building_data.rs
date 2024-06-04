use crate::polygon::DetectorPixel;

pub trait Transformable{
    fn moved(self,offset:(f64,f64))->Self;
    fn rotated(self,angle:f64)->Self;
}

pub trait PixelMaker{
    fn get_pixels(&self)->Vec<DetectorPixel>;
}

#[derive(Clone,Debug)]
pub struct PolygonArray(pub Vec<(f64,f64)>);

impl Transformable for PolygonArray{
    fn moved(mut self,offset:(f64,f64))->Self {
        for pos in self.0.iter_mut(){
            pos.0 += offset.0;
            pos.1 += offset.1;
        }
        self
    }

    fn rotated(mut self, angle:f64)->Self{
        let a11 = angle.cos();
        let a12 = -angle.sin();
        let a21 = angle.sin();
        let a22 = angle.cos();
        for pos in self.0.iter_mut(){
            let x = pos.0;
            let y = pos.1;
            *pos = (x*a11+y*a12,x*a21+y*a22);
        }
        self
    }
}

impl From<Vec<(f64,f64)>> for PolygonArray{
    fn from(value: Vec<(f64,f64)>) -> Self {
        Self(value)
    }
}

#[derive(Clone,Debug)]
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
    fn moved(mut self,offset:(f64,f64))->Self {
        self.polygon = self.polygon.moved(offset);
        self
    }

    fn rotated(mut self, angle:f64)->Self{
        self.polygon = self.polygon.rotated(angle);
        self
    }
}
