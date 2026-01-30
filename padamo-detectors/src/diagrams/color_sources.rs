use padamo_arraynd::ArrayND;
use crate::polygon::DetectorContent;

use super::traits::ColorSource;
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
}



impl<'a> ColorSource for MatrixSource<'a>
{
    fn get_color(&self, pixel:&[usize]) -> ShapeStyle {
        if self.mask.try_get(pixel).map(|x| *x).unwrap_or(false){
            let (min_, mut max_) = self.scale.get_bounds(self.data,self.mask);
            max_ = if max_>min_ {max_} else {min_+0.1};
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
}

pub struct ColoredMaskSource<'a>
{
    mask:&'a ArrayND<bool>,
}

impl<'a> ColorSource for ColoredMaskSource<'a>{
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
}
