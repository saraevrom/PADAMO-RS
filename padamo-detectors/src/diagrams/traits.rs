use plotters::prelude::*;

pub trait ColorSource{
    fn get_color(&self, pixel:&[usize]) -> ShapeStyle;
}

impl<T:Fn(&[usize]) -> RGBColor> ColorSource for T {
    fn get_color(&self, pixel:&[usize]) -> ShapeStyle {
        (self)(pixel).filled()
    }
}

impl ColorSource for RGBColor{
    fn get_color(&self, _pixel:&[usize]) -> ShapeStyle {
        self.filled()
    }
}

impl ColorSource for ShapeStyle{
    fn get_color(&self, _pixel:&[usize]) -> ShapeStyle {
        *self
    }
}
