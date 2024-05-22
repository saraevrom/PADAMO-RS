use std::{fmt::Debug, sync::Arc};

use abi_stable::type_level::trait_marker::Display;
use padamo_api::lazy_array_operations::{cutter::CutError, ndim_array::indexing::ShapeIterator, ArrayND, LazyArrayOperation, LazyArrayOperationBox, LazyDetectorSignal, LazyTrigger};
// use tract_core::ndarray::{IxDyn, OwnedRepr};
// use tract_onnx::prelude::*;
use ndarray::prelude::*;


#[derive(thiserror::Error,Debug,Clone,Copy)]
pub enum ANNError {
    #[error("THis ANN supports only 3D data")]
    DimensionMisalignmentError,
    #[error("Source array is empty")]
    EmptySource,
    #[error("Source array is too small")]
    SmallSource,
    #[error("Source array dimension {dimension} has incompatible sizes. Source: {src}, found {hint}")]
    IncompatibleShape {dimension:usize, src:usize, hint:usize},
    #[error("Trigger has insufficient coverage")]
    BadCoverage,
    #[error("Source signal is misaligned")]
    Misaligned
}

// #[derive(Clone,Copy,Debug)]
// pub struct DimensionMisalignmentError{
//     srclen:usize,
//     hintsize:usize
// }
//
// impl std::fmt::Display for DimensionMisalignmentError{
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "ANN dimensions are not aligned: expected {}, but found {}", self.srclen, self.hintsize)
//     }
// }
//
// impl Error for DimensionMisalignmentError{}




#[derive(Clone)]
pub struct LazyANNTrigger3D{
    //path:String,
    model:Arc<ort::Session>,
    //model:SimplePlan<TypedFact,Box<dyn TypedOp>,Graph<TypedFact,Box<dyn TypedOp>>>,
    source:LazyDetectorSignal,
    threshold:f32,
    stride:usize,
    size_hint:(usize,usize,usize),
    output_layer:String,
    //output_shape:Vec<usize>,
}

impl std::fmt::Debug for LazyANNTrigger3D{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"LazyANNTrigger{{source:{:?}, threshold:{:?},stride:{:?}}}", self.source,self.threshold, self.stride)
    }
}

impl LazyANNTrigger3D{
    pub fn align_data<T:Clone+Debug+'static>(source:LazyArrayOperationBox<T>,stride:usize, size_hint:(usize,usize,usize))->Result<LazyArrayOperationBox<T>,CutError>{
        let source_length = source.length();
        let last_window_index = source_length-size_hint.0;
        let last_pack_index = last_window_index/stride*stride;
        let aligned_length = last_pack_index+size_hint.0;
        source.cut(0, aligned_length)
    }

    pub fn new(model: Arc<ort::Session>, source:LazyDetectorSignal,threshold:f32,stride:usize,size_hint:(usize,usize,usize),output_layer:String)-> Result<Self, Box<dyn std::error::Error>>{
        let source_length = source.length();
        if source_length==0{
            return Err(Box::new(ANNError::EmptySource));
        }
        let probe:ArrayND<f64> = source.request_range(0,1);

        if probe.shape.len() != 3{
            return Err(Box::new(ANNError::DimensionMisalignmentError));
        }
        let width = probe.shape[1];
        let height = probe.shape[2];

        if source_length<size_hint.0{
            return Err(Box::new(ANNError::SmallSource));
        }

        if width%size_hint.1!=0{
            return Err(Box::new(ANNError::IncompatibleShape { dimension: 1, src: width, hint: size_hint.1}));
        }

        if height%size_hint.2!=0{
            return Err(Box::new(ANNError::IncompatibleShape { dimension: 1, src: height, hint: size_hint.2}));
        }


        if stride>size_hint.0 || stride==0{
            return Err(Box::new(ANNError::BadCoverage));
        }

        //let raw_length = source_length-size_hint[0]+1;

        let last_window_index = source_length-size_hint.0;
        if last_window_index%stride!=0{
            return Err(Box::new(ANNError::Misaligned));
        }

        Ok(Self { model, source,threshold, stride,size_hint, output_layer})
    }
}

impl LazyArrayOperation<ArrayND<bool>> for LazyANNTrigger3D{
    #[allow(clippy::let_and_return)]
    fn length(&self,) -> usize {
        self.source.length()
    }

    #[allow(clippy::let_and_return)]
    fn request_range(&self,start:usize,end:usize,) -> ArrayND<bool> where {
        let window = self.size_hint.0;
        let len = self.length();
        let snapped_start = start/self.stride*self.stride;

        let cut_start = if snapped_start+self.stride>=window{
            snapped_start+self.stride-window
        }
        else{
            0
        };

        let snapped_end = end/self.stride*self.stride;
        let mut cut_end = snapped_end+window;
        //println!("Precut {}, {}", cut_start,cut_end);
        if cut_end>len{
            cut_end = len;
        }
        let cut_end = cut_end;
        println!("Cut {}, {}", cut_start,cut_end);

        let source_data = self.source.request_range(cut_start,cut_end);
        let src_time = source_data.shape[0];
        let src_width = source_data.shape[1];
        let src_height = source_data.shape[2];

        let blocks_w = src_width/self.size_hint.1;
        let blocks_h = src_height/self.size_hint.2;
        let source_data = source_data.to_ndarray();
        let source_data = source_data.map(|x| *x as f32);

        //assert_eq!((src_time-self.size_hint.0)/self.stride,0);

        let mut res = Array::<bool,_>::default((end-start,blocks_w,blocks_h));

        println!("Blocks {}, {}", blocks_w, blocks_h);

        for i in 0usize..blocks_w{
            for j in 0usize..blocks_h{
                println!("Block {}, {}", i, j);
                let block = source_data.slice(ndarray::s![..,i..i+self.size_hint.1,j..j+self.size_hint.2]);
                println!("Slice 1 OK");
                let windows_amount = (src_time-self.size_hint.0)/self.stride+1;
                println!("There are {} windows",windows_amount);
                let mut slided = Array::<f32,_>::zeros((windows_amount,self.size_hint.0,self.size_hint.1,self.size_hint.2));
                for k in 0..windows_amount{
                    let base = k*self.stride;
                    let src = block.slice(ndarray::s![base..base+self.size_hint.0,..,..]);
                    slided.slice_mut(ndarray::s![k,..,..,..]).zip_mut_with(&src, |a,b|{*a = *b});
                }


                println!("Slice 2 OK");


                let outputs = self.model.run(ort::inputs!["input" => slided.view()].unwrap()).unwrap();
                let output = outputs[self.output_layer.as_str()].try_extract_tensor::<f32>().unwrap().into_owned();

                // let input:Tensor = slided.into();
                // let found = self.model.run(tvec![input.into()]);
                // println!("ANN OK");
                // let found = found.unwrap().remove(0);
                // println!("ANN unwrap OK");
                // let found_array = found.to_array_view::<f32>().unwrap();

                let triggered = output.map(|x| *x>self.threshold);
                let triggered = triggered.fold_axis(Axis(1), false, |a,b| {*a || *b});
                println!("ANN threshold OK {:?}",triggered);


                let mut expanded = Array::<bool,_>::default(src_time);
                triggered.iter().enumerate().for_each(|(i,v)|{
                    let base = i*self.stride;
                    //println!("BASE {}..{}",base,base+self.size_hint.0);
                    expanded.slice_mut(ndarray::s![base..base+self.size_hint.0]).mapv_inplace(|x|{
                        x || *v
                    });
                });
                println!("ANN deconv OK");

                let needed_part = expanded.slice(ndarray::s![start-cut_start..end-cut_start]);
                res.slice_mut(ndarray::s![..,i,j]).zip_mut_with(&needed_part, |a,b| {*a = *b;});

            }
        }

        res.into()
    }

    #[allow(clippy::let_and_return)]
    fn calculate_overhead(&self,start:usize,end:usize,) -> usize where {
        let window = self.size_hint.0;
        let len = self.length();
        let snapped_start = start/self.stride*self.stride;

        let cut_start = if snapped_start+self.stride>=window{
            snapped_start+self.stride-window
        }
        else{
            0
        };

        let snapped_end = end/self.stride*self.stride;
        let mut cut_end = snapped_end+window;
        if cut_end>len{
            cut_end = len;
        }
        let cut_end = cut_end;
        self.source.calculate_overhead(cut_start,cut_end)
    }
}
