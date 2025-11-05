use padamo_detectors::{loaded_detectors_storage::ProvidedDetectorInfo, mesh::{Marker, Mesh}};
use padamo_iced_forms::{IcedForm, IcedFormBuffer};

#[derive(Clone,Debug, IcedForm)]
pub struct HeightProbeTestObject{
    #[field_name("Height")] pub height:f64,
    #[field_name("Detector X")] pub detector_x:f64,
    #[field_name("Detector Y")] pub detector_y:f64,
    #[field_name("Marker size")] pub marker_size:f64,

}

impl Default for HeightProbeTestObject{
    fn default() -> Self {
        Self {
            height: 100.0,
            detector_x: 0.0,
            detector_y: 0.0,
            marker_size: 1.0,
        }
    }
}

impl HeightProbeTestObject{
    pub fn generate_mesh(&self, detector_info:Option<&ProvidedDetectorInfo>)->Option<Mesh>{
        let detector = detector_info?;
        if detector.focal_distance<=0.0{
            return None;
        }
        let mut mesh = Mesh::new();
        mesh.add_vertex(nalgebra::Vector3::new(0.0,0.0,0.0), self.marker_size, Marker::Circle);
        let pos = nalgebra::Vector3::new(self.detector_x/detector.focal_distance, self.detector_y/detector.focal_distance, 1.0)*self.height;
        mesh.add_vertex(pos, self.marker_size, Marker::Cross);
        mesh.regiester_path(vec![0,1]);
        Some(mesh)
    }
}
