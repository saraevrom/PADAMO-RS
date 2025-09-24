use std::path::{Path, PathBuf};

// use ndarray::Array3;

// use video_rs::encode::{Encoder, Settings};
// use video_rs::time::Time;
use std::io::{Cursor, Read, Seek, SeekFrom};

use image::{EncodableLayout, RgbImage};
// use imageproc::drawing::draw_filled_circle_mut;
use minimp4::Mp4Muxer;

use openh264::encoder::{Encoder, EncoderConfig};


//Firstly I tried using video_rs. But building FFMPEG in github is nontrivial.
// Switched to h264 solution found here: https://users.rust-lang.org/t/create-mp4-video-from-pictures-in-rust/70850/3

#[derive(Clone, Copy,Debug)]
pub enum FrameDelay{
    DelayMS(usize),
    FPS(usize)
}

impl FrameDelay{
    // pub fn to_delay(self)->Time{
    //     match self {
    //         Self::FPS(fps) => Time::from_nth_of_a_second(fps),
    //         Self::DelayMS(ms) => Time::from_secs_f64((ms as f64)*0.001)
    //     }
    // }
    pub fn to_fps(self)->u32{
        match self {
            Self::FPS(fps) => fps as u32,
            Self::DelayMS(ms) => (1000/ms) as u32
        }
    }
}

pub struct VideoBackend{
    // encoder:Encoder,
    pub width:u32,
    pub height:u32,
    pub frame_delay_data:FrameDelay,
    target:PathBuf,
    // pub buffer:Array3<u8>,/*
    // current_position:Time,
    // duration:Time,*/
    pub image_buffer:RgbImage,
    video_buffer:Vec<u8>,
    pub canvas_edited:bool,
    encoder:Encoder,
}


impl VideoBackend{
    pub fn new<T:AsRef<Path>>(destination:T,width:u32,height:u32, delay:FrameDelay)->Result<Self,crate::errors::VideoBackendError>{
        // let encoder = Encoder::new(destination.as_ref(), Settings::preset_h264_yuv420p(width, height, true))?;
        // let mut buffer = Array3::zeros((height,width,3));
        // buffer.fill(255);
        // let current_position = Time::zero();
        // Ok(Self{encoder,width,height,buffer,current_position,duration:delay.to_delay(), canvas_edited:false})
        let config = EncoderConfig::new();
        let encoder = Encoder::with_api_config(openh264::OpenH264API::from_source(), config)?;
        let image_buffer = RgbImage::new(width, height);
        let target = destination.as_ref().to_owned();
        let video_buffer = Vec::new();
        Ok(Self{width,height, canvas_edited:false, encoder, image_buffer, video_buffer, target, frame_delay_data:delay})
    }

    pub fn clear_buffer(&mut self){
        self.image_buffer.fill(255);
    }

    pub fn flush_frame(&mut self)->Result<(),crate::errors::VideoBackendError>{
        if self.canvas_edited{
            let frame = self.image_buffer.as_bytes().to_vec();
            //let mut yuv = openh264::formats::RBGYUVConverter::new(512, 512);
            let mut yuv = openh264::formats::YUVBuffer::new(self.width as usize, self.height as usize);
            let frame_src = openh264::formats::RgbSliceU8::new(&frame, (self.width as usize, self.height as usize));
            yuv.read_rgb(frame_src);
            //yuv.convert(&frame[..]);
            let bitstream = self.encoder.encode(&yuv)?;
            bitstream.write_vec(&mut self.video_buffer);
            self.clear_buffer();
            self.canvas_edited = false;
        }
        Ok(())
    }

    #[allow(unused)]
    fn finish(&mut self)->Result<(),crate::VideoBackendError>{
        self.flush_frame();
        let mut video_buffer = Cursor::new(Vec::new());
        let mut mp4muxer = Mp4Muxer::new(&mut video_buffer);
        mp4muxer.init_video(self.width as i32, self.height as i32, false, "Plot");
        mp4muxer.write_video_with_fps(&self.video_buffer, self.frame_delay_data.to_fps());
        mp4muxer.close();

        // Some shenanigans to get the raw bytes for the video.
        video_buffer.seek(SeekFrom::Start(0))?;
        let mut video_bytes = Vec::new();
        video_buffer.read_to_end(&mut video_bytes)?;

        std::fs::write(&self.target, &video_bytes)?;

        Ok(())
    }

    pub fn get_color(&self,x:u32,y:u32)->Option<(u8,u8,u8)>{
        let pixel = self.image_buffer.get_pixel_checked(x, y);
        pixel.map(|pix| {
            let rgb = pix.0;
              (rgb[0],rgb[1],rgb[2])
        })

        // let r = if let Some(v) = self.buffer.get((y,x,0)) {v} else {return None;};
        // let g = if let Some(v) = self.buffer.get((y,x,1)) {v} else {return None;};
        // let b = if let Some(v) = self.buffer.get((y,x,2)) {v} else {return None;};
        // Some((*r,*g,*b))
    }

    // fn set_color_unchecked(&mut self,x:usize,y:usize, color:(u8,u8,u8)){
    //     if let Some(pixel )= self.image_buffer.get_pixel_mut_checked(, )
    // }

    pub fn set_color(&mut self,x:u32,y:u32, color:(u8,u8,u8)){
        if let Some(pixel) = self.image_buffer.get_pixel_mut_checked(x, y){
            pixel.0[0] = color.0;
            pixel.0[1] = color.1;
            pixel.0[2] = color.2;
            self.canvas_edited = true;
        }
    }
}


impl Drop for VideoBackend{
    fn drop(&mut self) {
        let _ = self.finish();
    }
}
