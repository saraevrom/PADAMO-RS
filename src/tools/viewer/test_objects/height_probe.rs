use padamo_iced_forms::{IcedForm, IcedFormBuffer};

#[derive(Clone,Debug, IcedForm)]
pub struct HeightProbeTestObject{
    #[field_name("Height")] pub height:f64,
    #[field_name("Detector X")] pub detector_x:f64,
    #[field_name("Detector Y")] pub detector_y:f64,
}

impl Default for HeightProbeTestObject{
    fn default() -> Self {
        Self {
            height: 100.0,
            detector_x: 0.0,
            detector_y: 0.0
        }
    }
}
