use clipboard_rs::{Clipboard, ClipboardContext};

// use std::io::prelude::*;
// use flate2::Compression;
// use flate2::write::ZlibEncoder;
// use flate2::write::ZlibDecoder;

use super::nodes::GraphNodeCloneBufferSerializable;

use base64::prelude::*;


pub fn copy_to_clipboard(start:&GraphNodeCloneBufferSerializable)->anyhow::Result<()>{
    let serialized = serde_cbor::to_vec(start)?;

    // let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    // e.write_all(&serialized)?;
    // let serialized = e.finish()?;

    // let txt = serialized.encode_hex();
    let txt = BASE64_STANDARD.encode(serialized);

    let ctx = ClipboardContext::new().map_err(|x| anyhow::format_err!("{}",x))?;
    // let mut clipboard = clippers::Clipboard::get();
    // clipboard.write_text(txt)?;
    ctx.set_text(txt).map_err(|x| anyhow::format_err!("{}",x))?;
    Ok(())
}

pub fn paste_from_clipboard()->anyhow::Result<GraphNodeCloneBufferSerializable>{
    // let mut clipboard = clippers::Clipboard::get();
    let ctx = ClipboardContext::new().map_err(|x| anyhow::format_err!("{}",x))?;
    match ctx.get_text(){
        Ok(text)=>{
            // let serialized:Vec<u8> = FromHex::from_hex(text)?;
            let serialized = BASE64_STANDARD.decode(text)?;

            // let mut d = ZlibDecoder::new(Vec::new());
            // d.write_all(&serialized)?;
            //
            // let serialized = d.finish()?;

            let res = serde_cbor::from_slice(&serialized)?;
            Ok(res)
        },
        Err(e)=>{
            Err(anyhow::format_err!("{}",e))
        }
    }
}
