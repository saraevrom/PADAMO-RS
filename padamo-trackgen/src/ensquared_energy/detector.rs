use padamo_detectors::polygon::{DetectorContent,DetectorPixel};


#[derive(Clone,Debug)]
pub struct DetectorPixelWireframe{
    pub triangles:Vec<[(f64,f64); 3]>,
    pub index:Vec<usize>
}

#[derive(Clone,Debug)]
pub struct DetectorWireframe{
    pub triangles:Vec<DetectorPixelWireframe>,
    pub shape:Vec<usize>
}

impl DetectorPixelWireframe {
    pub fn new(triangles: Vec<[(f64,f64); 3]>, index: Vec<usize>) -> Self { Self { triangles, index } }
}



impl Into<DetectorPixelWireframe> for &DetectorPixel{
    fn into(self) -> DetectorPixelWireframe {
        let tris = self.triangles();
        DetectorPixelWireframe::new(tris, self.index.clone())
    }
}

pub fn wireframe(detector:DetectorContent)->DetectorWireframe{
    let triangles = detector.content.iter().map(|x| x.into()).collect();
    let shape = detector.compat_shape;
    let res = DetectorWireframe { triangles, shape };
    //println!("{:?}",res);
    res
}
