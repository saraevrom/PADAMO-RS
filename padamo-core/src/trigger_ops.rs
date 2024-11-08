use padamo_api::lazy_array_operations::cache::Cache;
use padamo_api::lazy_array_operations::ArrayND;
use padamo_api::lazy_array_operations::{LazyArrayOperation,LazyTrigger};

#[derive(Clone,Debug)]
pub struct LazyTriggerExpand{
    source:LazyTrigger,
    expansion:usize,
    //src_shape:Vec<usize>
}

impl LazyTriggerExpand{
    pub fn new(source: LazyTrigger, expansion:usize) -> Self {
        Self { source, expansion}
    }

}

fn convolve_bools(source:ArrayND<bool>,expansion:usize)->ArrayND<bool>{
    let mut res = ArrayND::new(source.shape.clone().into(), false);
    let full_len = source.shape[0];
    for index in res.enumerate(){
        let start_i = if index[0]<expansion{
            0
        }
        else{
            index[0]-expansion
        };

        let end_i = if index[0]>=(full_len-expansion){
            full_len
        }
        else{
            index[0]+expansion
        };

        for i in start_i..end_i{
            let mut src_index = index.clone();
            src_index[0] = i;
            res[&index] = res[&index] || source[&src_index];
        }

    }
    res
}


impl LazyArrayOperation<ArrayND<bool>> for LazyTriggerExpand{
    #[allow(clippy::let_and_return)]
    fn length(&self) -> usize where {
        self.source.length()
    }

    #[allow(clippy::let_and_return)]
    fn request_range(&self,start:usize,end:usize) -> ArrayND<bool> {

        let full_len = self.length();
        let expansion = self.expansion;


        if full_len<2*expansion+1{
            let data = self.source.request_range(0,full_len);
            let res_data = convolve_bools(data, expansion);
            return res_data.cut_end(full_len-(end-start));
        }

        let (src_start,cut_start) = if start<expansion{
            (0,start)
        }
        else{
            (start-expansion,expansion)
        };

        let (src_end,cut_end) = if end>=full_len-expansion{
            (full_len,full_len-end)
        }
        else{
            (end+expansion,expansion)
        };


        let src_data = self.source.request_range(src_start,src_end);
        let convolved = convolve_bools(src_data, expansion);
        let convolved = convolved.cut_front(cut_start);
        let convolved = convolved.cut_end(cut_end);

        assert_eq!(convolved.shape[0],end-start);
        convolved
    }

    #[allow(clippy::let_and_return)]
    fn calculate_overhead(&self,start:usize,end:usize) -> usize where {
        end-start+self.expansion
    }
}


#[derive(Clone,Debug)]
pub struct LazyTriggerNegate{
    source:LazyTrigger,
    //src_shape:Vec<usize>
}

impl LazyTriggerNegate{
    pub fn new(source: LazyTrigger) -> Self {
        Self { source}
    }
}

impl LazyArrayOperation<ArrayND<bool>> for LazyTriggerNegate{
    fn length(&self)->usize {
        self.source.length()
    }

    fn calculate_overhead(&self,start:usize, end:usize)->usize {
        self.source.calculate_overhead(start,end)
    }

    fn request_range(&self,start:usize, end:usize)->ArrayND<bool> {
        let mut pre = self.source.request_range(start,end);
        pre.flat_data.iter_mut().for_each(|x| {*x = !*x;});
        pre
    }
}
