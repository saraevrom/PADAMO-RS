pub mod mesh;
pub mod height_probe;

use padamo_iced_forms::{IcedForm,IcedFormBuffer};



// #[derive(Clone,Debug)]
// pub struct CustomTestObject{
//
// }

#[derive(Clone,Debug, IcedForm)]
pub enum TestObjectSelector{
    #[field_name("Height probe")] HeightProbe(height_probe::HeightProbeTestObject),
    // #[field_name("Custom")] Custom,
}

impl Default for TestObjectSelector{
    fn default() -> Self {
        Self::HeightProbe(Default::default())
    }
}


#[derive(Clone,Debug, Default, IcedForm)]
#[spoiler_hidden]
pub struct TestObject{
    #[field_name("Object type")] pub selected_object:TestObjectSelector,
}

