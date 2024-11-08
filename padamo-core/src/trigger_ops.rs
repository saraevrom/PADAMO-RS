use abi_stable::rvec;
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


#[derive(Clone,Debug)]
pub struct LazyTriggerAnd{
    source_a:LazyTrigger,
    source_b:LazyTrigger,
}

impl LazyTriggerAnd {
    pub fn new(source_a: LazyTrigger, source_b: LazyTrigger) -> Self {
        Self { source_a, source_b }
    }
}

fn flatten_trigger(x:ArrayND<bool>)->Vec<bool>{
    let mut res:Vec<bool> = Vec::with_capacity(x.shape[0]);
    res.resize(x.shape[0], false);
    for i in x.enumerate(){
        res[i[0]] |= x[&i];
    }
    res
}

impl LazyArrayOperation<ArrayND<bool>> for LazyTriggerAnd{
    fn length(&self)->usize {
        self.source_a.length()
    }
    fn calculate_overhead(&self,start:usize, end:usize)->usize {
        self.source_a.calculate_overhead(start,end)+self.source_b.calculate_overhead(start,end)
    }

    fn request_range(&self,start:usize, end:usize)->ArrayND<bool> {
        let mut result = flatten_trigger(self.source_a.request_range(start,end));
        let mut start_i:usize = 0;
        let mut current_state = false;
        for i in 0..=result.len(){
            let value = result.get(i).map(|x| *x).unwrap_or(false);
            if !current_state && value{
                current_state = true;
                start_i = i;
            }
            if current_state && !value{
                current_state = false;
                let aux_interval = flatten_trigger(self.source_b.request_range(start_i,i));
                (start_i..i).for_each(|j| result[j]|=aux_interval[j-start_i]);
            }
        }
        ArrayND{shape:rvec![result.len()], flat_data:result.into()}
    }
}
