use padamo_arraynd::ArrayND;


pub mod polygon;
pub mod parser;
pub mod scripted;
pub mod colors;
// pub mod selector_chart;
// pub mod selector_chart_simple;
pub mod transformer;
pub mod loaded_detectors_storage;
pub mod mesh;
pub mod diagrams;
// pub mod transform_widget;

pub use transformer::Transform;
// pub use transform_widget::TransformState;
// pub use selector_chart::DetectorChartMap;

use crate::mesh::Mesh;

//use polygon::StableColorMatrix;

//TODO: Remove when reengineering is done.
pub use diagrams::scaling::Scaling;


// const COLORBAR_SEGMENTS:usize = 256;

// const COLORBAR_WIDTH:f64 = 1.0;

#[derive(Clone, Copy)]
pub struct Margins{
    pub top:u32,
    pub bottom:u32,
    pub left:u32,
    pub right:u32
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[derive(abi_stable::StableAbi)]
#[repr(C)]
pub struct DetectorAndMask{
    pub cells:polygon::Detector,
    pub alive_pixels:ArrayND<bool>,
}

impl DetectorAndMask {
    pub fn new(cells: polygon::Detector, alive_pixels: ArrayND<bool>) -> Self {
        Self { cells, alive_pixels }
    }

    pub fn from_cells(cells:polygon::Detector)->Self{
        let alive_pixels = ArrayND::new(cells.compat_shape.to_vec(), true) ;
        DetectorAndMask::new(cells, alive_pixels)
    }

    pub fn alive_pixels_mask(&self)->ArrayND<f64>{
        let shape = self.alive_pixels.shape.clone();
        let flat_data = self.alive_pixels.flat_data.iter().map(|x| if *x {1.0} else {0.0}).collect::<Vec<f64>>().into();
        ArrayND { flat_data, shape }
    }

    pub fn toggle_pixel(&mut self, index:&Vec<usize>){
        if let Some(v) = self.alive_pixels.try_get(index){
            self.alive_pixels[index] = !v;
        }
    }

    pub fn reset_mask(&mut self){
        self.alive_pixels.flat_data.iter_mut().for_each(|x| *x = true);
    }

    pub fn shape(&self)->&[usize]{
        &self.cells.compat_shape
    }

    pub fn default_vtl()->Self{
        Self::from_cells(polygon::Detector::default_vtl())
    }
}
