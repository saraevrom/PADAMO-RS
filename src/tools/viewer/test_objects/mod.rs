pub mod height_probe;

use padamo_detectors::{loaded_detectors_storage::ProvidedDetectorInfo, mesh::Mesh};
use padamo_iced_forms::{IcedForm,IcedFormBuffer};
use serde::{Deserialize, Serialize};



// #[derive(Clone,Debug)]
// pub struct CustomTestObject{
//
// }

#[derive(Clone,Debug, IcedForm, Serialize, Deserialize)]
pub enum TestObjectSelector{
    #[field_name("None")] None,
    #[field_name("Height probe")] HeightProbe(height_probe::HeightProbeTestObject),
    // #[field_name("Custom")] Custom,
}

impl Default for TestObjectSelector{
    fn default() -> Self {
        Self::None
    }
}

impl TestObjectSelector{
    pub fn generate_mesh(&self, provided_detector:Option<&ProvidedDetectorInfo>)->Option<Mesh>{
        match self {
            Self::None=>None,
            Self::HeightProbe(p)=> p.generate_mesh(provided_detector),
        }
    }
}

#[derive(Clone,Copy,Debug, IcedForm, Serialize, Deserialize)]
pub enum Relation{
    #[field_name("Origin")] Origin,
    #[field_name("Primary detector")] Primary,
    #[field_name("Auxiliary detector")] Auxiliary(usize),
}

impl Default for Relation{
    fn default() -> Self {
        Relation::Origin
    }
}

impl Relation{
    pub fn get_detector_id(&self)->Option<usize>{
        match self {
            Self::Origin => None,
            Self::Primary => Some(0),
            Self::Auxiliary(x) => Some(*x),
        }
    }
}

#[derive(Clone,Debug, Default, IcedForm, Serialize, Deserialize)]
#[spoiler_hidden]
pub struct TestObject{
    #[field_name("Object type")] pub selected_object:TestObjectSelector,
    #[field_name("Relative to")] pub relative_to:Relation,
}

