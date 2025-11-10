use padamo_iced_forms::{IcedForm, IcedFormBuffer};
use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Debug, IcedForm, Default, Serialize, Deserialize)]
pub enum DisplayMode{
    #[default] Seconds,
    Time,
    GTU
}

impl DisplayMode{
    pub fn is_temporal(&self)->bool{
        match self{
            Self::Seconds | Self::Time => true,
            Self::GTU => false,
        }
    }

    pub fn format_x(&self, x:f64, start_unixtime:f64)->String{
        match self{
            Self::Seconds=>  format!("{:.3}",x-start_unixtime),
            Self::GTU=> format!("{}", x.round() as usize),
            Self::Time=>{
                use chrono::prelude::*;
                let start = x as i64;
                let add = ((x - start as f64)*1e9) as u32;
                // println!();
                let ut = Utc.timestamp_opt(start, add).unwrap();
                ut.format("%Y-%m-%d %H:%M:%S").to_string()
            },
        }
    }
}

#[derive(IcedForm, Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub enum LCSelection{
    #[default] Off,
    Selected,
    All
}

#[derive(IcedForm, Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct LCDisplay{
    #[field_name("LC")] pub selection: LCSelection,
    #[field_name("Mean")] pub mean: bool,
    #[field_name("Only")] pub only: bool
}

#[derive(IcedForm, Clone, Debug, Serialize, Deserialize)]
pub struct PlotSettings{
    #[field_name("Display mode")] pub display_mode:DisplayMode,
    #[field_name("LC display")] pub lc_display:LCDisplay,
    #[field_name("Min value")] pub min_value:f64,
    #[field_name("Max value")] pub max_value:f64
}

impl Default for PlotSettings{
    fn default() -> Self {
        Self {
            display_mode: DisplayMode::Time,
            lc_display: Default::default(),
            min_value: 0.0,
            max_value: 100.0,
        }
    }
}

#[derive(IcedForm, Clone, Debug, Serialize, Deserialize)]
pub struct SecondaryDetector{
    #[field_name("Detector ID")] pub detector_id:usize,
    #[field_name("Plot settings")] pub plot_settings:PlotSettings,
}

impl Default for SecondaryDetector{
    fn default() -> Self {
        Self {
            detector_id: 1,
            plot_settings:Default::default()
        }
    }
}

// #[derive(IcedForm, Clone, Debug)]
// pub enum Cho

#[derive(IcedForm, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum CurrentView{
    #[field_name("Plots")]Plots,
    #[field_name("Pixelmap (pri)")] ChoosingPrimary,
    #[field_name("Pixelmap (sec)")] ChoosingSecondary
}

impl Default for CurrentView{
    fn default() -> Self {
        Self::Plots
    }
}

#[derive(IcedForm, Clone, Debug, Serialize, Deserialize, Default)]
#[spoiler_hidden]
pub struct DisplayForm{
    #[field_name("Primary detector")]  primary:PlotSettings,
    #[field_name("Secondary detector")] secondary:Option<SecondaryDetector>,
}

impl DisplayForm{
    pub fn get_aux_id(&self)->Option<usize>{
        if let Some(det) = &self.secondary{
            Some(det.detector_id)
        }
        else{
            None
        }
    }

    pub fn get_primary_settings(&self)->&PlotSettings{
        &self.primary
    }

    pub fn get_secondary_settings(&self)->Option<&PlotSettings>{
        if let Some(sec) = &self.secondary{
            Some(&sec.plot_settings)
        }
        else{
            None
        }
    }
}

#[derive(IcedForm, Clone, Debug, Serialize, Deserialize)]
pub struct PlotterForm{
    #[field_name("Frames safeguard")] pub frames_safeguard:usize,
    #[field_name("Current view")] pub current_view: CurrentView,
    #[field_name("Display settings")] pub display_settings: DisplayForm,
    #[field_name("Selection")] pub selector: super::bulk_selector::SelectForm,
}

impl Default for PlotterForm{
    fn default() -> Self {
        Self {
            // plot_settings: Default::default(),
            // secondary_detector: None,
            frames_safeguard: 30_000,
            current_view: Default::default(),
            display_settings: Default::default(),
            selector: Default::default(),
        }
    }
}

/*
impl PlotterForm{
    pub fn get_aux_id(&self)->Option<usize>{
        self.display_settings.get_aux_id()
    }

    pub fn get_primary_settings(&self)->&PlotSettings{
        self.display_settings.get_primary_settings()
    }

    pub fn get_secondary_settings(&self)->Option<&PlotSettings>{
        self.display_settings.get_secondary_settings()
    }
}*/
