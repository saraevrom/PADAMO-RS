use padamo_iced_forms::{IcedForm,IcedFormBuffer};
use padamo_iced_forms::make_action;
use padamo_iced_forms::Action;

#[derive(Clone,Debug,IcedForm)]
#[spoiler_hidden]
pub struct AnimationParameters{
    #[field_name("Create animation")] _start:Action<ViewerActions,ViewerActionStartAnimation>,
    #[field_name("Stop")] _stop:Action<ViewerActions,ViewerActionStopAnimation>,
    #[field_name("Width [pix]")] pub width:u32,
    #[field_name("Height [pix]")] pub height:u32,
    #[field_name("Frame delay [ms]")] pub framedelay:u32,
    #[field_name("Display LC")] pub displaylc:bool,
    #[field_name("LC height[pix]")] pub lcheight:u32,
    //#[field_name("Display LC")] displaylc:bool,
}

#[derive(Clone,Debug,IcedForm)]
#[spoiler_hidden]
pub struct ExportParameters{
    //#[field_name("Frames step")] pub framesstep:usize,
    #[field_name("Export")] _start:Action<ViewerActions,ViewerActionStartExport>,
    #[field_name("Stop")] _stop:Action<ViewerActions,ViewerActionStopExport>,
    #[field_name("RAM part")] pub rampart:f64,
    #[field_name("Deflate")] pub deflate:bool,
    #[field_name("Deflate level")] pub deflatelevel:u8,
    #[field_name("Signal field")] pub spatialfield:String,
    #[field_name("Time field")] pub temporalfield:String,
    #[field_name("HDF chunk length [frames]")] pub chunk:usize,
    //#[field_name("Display LC")] displaylc:bool,
}

#[derive(Clone,Debug,IcedForm)]
#[spoiler_hidden]
pub struct FrameParameters{
    #[field_name("Save frame")] _action:Action<ViewerActions,ViewerActionMakeFrame>,
    #[field_name("Width [pix]")] pub width:u32,
    #[field_name("Height [pix]")] pub height:u32,
}

#[derive(Clone,Debug,IcedForm, Default)]
pub struct ViewerForm{
    //#[field_name("Stop on trigger")] pub stop_on_trigger:bool,
    #[field_name("Animation")] pub animation:AnimationParameters,
    #[field_name("Export")] pub export:ExportParameters,
    #[field_name("Single Frame")] pub single_frame:FrameParameters,
    #[field_name("Test object")] pub test_object:super::test_objects::TestObject,
}

#[derive(Clone,Debug)]
pub enum ViewerActions{
    Noop,
    StartAnimation,
    StopAnimation,
    StartExport,
    StopExport,
    MakeFrame,
}

make_action!(ViewerActionNoop,ViewerActions,Noop);
make_action!(ViewerActionStartAnimation,ViewerActions,StartAnimation);
make_action!(ViewerActionStopAnimation,ViewerActions,StopAnimation);
make_action!(ViewerActionStartExport,ViewerActions,StartExport);
make_action!(ViewerActionStopExport,ViewerActions,StopExport);
make_action!(ViewerActionMakeFrame,ViewerActions,MakeFrame);

// #[derive(Clone,Debug,Default)]
// pub struct ViewerActionsNoop;
// impl

impl Default for ViewerActions{
    fn default() -> Self {
        ViewerActions::Noop
    }
}

// impl Default for ViewerForm{
//     fn default() -> Self {
//         Self {animation: Default::default(), export: Default::default(), single_frame: Default::default() }
//     }
// }

impl Default for FrameParameters{
    fn default() -> Self {
        FrameParameters { _action: Default::default(), width: 1024, height: 1024 }
    }
}

impl Default for AnimationParameters{
    fn default() -> Self {
        Self {
            width: 1024,
            height: 1024,
            framedelay:200,
            displaylc:false,
            lcheight: 200,
            _start:Default::default(),
            _stop:Default::default(),
        }
    }
}

impl Default for ExportParameters{
    fn default() -> Self {
        Self {
            deflate: true ,
            deflatelevel:3,
            spatialfield:"pdm_2d_rot_global".into(),
            temporalfield:"unixtime_dbl_global".into(),
            //framesstep:1,
            rampart:0.01,
            chunk:16,
            _start:Default::default(),
            _stop:Default::default(),
        }
    }
}
