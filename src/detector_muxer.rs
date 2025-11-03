
pub const VIEWER_PRIMARY_SIGNAL_VAR:&'static str = "ViewerSignal";
pub const VIEWER_PRIMARY_MASK_VAR:&'static str = "alive_pixels";
pub const VIEWER_TEST_OBJECT_KEY:&'static str = "test_object_transform";

pub fn get_signal_var(id:usize)->String{
    if id==0{
        VIEWER_PRIMARY_SIGNAL_VAR.into()
    }
    else{
        format!("ViewerSignalAux{}",id)
    }
}

pub fn get_mask_var(id:usize)->String{
    if id==0{
        VIEWER_PRIMARY_MASK_VAR.into()
    }
    else{
        format!("alive_pixels_aux_{}",id)
    }
}

pub fn get_transform_var(id:usize)->String{
    if id==0{
        "detector_transform".into()
    }
    else{
        format!("detector_transform_aux_{}",id)
    }
}
