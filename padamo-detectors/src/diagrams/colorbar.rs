use plotters::prelude::*;

pub const COLORBAR_SEGMENTS:usize = 256;

pub const COLORBAR_WIDTH:f64 = 1.0;


pub struct ColorbarRects<'a>{
    pub min:f64,
    pub max:f64,
    pub colormap:&'a dyn ColorMap<RGBColor,f64>,
    index:usize,
}

impl<'a> ColorbarRects<'a>{
    pub fn new(min:f64, max:f64, colormap:&'a dyn ColorMap<RGBColor,f64>)->Self{
        let max = if max>min {max} else {min+0.1};
        Self { min, max, colormap, index: 0 }
    }
}

impl<'a> Iterator for ColorbarRects<'a>{
    type Item = Rectangle<(f64,f64)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index>=COLORBAR_SEGMENTS{
            return None;
        }

        let step = (self.max - self.min)/(COLORBAR_SEGMENTS as f64);
        let start_y = step * (self.index as f64)+self.min;
        let end_y = start_y+step;
        let mid_y = start_y+step*0.5;

        let s_coords = (0.0,start_y);
        let e_coords = (COLORBAR_WIDTH,end_y);


        let color = self.colormap.get_color_normalized(mid_y,self.min, self.max).filled();

        self.index += 1;
        Some(Rectangle::new(
            [s_coords, e_coords],
            color
        ))
    }

}
