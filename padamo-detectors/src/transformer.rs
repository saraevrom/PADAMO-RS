use std::ops::Range;

#[derive(Clone,Copy,Debug)]
pub struct Transform{
    pub zoom:f64,
    pub delta_x:f64,
    pub delta_y:f64
}

impl Transform{
    pub fn new(zoom: f64, delta_x: f64, delta_y: f64) -> Self {
        Self { zoom, delta_x, delta_y }
    }

    pub fn transform_x_range(&self,range_x:Range<f64>)->Range<f64>{
        (self.delta_x+range_x.start*self.zoom)..(self.delta_x+range_x.end*self.zoom)
    }
    pub fn transform_y_range(&self,range_y:Range<f64>)->Range<f64>{
        (self.delta_y+range_y.start*self.zoom)..(self.delta_y+range_y.end*self.zoom)
    }
}

impl Default for Transform{
    fn default() -> Self {
        Self::new(1.0, 0.0, 0.0)
    }
}
