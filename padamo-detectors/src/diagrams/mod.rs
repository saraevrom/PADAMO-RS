use crate::polygon::DetectorContent;

pub mod traits;
pub mod color_sources;
pub mod scaling;
pub mod modular_diagram;
pub mod auxiliary;
pub mod colorbar;

use plotters::prelude::Polygon;

pub use modular_diagram::PadamoDetectorDiagram;
pub use color_sources::{ColoredMaskSource, MatrixSource, DualColoredMaskSource, autoselect_source};

pub struct PolyIterator<'a>
{
    source:&'a dyn traits::ColorValueSource,
    current_index:usize,
    detector:&'a DetectorContent
}

impl<'a> PolyIterator<'a>
{
    pub fn new(source: &'a dyn traits::ColorValueSource, detector: &'a DetectorContent) -> Self {
        Self { source, current_index:0, detector }
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
            let res = pixel.make_polygon(color);
            self.current_index += 1;
            Some(res)
        }
        else{
            None
        }
    }
}


