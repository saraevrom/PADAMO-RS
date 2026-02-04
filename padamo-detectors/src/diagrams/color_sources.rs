use padamo_arraynd::ArrayND;

use crate::{DetectorAndMask};

use super::traits::ColorValueSource;
use plotters::prelude::*;
use super::scaling::Scaling;

pub struct MatrixSource<'a>
{
    data:&'a ArrayND<f64>,
    mask:&'a ArrayND<bool>,
    scale:Scaling,
    colormap:&'a dyn ColorMap<RGBColor, f64>,
    bad_color:RGBColor,
    nodata_color:RGBColor,
    masked_color:RGBColor,
}

impl<'a> MatrixSource<'a>
{
    pub fn new(data: &'a ArrayND<f64>, mask: &'a ArrayND<bool>) -> Self {
        Self { data, mask,
            scale: Scaling::Autoscale,
            colormap: &ViridisRGB,
            bad_color: RED,
            nodata_color: RED,
            masked_color: BLACK
        }
    }

    pub fn with_scaling(mut self, scale:Scaling) -> Self{
        self.scale = scale;
        self
    }

    pub fn with_colormap<T:ColorMap<RGBColor, f64>>(mut self, colormap:&'a T) -> Self{
        self.colormap = colormap;
        self
    }

    pub fn with_bad_color(mut self, color:RGBColor) -> Self{
        self.bad_color = color;
        self
    }

    pub fn with_nodata_color(mut self, color:RGBColor) -> Self{
        self.nodata_color = color;
        self
    }

    pub fn with_masked_color(mut self, color:RGBColor) -> Self{
        self.masked_color = color;
        self
    }

    fn get_norm(&self) -> (f64, f64) {
        let (min_, mut max_) = self.scale.get_bounds(self.data,self.mask);
        max_ = if max_>min_ {max_} else {min_+0.1};
        (min_, max_)
    }
}



impl<'a> ColorValueSource for MatrixSource<'a>
{
    fn get_color(&self, pixel:&[usize]) -> ShapeStyle {
        if self.mask.try_get(pixel).map(|x| *x).unwrap_or(false){
            let (min_, max_) = self.get_norm();
            //println!("COORDS {},{}",self.i,self.j);
            if let Some(v) = self.data.try_get(&pixel){
                if !(v.is_nan() || min_.is_nan() || max_.is_nan()){
                    //println!("GET CMAP {}, [{}, {}]",*v,min_,max_);
                    self.colormap.get_color_normalized(*v,min_,max_).filled()
                }
                else{
                    self.bad_color.filled()
                }
            }
            else{
                self.nodata_color.filled()
            }
        }
        else{
            self.masked_color.filled()
        }
    }

    fn get_bar<'b>(&'b self) -> Option<((f64, f64),&'b dyn ColorMap<RGBColor, f64>)> {
        let edges = self.get_norm();
        Some((edges, self.colormap))
    }

    fn get_value(&self, pixel:&[usize]) -> Option<f64> {
        Some(self.data.try_get(pixel).map(|x|*x).unwrap_or(f64::NAN))
    }
}

pub struct ColoredMaskSource<'a>
{
    mask:&'a ArrayND<bool>,
}

impl<'a> ColoredMaskSource<'a> {
    pub fn new(mask: &'a ArrayND<bool>) -> Self {
        Self { mask }
    }
}

impl<'a> ColorValueSource for ColoredMaskSource<'a>{
    fn get_color(&self, pixel:&[usize]) -> ShapeStyle {
        if self.mask.try_get(pixel).map(|x| *x).unwrap_or(false){
            let (r,g,b) = crate::colors::get_color_indexed(pixel);
            let r = (r*256.0) as u8;
            let g = (g*256.0) as u8;
            let b = (b*256.0) as u8;
            RGBColor(r,g,b).filled()
        }
        else{
            WHITE.filled()
        }
    }

    fn has_outline(&self) -> bool {
        true
    }
}

pub struct DualColoredMaskSource<'a>
{
    mask:&'a ArrayND<bool>,
    active_color: RGBColor,
    inactive_color: RGBColor
}

impl<'a> DualColoredMaskSource<'a> {
    pub fn new(mask: &'a ArrayND<bool>, active_color: RGBColor, inactive_color: RGBColor) -> Self {
        Self { mask, active_color, inactive_color }
    }
}

impl<'a> ColorValueSource for DualColoredMaskSource<'a>{
    fn get_color(&self, pixel:&[usize]) -> ShapeStyle {
        if self.mask.try_get(pixel).map(|x| *x).unwrap_or(false){
            self.active_color.filled()
        }
        else{
            self.inactive_color.filled()
        }
    }

}

pub fn autoselect_source<'a>(detector:Option<&'a DetectorAndMask>, buffer:Option<&'a ArrayND<f64>>)->Box<dyn ColorValueSource+'a>{
    if let Some(det) = detector{
        if let Some(buf) = buffer{
            Box::new(MatrixSource::new(&buf, &det.alive_pixels))
        }
        else{
            Box::new(DualColoredMaskSource::new(&det.alive_pixels, plotters::prelude::BLUE, plotters::prelude::BLACK))
        }
    }
    else{
        Box::new(plotters::prelude::BLUE)
    }
}

pub struct Contoured<T:ColorValueSource>{
    pub source:T
}

impl<T: ColorValueSource> Contoured<T> {
    pub fn new(source: T) -> Self {
        Self { source }
    }
}

impl<T:ColorValueSource> ColorValueSource for Contoured<T>{
    fn get_color(&self, pixel:&[usize]) -> ShapeStyle {
        self.source.get_color(pixel)
    }

    fn get_value(&self, pixel:&[usize]) -> Option<f64> {
        self.source.get_value(pixel)
    }

    fn get_bar<'a>(&'a self) -> Option<((f64, f64),&'a dyn ColorMap<RGBColor, f64>)> {
        self.source.get_bar()
    }

    fn has_outline(&self)->bool{
        true
    }
}

pub trait Contourable: ColorValueSource+Sized{
    fn contoured(self)->Contoured<Self>{
        Contoured::new(self)
    }
}

impl<T:ColorValueSource> Contourable for T{

}
