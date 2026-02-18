use std::path::Path;

use playa_ffmpeg::{Packet, Rational, codec::Context, encoder, error::EAGAIN, format::context, rescale::TIME_BASE, util::frame};


// There is a lot of code from video-rs crate. Giving them credit.
// https://github.com/oddity-ai/video-rs

pub struct VideoFrameByFrameWriter{
    buffer: frame::Video,
    buffer_changed:bool,
    done:bool,
    begun:bool,
    frame_count: u64,
    width:u32,
    true_width:u32,
    height:u32,
    output: context::Output,
    encoder: encoder::video::Encoder,
    scaler: playa_ffmpeg::software::scaling::context::Context,
    keyframe_interval: u64,
    writer_stream_index: usize,
    encoder_time_base:Rational,
}

pub fn codec_context_as(codec: &playa_ffmpeg::codec::codec::Codec) -> Option<Context> {
    unsafe {
        let context_ptr = playa_ffmpeg::ffi::avcodec_alloc_context3(codec.as_ptr());
        if !context_ptr.is_null() {
            Some(Context::wrap(context_ptr, None))
        } else {
            None
        }
    }
}

pub fn get_encoder_time_base(encoder: &encoder::video::Encoder) -> Rational {
    unsafe { (*encoder.0.as_ptr()).time_base.into() }
}

impl VideoFrameByFrameWriter {
    pub fn new<T:AsRef<Path>+?Sized>(destination:&T, width:u32, height:u32) -> Result<Self,crate::VideoBackendError> {

        playa_ffmpeg::init()?;
        println!("Init - OK");
        let mut output = playa_ffmpeg::format::output(destination)?;

        println!("Output - OK");
        let global_header = output
            .format()
            .flags()
            .contains(playa_ffmpeg::format::Flags::GLOBAL_HEADER);

        let mut writer_stream = output.add_stream(None)?;
        let writer_stream_index = writer_stream.index();

        println!("Stream - OK");
        let codec = playa_ffmpeg::encoder::find_by_name("libx264").unwrap_or_else(|| playa_ffmpeg::encoder::find(playa_ffmpeg::codec::Id::H264).unwrap());
        let mut encoder_context = codec_context_as(&codec).unwrap_or_else(playa_ffmpeg::codec::Context::new);
        println!("Encoder context - OK");

        if global_header{
            encoder_context.set_flags(playa_ffmpeg::codec::Flags::GLOBAL_HEADER);
        }

        let mut encoder = encoder_context.encoder().video()?;
        encoder.set_width(width);
        encoder.set_height(height);
        encoder.set_format(playa_ffmpeg::format::Pixel::YUV420P);
        encoder.set_frame_rate(Some((30, 1))); //30 FPS. Will be changed

        println!("Encoder - OK");
        encoder.set_time_base(TIME_BASE);

        let mut options = playa_ffmpeg::Dictionary::new();
        options.set("preset", "medium");

        let encoder = encoder.open_with(options)?;
        let encoder_time_base = get_encoder_time_base(&encoder);
        println!("Encoder open - OK");

        writer_stream.set_parameters(&encoder);

        let scaler_width = encoder.width();
        let scaler_height = encoder.height();
        let scaler = playa_ffmpeg::software::scaling::context::Context::get(
            playa_ffmpeg::format::Pixel::RGB24,
            scaler_width,
            scaler_height,
            encoder.format(),
            scaler_width,
            scaler_height,
            playa_ffmpeg::software::scaling::flag::Flags::empty(),
        )?;
        let buffer = frame::Video::new(playa_ffmpeg::format::Pixel::RGB24, scaler_width, scaler_height);

        let true_width = (buffer.data(0).len() as u32)/3/scaler_height;
        println!("Widths: Passed={} Scaler={} True={}", width, scaler_width, true_width);

        let mut res = Self {
            output, encoder,
            scaler,
            buffer_changed:false,
            done:false,
            begun:false,
            frame_count:0,
            keyframe_interval: 12, // TODO: Allow changes
            writer_stream_index,
            true_width,
            width:scaler_width,
            height:scaler_height,
            encoder_time_base,
            buffer, //width, height,
        };
        res.clear_buffer();

        Ok(res)
    }

    fn clear_buffer(&mut self){
        self.buffer.data_mut(0).fill(255);
        self.buffer_changed = false;
    }

    fn get_pixel_offset(&self, x:usize, y:usize)->usize{
        // let width = self.buffer.data(0).len()/3/(self.height as usize); // Actual width of image is cursed. there is a way of mitigating this
        (self.true_width as usize)*y+x
        // x
    }

    pub fn set_pixel(&mut self, x:usize, y:usize, rgb:(u8,u8,u8)){
        self.buffer_changed = true;
        let pixel_offset = self.get_pixel_offset(x, y);
        let dat = self.buffer.data_mut(0);
        //let pixel_offset = (self.width as usize)*y+x;
        let r_index = pixel_offset*3;
        let g_index = r_index+1;
        let b_index = r_index+2;
        let (r,g,b) = rgb;
        if let Some(r_tgt)= dat.get_mut(r_index){
            *r_tgt = r;
        }
        if let Some(g_tgt)= dat.get_mut(g_index){
            *g_tgt = g;
        }
        if let Some(b_tgt)= dat.get_mut(b_index){
            *b_tgt = b;
        }
    }

    pub fn get_pixel_signed(&self, x:i32, y:i32) -> Option<(u8,u8,u8)>{
        if x<0 || (x as u32) >= self.width{
            return None;
        }
        if y<0 || (x as u32) >= self.height{
            return None;
        }
        self.get_pixel(x as usize, y as usize)
    }

    pub fn get_pixel(&self, x:usize, y:usize) -> Option<(u8,u8,u8)>{
        let pixel_offset = self.get_pixel_offset(x, y);
        let r_off = pixel_offset*3;
        let dat = self.buffer.data(0);
        let r = *dat.get(r_off)?;
        let g = *dat.get(r_off+1)?;
        let b = *dat.get(r_off+2)?;
        Some((r,g,b))
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width,self.height)
    }

    fn stream_time_base(&mut self) -> Rational {
        self.output
        .stream(self.writer_stream_index)
        .unwrap()
        .time_base()
    }

    pub fn flush_frame(&mut self)->Result<(),crate::VideoBackendError>{
        let mut frame_scaled = frame::Video::empty();
        self.scaler.run(&self.buffer, &mut frame_scaled)?;
        frame_scaled.set_pts(Some(self.frame_count as i64));

        if self.frame_count%self.keyframe_interval==0{
            frame_scaled.set_kind(playa_ffmpeg::picture::Type::I);
        }
        if !self.begun{
            self.output.write_header()?;
            self.begun = true;
        }
        self.encoder.send_frame(&frame_scaled)?;
        self.clear_buffer();
        self.frame_count+= 1;

        if let Some(packet) = self.encoder_receive_packet()? {
            // self.write(packet)?;
            self.write(packet)?;
            // self.writer.write(&mut packet)?;
        }

        Ok(())
    }

    fn write(&mut self, mut packet: Packet) -> Result<(), crate::VideoBackendError> {
        packet.set_stream(self.writer_stream_index);
        packet.set_position(-1);
        packet.rescale_ts(self.encoder_time_base, self.stream_time_base());
        packet.write(&mut self.output)?;
        Ok(())
    }

    fn encoder_receive_packet(&mut self) -> Result<Option<Packet>, crate::VideoBackendError> {
        let mut packet = Packet::empty();
        let encode_result = self.encoder.receive_packet(&mut packet);
        match encode_result {
            Ok(()) => Ok(Some(packet)),
            Err(playa_ffmpeg::Error::Other { errno }) if errno == EAGAIN => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    pub fn finish(&mut self)->Result<(),crate::VideoBackendError>{
        if self.done{
            return Ok(());
        }
        if self.buffer_changed{
            self.flush_frame()?;
        }
        self.flush()?;
        self.output.write_trailer()?;
        self.done = true;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), crate::VideoBackendError> {
        // Maximum number of invocations to `encoder_receive_packet`
        // to drain the items still on the queue before giving up.
        const MAX_DRAIN_ITERATIONS: u32 = 100;

        // Notify the encoder that the last frame has been sent.
        self.encoder.send_eof()?;

        // We need to drain the items still in the encoders queue.
        for _ in 0..MAX_DRAIN_ITERATIONS {
            match self.encoder_receive_packet() {
                Ok(Some(packet)) => self.write(packet)?,
                Ok(None) => continue,
                Err(_) => break,
            }
        }

        Ok(())
    }
}

impl Drop for VideoFrameByFrameWriter{
    fn drop(&mut self) {
        let _ = self.finish();
    }
}


unsafe impl Send for VideoFrameByFrameWriter{}
unsafe impl Sync for VideoFrameByFrameWriter{}
