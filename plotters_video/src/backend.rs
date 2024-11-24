use super::VideoBackend;
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, VPos};
use plotters_backend::{
    BackendColor, BackendStyle, BackendTextStyle, DrawingBackend, DrawingErrorKind,
};

fn interpolate(rgb_src:u8,rgb_tgt:u8,alpha:f64)->u8{
    let delta = rgb_tgt as f64-rgb_src as f64;
    let base = rgb_src as f64;
    let raw = delta*alpha+base;
    raw.round() as u8
}

fn interpolate_color(rgb_src:(u8,u8,u8),rgb_tgt:(u8,u8,u8),alpha:f64)->(u8,u8,u8){
    (
        interpolate(rgb_src.0, rgb_tgt.0, alpha),
        interpolate(rgb_src.1, rgb_tgt.1, alpha),
        interpolate(rgb_src.2, rgb_tgt.2, alpha),
    )
}

impl DrawingBackend for VideoBackend{
    type ErrorType = crate::VideoBackendError;
    fn get_size(&self) -> (u32, u32) {
        (
            self.width as u32,
            self.height as u32,
        )
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        Ok(())
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.flush_frame().map_err(DrawingErrorKind::DrawingError)
    }

    fn draw_pixel(
        &mut self,
        point: plotters_backend::BackendCoord,
        color: BackendColor,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        // if point.0 < 0 || point.1 < 0
        //     || point.0 as usize >= self.width
        //     || point.1 as usize >= self.height
        // {
        //     return Ok(());
        // }
        if let Some(src_color) = self.get_color(point.0 as u32, point.1 as u32){
            let new_color = interpolate_color(src_color, color.rgb, color.alpha);
            self.set_color(point.0 as u32, point.1 as u32,new_color);
        }
        Ok(())
    }

    // fn draw_line<S: BackendStyle>(
    //         &mut self,
    //         from: plotters_backend::BackendCoord,
    //         to: plotters_backend::BackendCoord,
    //         style: &S,
    //     ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
    //     plotters_backend::rasterizer::draw_line(self, from, to, style)
    // }
    //
    // fn draw_rect<S: BackendStyle>(
    //     &mut self,
    //     upper_left: plotters_backend::BackendCoord,
    //     bottom_right: plotters_backend::BackendCoord,
    //     style: &S,
    //     fill: bool,
    // ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
    //     plotters_backend::rasterizer::draw_rect(self, upper_left, bottom_right, style, fill)
    // }

    // fn blit_bitmap(
    //     &mut self,
    //     pos: plotters_backend::BackendCoord,
    //     (iw, ih): (u32, u32),
    //     src: &[u8],
    // ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
    //     todo!()
    // }
}
