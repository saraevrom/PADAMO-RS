use plotters::prelude::*;

pub trait ColorValueSource{
    fn get_color(&self, pixel:&[usize]) -> ShapeStyle;
    fn get_value(&self, _pixel:&[usize]) -> Option<f64>{None}
    fn get_norm(&self) -> Option<(f64, f64)>{None}
    fn has_outline(&self) -> bool {false}
}

impl<T:Fn(&[usize]) -> RGBColor> ColorValueSource for T {
    fn get_color(&self, pixel:&[usize]) -> ShapeStyle {
        (self)(pixel).filled()
    }
}

impl ColorValueSource for RGBColor{
    fn get_color(&self, _pixel:&[usize]) -> ShapeStyle {
        self.filled()
    }
}

impl ColorValueSource for ShapeStyle{
    fn get_color(&self, _pixel:&[usize]) -> ShapeStyle {
        *self
    }
}


