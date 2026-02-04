use padamo_iced_forms::ActionOrUpdate;
use super::sub_plotter::SubplotterMessage;


// #[derive(Clone, Debug)]
// pub struct PokedPixel{
//     pub detector_id: usize,
//     pub pixel_id: Vec<usize>
// }

#[derive(Clone, Debug)]
pub enum NewPlotterMessage{
    FormMessage(ActionOrUpdate<super::form::PlotterFormMessage>),
    PrimarySubplotterMessage(SubplotterMessage),
    SecondarySubplotterMessage(SubplotterMessage),
    SyncData{start:usize, end:usize, pointer:Option<usize>},
}
