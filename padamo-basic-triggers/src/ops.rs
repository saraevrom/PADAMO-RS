use padamo_api::lazy_array_operations::{LazyArrayOperation, LazyDetectorSignal};
use padamo_api::trigger_operations::SparseTagArray;
use rayon::iter::ParallelIterator;
use medians::Medianf64;

#[derive(Clone,Debug)]
pub struct LazyPixelThresholdTrigger{
    src: LazyDetectorSignal,
    threshold:f64,
}


impl LazyPixelThresholdTrigger{
    pub fn new(src: LazyDetectorSignal, threshold:f64)->Self{
        Self{src,threshold}
    }
}


fn threshold_trigger<F1, F2,S>(src:&LazyDetectorSignal, start:usize, end:usize, mut f:F1, threshold:f64, tag:S, mut fmt:F2)->SparseTagArray
where
    F1: FnMut(&[f64])->f64 + Copy + Clone,
    F2: FnMut(f64,&f64)->f64 + Copy + Clone,
    S: Into<String>
{
    let len = src.length();
    let tag_prefix = tag.into();
    if len==0{
        return SparseTagArray::new(); // Found no negative values. Therefore no events can be reported. :(
    }

    //Firstly, let's check if the start is a continuation of another event.
    let mut real_start = start;
    if real_start>0{
        let initial_arr = src.request_range(start-1,end);
        let initial_ampls = initial_arr.apply_on_frames(f);
        if initial_ampls[0]>threshold{
            // This is a continuation of the event.
            while real_start<end && initial_ampls[real_start-start+1]>threshold{
                real_start+=1;
            }
        }
    }

    if real_start>=end{
        return SparseTagArray::new(); // Found no negative values. Therefore no events can be reported. :(
    }

    let mut real_end = end;
    //Second thing to consider:let's correct the end the same way
    let mut last_frame = src.request_range(real_end-1,real_end).take_frame().unwrap();
    while real_end<len && f(&last_frame.flat_data)>threshold{
        real_end+= 1;
        last_frame = src.request_range(real_end-1,real_end).take_frame().unwrap();
    }

    //println!("Corrected interval: {} {}", real_start, real_end);
    let final_source = src.request_range(real_start,real_end).apply_on_frames(f);
    let mut res = SparseTagArray::new();
    let mut current_state = None;
    for (i,x) in final_source.iter().enumerate(){
        let trig = *x>threshold;
        //println!("TRIG {} {:?} {}", x, current_state, trig);
        match (current_state, trig){
            (Some((start,amp)), false)=>{
                let duration = i-start;
                current_state = None;
                res.push(format!("{} {}",tag_prefix, amp), start+real_start, duration);
            }
            (Some((start,amp)), true)=>{
                current_state = Some((start,fmt(amp,&x)));
            }
            (None, true)=>{
                //println!("Start captured: {}", i);
                current_state = Some((i,*x));
            }
            _ => (),
        }
    }
    if let Some((start,amp)) = current_state{
        let duration = real_end-start;
        res.push(format!("{} {}",tag_prefix, amp), start, duration);
    }
    res
}

impl LazyArrayOperation<SparseTagArray> for LazyPixelThresholdTrigger{
    fn length(&self) -> usize {
        self.src.length()
    }
    fn calculate_overhead(&self, start: usize, end: usize) -> usize{
        self.src.calculate_overhead(start,end)
    }
    fn request_range(&self, start: usize, end: usize) -> SparseTagArray {
        threshold_trigger(&self.src, start, end,
                          |x| x.iter().fold(std::f64::MIN, |a,b| a.max(*b)),
                          self.threshold, "Peak:", |a,b| a.max(*b))
        // let base: ndim_array::ArrayND<f64> = self.src.request_range(start,end);
        // let shape = base.shape;
        // let flat_data = base.flat_data.into_vec();
        // let flat_data:Vec<bool> = flat_data.par_iter().map(|x| *x>self.threshold).collect();
        // ndim_array::ArrayND{flat_data:flat_data.into(),shape}
    }
}

#[derive(Clone,Debug)]
pub struct LazyLCThresholdTrigger{
    src: LazyDetectorSignal,
    threshold:f64,
}


impl LazyLCThresholdTrigger{
    pub fn new(src: LazyDetectorSignal, threshold:f64)->Self{
        Self{src,threshold}
    }
}

impl LazyArrayOperation<SparseTagArray> for LazyLCThresholdTrigger{
    fn length(&self) -> usize {
        self.src.length()
    }
    fn calculate_overhead(&self, start: usize, end: usize) -> usize{
        self.src.calculate_overhead(start,end)
    }
    fn request_range(&self, start: usize, end: usize) -> SparseTagArray {
        threshold_trigger(&self.src, start, end,
                          |x| x.iter().fold(0.0, |a,b| a + *b),
                          self.threshold, "Peak:", |a,b| a.max(*b))
        // let base: ndim_array::ArrayND<f64> = self.src.request_range(start,end);
        // let mut lc:Vec<f64> = Vec::with_capacity(end-start);
        // lc.resize(end-start, 0.0);
        //
        // for index in base.enumerate(){
        //     lc[index[0]] += base[&index];
        // }
        //
        // let flat_data:RVec<bool> = lc.iter().map(|x| *x>self.threshold).collect();
        // let shape = vec![end-start];
        // //let flat_data:RVec<bool> = base.flat_data.iter().map(|x| *x>self.threshold).collect();
        // ndim_array::ArrayND{flat_data,shape:shape.into()}
    }
}


#[derive(Clone,Debug)]
pub struct LazyMedianTrigger{
    src: LazyDetectorSignal,
    threshold:f64,
}


impl LazyMedianTrigger{
    pub fn new(src: LazyDetectorSignal, threshold:f64)->Self{
        Self{src,threshold}
    }
}


impl LazyArrayOperation<SparseTagArray> for LazyMedianTrigger{
    fn length(&self,) -> usize where {
        self.src.length()
    }
    fn calculate_overhead(&self,start:usize,end:usize,) -> usize where {
        self.src.calculate_overhead(start,end)
    }
    fn request_range(&self,start:usize,end:usize,) -> SparseTagArray {
        threshold_trigger(&self.src, start, end,
                          |x| x.medf_checked().expect("Median computation failed"),
                          self.threshold, "Peak:", |a,b| a.max(*b))

        // let workon = self.src.request_range(start,end);
        // let sublen = end-start;
        // let indices=  workon.shape.len();
        // let workon = workon.to_ndarray();
        // let thresh = self.threshold;
        //
        // let res = Arc::new(Mutex::new(ArrayND::new(vec![sublen],false)));
        // let passed = res.clone();
        // (0..sublen).par_bridge().for_each(move |i|{
        //     let slices:Vec<SliceInfoElem> = (0..indices).map(
        //         |j| if j==0{
        //             SliceInfoElem::Index(i as isize)
        //         }
        //         else{
        //             SliceInfoElem::Slice { start: 0, end: None, step: 1 }
        //         }).collect();
        //
        //     let slicing = SliceInfo::<&[SliceInfoElem],ndarray::Dim<ndarray::IxDynImpl>,ndarray::Dim<ndarray::IxDynImpl>>::try_from(slices.as_slice()).expect("Slicing error");
        //
        //     let part = workon.slice(slicing);
        //     let (vector,off_opt) = part.to_owned().into_raw_vec_and_offset();
        //     let mut vector = vector;
        //     if let Some(off) = off_opt{
        //         vector.drain(..off);
        //     }
        //     let vector = vector;
        //
        //     if let Ok(v) = vector.medf_checked(){
        //         passed.lock().unwrap().flat_data[i] = v>thresh;
        //         //res.flat_data[i] = v>thresh;
        //     }
        // });
        //
        // let lock = Arc::try_unwrap(res).expect("Lock still has multiple owners");
        // lock.into_inner().expect("Mutex cannot be locked")
    }
}
