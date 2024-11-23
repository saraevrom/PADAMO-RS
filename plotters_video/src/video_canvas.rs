use std::path::Path;

use ndarray::Array3;

use video_rs::encode::{Encoder, Settings};
use video_rs::time::Time;


pub struct VideoBackend{
    encoder:Encoder,
    pub width:usize,
    pub height:usize,
    pub buffer:Array3<u8>,
    current_position:Time,
    duration:Time,
    canvas_edited:bool,
}

impl VideoBackend{
    pub fn new<T:AsRef<Path>>(destination:T,width:usize,height:usize, fps:usize)->Result<Self,video_rs::Error>{
        let encoder = Encoder::new(destination.as_ref(), Settings::preset_h264_yuv420p(width, height, true))?;
        let mut buffer = Array3::zeros((height,width,3));
        buffer.fill(255);
        let current_position = Time::zero();
        Ok(Self{encoder,width,height,buffer,current_position,duration:Time::from_nth_of_a_second(fps), canvas_edited:false})
    }

    pub fn clear_buffer(&mut self){
        self.buffer.fill(255);
    }

    pub fn flush_frame(&mut self)->Result<(),video_rs::Error>{
        if self.canvas_edited{
            self.encoder.encode(&self.buffer, self.current_position)?;
            self.clear_buffer();
            self.current_position = self.current_position.aligned_with(self.duration).add();
            self.canvas_edited = false;
        }
        Ok(())
    }

    fn finish(mut self)->Result<(),video_rs::Error>{
        self.encoder.finish()?;
        Ok(())
    }

    pub fn get_color(&self,x:usize,y:usize)->Option<(u8,u8,u8)>{
        let r = if let Some(v) = self.buffer.get((y,x,0)) {v} else {return None;};
        let g = if let Some(v) = self.buffer.get((y,x,1)) {v} else {return None;};
        let b = if let Some(v) = self.buffer.get((y,x,2)) {v} else {return None;};
        Some((*r,*g,*b))
    }

    fn set_color_unchecked(&mut self,x:usize,y:usize, color:(u8,u8,u8)){
        self.buffer[[y,x,0]] = color.0;
        self.buffer[[y,x,1]] = color.1;
        self.buffer[[y,x,2]] = color.2;
        self.canvas_edited = true;
    }

    pub fn set_color(&mut self,x:usize,y:usize, color:(u8,u8,u8)){
        if x>=self.width || y>=self.height{
            return;
        }
        self.set_color_unchecked(x, y, color);
    }
}


impl Drop for VideoBackend{
    fn drop(&mut self) {
        let _ = self.flush_frame();
        let _ = self.encoder.finish();
    }
}
