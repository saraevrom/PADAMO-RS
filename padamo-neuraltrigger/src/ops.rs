use std::{fmt::Debug, sync::{Arc, Mutex}};

use ort::value::Tensor;
use padamo_api::{lazy_array_operations::{cutter::CutError, ArrayND, LazyArrayOperation, LazyArrayOperationBox, LazyDetectorSignal}, trigger_operations::SparseTagArray};
// use tract_core::ndarray::{IxDyn, OwnedRepr};
// use tract_onnx::prelude::*;
use ndarray::prelude::*;


#[derive(thiserror::Error,Debug,Clone,Copy)]
pub enum ANNError {
    #[error("This ANN supports only 3D data")]
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
    model:Arc<Mutex<ort::session::Session>>,
    //model:SimplePlan<TypedFact,Box<dyn TypedOp>,Graph<TypedFact,Box<dyn TypedOp>>>,
    source:LazyDetectorSignal,
    threshold:f32,
    stride:usize,
    size_hint:(usize,usize,usize),
    output_layer:String,
    squeeze_source:bool,
    tag_prefix:String,
    minimal_amplitude:f64
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

    pub fn new(model: Arc<Mutex<ort::session::Session>>, source:LazyDetectorSignal,threshold:f32,stride:usize,size_hint:(usize,usize,usize),output_layer:String,squeeze_source:bool, tag_prefix:String, minimal_amplitude:f64)-> Result<Self, Box<dyn std::error::Error>>{
        let source_length = source.length();
        if source_length==0{
            return Err(Box::new(ANNError::EmptySource));
        }
        let probe:ArrayND<f64> = source.request_range(0,1);

        // let probe_len:usize = if squeeze_source{
        //     probe.shape.iter().skip(1).filter(|x| **x!=1).count()+1
        // }
        // else{
        //     probe.shape.len()
        // };
        let probe_shape:Vec<usize> = if squeeze_source{
            probe.shape.iter().enumerate().filter(|(i,x)| **x != 1 || *i==0).map(|(_,x)| *x).collect()
        }
        else{
            probe.shape.clone().into()
        };

        if probe_shape.len() != 3{
            return Err(Box::new(ANNError::DimensionMisalignmentError));
        }
        let width = probe_shape[1];
        let height = probe_shape[2];

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

        Ok(Self { model, source,threshold, stride,size_hint, output_layer, squeeze_source, tag_prefix, minimal_amplitude})
    }
}

impl LazyArrayOperation<SparseTagArray> for LazyANNTrigger3D{
    #[allow(clippy::let_and_return)]
    fn length(&self,) -> usize {
        self.source.length()
    }

    #[allow(clippy::let_and_return)]
    fn request_range(&self,start:usize,end:usize,) -> SparseTagArray where {
        let window = self.size_hint.0;
        let len = self.length();
        let snapped_start = start/self.stride*self.stride;

        // let cut_start = if snapped_start+self.stride>=window{
        //     snapped_start+self.stride-window
        // }
        // else{
        //     0
        // };
        let cut_start = if snapped_start<start{
            snapped_start+self.stride
        }
        else{
            snapped_start
        };

        let mut cut_end = end/self.stride*self.stride+window;
        if cut_end>len{
            cut_end -=window;
        }
        // let mut cut_end = if end<snapped_end{
        //     snapped_end
        // }
        // else{
        //     snapped_end-
        // };

        if cut_start>=cut_end{
            return SparseTagArray::new();
        }

        // let mut cut_end = snapped_end+window;
        // //println!("Precut {}, {}", cut_start,cut_end);
        // if cut_end>len{
        //     cut_end = len;
        // }
        // let cut_end = cut_end;

        println!("Cut {}, {}", cut_start,cut_end);

        let mut source_data = self.source.request_range(cut_start,cut_end);
        if self.squeeze_source{
            let src_shape:Vec<usize> = source_data.shape.clone().into();
            let mut squeezed_shape = vec![src_shape[0]];
            squeezed_shape.extend(src_shape.iter().skip(1).filter(|x| **x!=1).map(|x| *x).collect::<Vec<usize>>());
            source_data.shape = squeezed_shape.into();
        }
        let source_data = source_data;


        let src_time = source_data.shape[0];
        let src_width = source_data.shape[1];
        let src_height = source_data.shape[2];

        let blocks_w = src_width/self.size_hint.1;
        let blocks_h = src_height/self.size_hint.2;
        let source_data = source_data.to_ndarray();
        let source_data = source_data.map(|x| *x as f32);

        //assert_eq!((src_time-self.size_hint.0)/self.stride,0);

        let mut res = SparseTagArray::new();

        println!("Blocks {}, {}", blocks_w, blocks_h);

        let windows_amount = (src_time-self.size_hint.0)/self.stride+1;
        let mut slided = Array::<f32,_>::zeros((windows_amount,self.size_hint.0,self.size_hint.1,self.size_hint.2));

        let min_ampl = self.minimal_amplitude as f32;

        for i in 0usize..blocks_w{
            for j in 0usize..blocks_h{
                println!("Block {}, {}", i, j);
                let block = source_data.slice(ndarray::s![..,i*self.size_hint.1..(i+1)*self.size_hint.1,j*self.size_hint.2..(j+1)*self.size_hint.2]);
                println!("Slice 1 OK");
                println!("There are {} windows",windows_amount);
                // let mut slided = Array::<f32,_>::zeros((windows_amount,self.size_hint.0,self.size_hint.1,self.size_hint.2));
                let viable = block.map(|x| *x > min_ampl).fold(false, |a,b| a || *b);
                // let full_amp = block.fold(0.0f32, |a,b| a.max(*b));
                if viable{
                    for k in 0..windows_amount{
                        let base = k*self.stride;
                        let src = block.slice(ndarray::s![base..base+self.size_hint.0,..,..]);
                        // let source_is_active = true;//src.map(|x| *x > min_ampl).fold(false, |a,b| a || *b);
                        // let amp:f32 = src.fold(std::f32::MIN, |a,b| a.max(*b));
                        slided.slice_mut(ndarray::s![k,..,..,..]).zip_mut_with(&src, |a,b|{*a = *b});
                    }

                    println!("Slice 2 OK");


                    let mut model_lock = self.model.lock().unwrap();
                    let outputs = model_lock.run(ort::inputs![Tensor::from_array(slided.clone()).unwrap()]).unwrap();

                    let output = outputs[self.output_layer.as_str()].try_extract_array::<f32>().unwrap().to_owned();
                    println!("ANN OK");
                    // drop(model_lock);

                    // let input:Tensor = slided.into();
                    // let found = self.model.run(tvec![input.into()]);
                    // println!("ANN OK");
                    // let found = found.unwrap().remove(0);
                    // println!("ANN unwrap OK");
                    // let found_array = found.to_array_view::<f32>().unwrap();

                    let triggered = output.map(|x| *x>self.threshold);
                    let triggered = triggered.fold_axis(Axis(1), 0u64, |a,b| {((*a)<<1) + (if *b {1} else {0})});
                    println!("ANN threshold OK {:?}",triggered);


                    triggered.iter().enumerate().for_each(|(i1,v)|{
                        if *v>0{
                            let base = i1*self.stride;
                            let active = block.slice(ndarray::s![base..base+self.size_hint.0,..,..]).map(|x| *x>min_ampl).fold(false, |a,b| a || *b);
                            if active{
                                let position = base+cut_start;
                                let tag = format!("{}(Block {} {}): {}",self.tag_prefix, i,j,v);
                                println!("{}",tag);
                                res.push(tag, position, self.size_hint.0);
                            }

                        }
                    });
                }
                else{
                    println!("Block skipped");
                }
                // let mut expanded = Array::<bool,_>::default(src_time);
                // triggered.iter().enumerate().for_each(|(i,v)|{
                //     let base = i*self.stride;
                //     //println!("BASE {}..{}",base,base+self.size_hint.0);
                //     expanded.slice_mut(ndarray::s![base..base+self.size_hint.0]).mapv_inplace(|x|{
                //         x || *v
                //     });
                // });
                // println!("ANN deconv OK");
                //
                // let needed_part = expanded.slice(ndarray::s![start-cut_start..end-cut_start]);
                // res.slice_mut(ndarray::s![..,i,j]).zip_mut_with(&needed_part, |a,b| {*a = *b;});

            }
        }

        res
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
