use std::sync::Arc;
use padamo_api::lazy_array_operations::cache::Cache;
use padamo_api::lazy_array_operations::ArrayND;
use padamo_api::lazy_array_operations::LazyArrayOperation;
use padamo_api::lazy_array_operations::LazyDetectorSignal;
use padamo_api::function_operator::DoubleFunctionOperatorBox;
use crate::stft::STFTConverter;

#[derive(Clone,Debug)]
pub struct  FilterOp{
    source:LazyDetectorSignal,
    modifier:DoubleFunctionOperatorBox,
    stft:STFTConverter,
    sample_rate: f64,
}

impl FilterOp {
    pub fn new(source: LazyDetectorSignal, modifier: DoubleFunctionOperatorBox, window:usize, sample_rate: f64) -> Self {
        Self { source, modifier, stft:STFTConverter::new(window), sample_rate }
    }
}




impl LazyArrayOperation<ArrayND<f64>> for FilterOp{
    fn length(&self,) -> usize{
        let src_len = self.source.length();
        let step = self.stft.window/2;
        let windows_amount = (src_len-self.stft.window)/step + 1;
        windows_amount*step+self.stft.window
    }

    fn calculate_overhead(&self,start:usize,end:usize,) -> usize where {
        self.source.calculate_overhead(start,end)*self.stft.window
    }

    fn request_range(&self,start:usize,end:usize,) -> ArrayND<f64> {
        // let left_offset = self.stft.window;
        let length = self.length();
        // let right_offset = self.stft.window;
        // let left_skip = if start<left_offset {start} else {left_offset};
        // let right_skip = if end+right_offset>length {length-end} else {right_offset};
        //
        // let actual_start = start-left_skip;
        // let actual_end = end+right_skip;
        let window_step = self.stft.window/2;
        let window_index_of_start = start/window_step;
        let window_index_of_end = end/window_step;

        let actual_start = if window_index_of_start*window_step>=self.stft.window {window_index_of_start*window_step-self.stft.window} else {0};

        //let last_window = (self.length()-self.stft.window)/window_step;
        let actual_end = (window_index_of_end*window_step+window_step*3).min(length);

        let left_skip = start-actual_start;
        let right_skip = actual_end - end;

        let signal:ArrayND<f64> = self.source.request_range(actual_start,actual_end);
        let mut signal = self.stft.filter_arrays(signal, self.modifier.clone(), self.sample_rate);
        signal = signal.cut_front(left_skip);
        signal = signal.cut_end(right_skip);
        signal
    }
}
