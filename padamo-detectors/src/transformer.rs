
#[derive(Clone,Copy,Debug)]
pub struct Transform{
    pub zoom:f64,
    pub delta_x:f64,
    pub delta_y:f64
}

impl Transform{
    pub fn transform_x_range(&self,range_x:(f64,f64))->(f64,f64){
        (self.delta_x+range_x.0*self.zoom,self.delta_x+range_x.1*self.zoom)
    }
    pub fn transform_y_range(&self,range_y:(f64,f64))->(f64,f64){
        (self.delta_y+range_y.0*self.zoom,self.delta_y+range_y.1*self.zoom)
    }
}
