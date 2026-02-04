use crate::polygon::Detector;

pub mod traits;
pub mod color_sources;
pub mod scaling;
pub mod modular_diagram;
pub mod auxiliary;
pub mod colorbar;

use plotters::prelude::{PathElement, Polygon};

pub use modular_diagram::PadamoDetectorDiagram;
pub use color_sources::{ColoredMaskSource, MatrixSource, DualColoredMaskSource, autoselect_source};
pub use color_sources::{ContourMask, Contoured};

pub struct PolyIterator<'a>
{
    source:&'a dyn traits::ColorValueSource,
    current_index:usize,
    detector:&'a Detector,
    rotation:f64,
}

impl<'a> PolyIterator<'a>
{
    pub fn new(source: &'a dyn traits::ColorValueSource, detector: &'a Detector, rotation:f64) -> Self {
        Self { source, current_index:0, detector, rotation }
    }
}

impl<'a> Iterator for PolyIterator<'a>
{
    type Item = Polygon<(f64,f64)>;

    fn next(&mut self) -> Option<Polygon<(f64,f64)>> {
        if self.current_index<self.detector.content.len(){
            // let res = self.get_current_result();
            // self.current_index += 1;
            // Some(res)
            let pixel = self.detector.content.get(self.current_index)?;
            let color = self.source.get_color(&pixel.index);
            let res = pixel.make_polygon(color,self.rotation);
            self.current_index += 1;
            Some(res)
        }
        else{
            None
        }
    }
}


pub struct PixelPathIterator<'a>{
    pub detector:&'a Detector,
    current_index:usize,
    color_source:&'a dyn traits::ColorValueSource,
    rotation:f64
}

impl<'a> PixelPathIterator<'a> {
    pub fn new(detector: &'a Detector, color_source:&'a dyn traits::ColorValueSource, rotation:f64) -> Self {
        Self { detector, current_index:0, color_source, rotation }
    }
}

impl<'a> Iterator for PixelPathIterator<'a>{
    type Item = PathElement<(f64,f64)>;

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.detector.content.len();
        //skipping unactives
        while self.current_index<len && !self.color_source.has_outline(&self.detector.content[self.current_index].index){
            self.current_index += 1;
        }
        if self.current_index<len{
            let pixel = &self.detector.content[self.current_index];
            let res = pixel.make_outline(self.rotation);
            self.current_index += 1;
            Some(res)
        }
        else{
            None
        }
    }
}

