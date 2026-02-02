use plotters::prelude::*;

use crate::polygon::DetectorContent;

pub fn display_pixel_id<DB:DrawingBackend>(
    detector: &DetectorContent,
    root: &DrawingArea<DB, plotters::coord::Shift>,
    value_source:&dyn super::traits::ColorValueSource,
    state: &super::modular_diagram::PadamoDetectorDiagramState
) {
    if let Some(index) = detector.position_index(state.pos){
        let mut unmapped_pos = state.unmapped;
        unmapped_pos.1 -= 20;
        let txt = if let Some(val) = value_source.get_value(index){
            format!("{:?} {:.3}",index,val)
        }
        else{
            format!("{:?} MAPPING INVALID",index)
        };
        root.draw_text(&txt, &(("sans-serif", 15).into()), unmapped_pos).unwrap();
    }
}
