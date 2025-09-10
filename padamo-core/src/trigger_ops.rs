use abi_stable::rvec;
use padamo_api::lazy_array_operations::cache::Cache;
use padamo_api::lazy_array_operations::ArrayND;
use padamo_api::lazy_array_operations::{LazyArrayOperation,LazyTrigger};
use padamo_api::trigger_operations::SparseTagArray;
use padamo_api::lazy_array_operations::merge::Merge;

#[derive(Clone,Debug)]
pub struct LazyTriggerMerge{
    source1:LazyTrigger,
    source2:LazyTrigger,
    //src_shape:Vec<usize>
}

impl LazyTriggerMerge {
    pub fn new(source1: LazyTrigger, source2: LazyTrigger) -> Self {
        Self { source1, source2 }
    }
}

impl LazyArrayOperation<SparseTagArray> for LazyTriggerMerge{
    #[allow(clippy::let_and_return)]
    fn length(&self,) -> usize where {
        self.source1.length()
    }

    #[allow(clippy::let_and_return)]
    fn request_range(&self,start:usize,end:usize,) -> SparseTagArray where {
        let a = self.source1.request_range(start,end);
        let b = self.source2.request_range(start,end);
        a.merge(b)
    }

    fn calculate_overhead(&self,start:usize,end:usize,) -> usize where {
        let a = self.source1.calculate_overhead(start,end);
        let b = self.source2.calculate_overhead(start,end);
        a.max(b)
    }

}

#[derive(Clone,Debug)]
pub struct LazyTriggerRemoveOverlap{
    source:LazyTrigger,
    template:String,
    //src_shape:Vec<usize>
}

impl LazyTriggerRemoveOverlap {
    pub fn new(source: LazyTrigger, template:String) -> Self {
        Self { source, template }
    }
}

impl LazyArrayOperation<SparseTagArray> for LazyTriggerRemoveOverlap{
    #[allow(clippy::let_and_return)]
    fn length(&self,) -> usize where {
        self.source.length()
    }

    #[allow(clippy::let_and_return)]
    fn request_range(&self,start:usize,end:usize,) -> SparseTagArray where {
        let mut res = self.source.request_range(start,end);
        let mut deduplicating = true;
        let mut extending = true;
        if res.tags.is_empty(){
            return SparseTagArray::new();
        }
        while deduplicating || extending{
            deduplicating = false;
            let mut i:usize = 0;
            while i+1<res.tags.len(){
                if res.tags[i].position+res.tags[i].duration >= res.tags[i+1].position{
                    let new_end = res.tags[i+1].position+res.tags[i+1].duration;
                    let new_length = new_end - res.tags[i].position;
                    res.tags[i+1].position = res.tags[i].position;
                    res.tags[i+1].duration = new_length;
                    res.tags.remove(i);
                    deduplicating = true;
                }
                else{
                    i += 1;
                }
            }

            extending = false;
            let mut is_running = true;
            while is_running{
                let addenum = res.tags.last().unwrap();
                let sub_start = addenum.position;
                let sub_end = sub_start+addenum.duration;
                let mut addenum_tags = self.source.request_range(sub_start, sub_end);
                while !addenum_tags.tags.is_empty() && addenum_tags.tags[0].position==sub_start{
                    addenum_tags.tags.remove(0);
                }

                is_running = false;
                if !addenum_tags.tags.is_empty(){
                    extending = true;
                    is_running = true;
                }
                res = res.merge(addenum_tags);
            }

        }

        res
    }

    fn calculate_overhead(&self,start:usize,end:usize,) -> usize where {
        self.source.calculate_overhead(start,end)
    }

}
