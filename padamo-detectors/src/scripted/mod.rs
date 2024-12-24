
use crate::polygon::DetectorContent;
use rhai::{Engine, EvalAltResult};
// pub mod errors;

pub fn parse_scripted(src:&str)->Result<DetectorContent,Box<EvalAltResult>>{
    let mut engine = Engine::new();
    engine.build_type::<crate::polygon::DetectorPixel>();
    engine.build_type::<crate::polygon::DetectorContent>();
    let res:DetectorContent = engine.eval(src)?;
    // let res1:DetectorContent = from_dynamic(&res)?;
    Ok(res)
}
