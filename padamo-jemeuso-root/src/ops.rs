
use abi_stable::{rvec, std_types::RVec};
use padamo_api::lazy_array_operations::{merge::Merge, ArrayND, LazyArrayOperation};
use oxyroot::RootFile;
pub use pseudotime::ops::AddTime;


#[derive(Clone,Debug)]
pub struct LazyROOTSpatialReader{
    pub file_path:String,
    pub tree:String,
    pub branch:String,
}

impl LazyROOTSpatialReader{
    pub fn new(file_path: String, tree: String, branch: String) -> Self {
        Self { file_path, tree, branch }
    }
}



impl LazyArrayOperation<ArrayND<f64>> for LazyROOTSpatialReader{
    fn length(&self,) -> usize where {
        let mut rootfile = if let Ok(v) = RootFile::open(&self.file_path) {v} else {return 0;};
        let tree = if let Ok(v) = rootfile.get_tree(&self.tree) {v} else {return 0;};
        let branch = if let Some(v) = tree.branch(&self.branch) {v} else {return 0;};
        let len = branch.entries() as usize;
        // println!("LENGTH OK {}", len);
        len
    }

    fn calculate_overhead(&self,start:usize,end:usize,) -> usize where {
        end-start
    }

    fn request_range(&self,start:usize,end:usize,) -> ArrayND<f64>where {
        let mut rootfile = if let Ok(v) = RootFile::open(&self.file_path) {v} else {return ArrayND::new(vec![1,1], 0.0);};
        let tree = if let Ok(v) = rootfile.get_tree(&self.tree) {v} else {return ArrayND::new(vec![1,1], 0.0);};
        let branch = if let Some(v) = tree.branch(&self.branch) {v} else {return ArrayND::new(vec![1,1], 0.0);};
        let branch_iter = branch.as_iter_manual::<crate::scalable_array::NDArrayRootWrapper>().skip(start).take(end-start);
        // println!("BRANCH OK");
        let frames:Vec<ArrayND<f64>> = branch_iter.map(|x| {
            // println!("data {:?}", x.data);
            let mut data:ArrayND<f64> = x.data.into();
            data.shape.insert(0, 1);
            // println!("data {:?}", data.shape);
            data
        }).collect();
        if frames.len()==0{
            return ArrayND::new(vec![1,1], 0.0);
        }
        else{
            let mut shape = frames.first().unwrap().shape.clone();
            shape[0] = 0;
            let res = frames.iter().fold(ArrayND::new(shape.into(), 0.0), |a,b| a.merge(b.clone()));
            res
        }

    }
}




#[derive(Clone,Debug)]
pub struct LazyROOTTemporalReader{
    pub file_path:String,
    pub tree:String,
    pub branch:String,
}

impl LazyROOTTemporalReader{
    pub fn new(file_path: String, tree: String, branch: String) -> Self {
        Self { file_path, tree, branch }
    }
}



impl LazyArrayOperation<RVec<f64>> for LazyROOTTemporalReader{
    fn length(&self,) -> usize where {
        let mut rootfile = if let Ok(v) = RootFile::open(&self.file_path) {v} else {return 0;};
        let tree = if let Ok(v) = rootfile.get_tree(&self.tree) {v} else {return 0;};
        let branch = if let Some(v) = tree.branch(&self.branch) {v} else {return 0;};
        let len = branch.entries() as usize;
        // println!("LENGTH OK {}", len);
        len
    }

    fn calculate_overhead(&self,start:usize,end:usize,) -> usize{
        end-start
    }

    fn request_range(&self,start:usize,end:usize,) -> RVec<f64>{
        let mut rootfile = if let Ok(v) = RootFile::open(&self.file_path) {v} else {return rvec![];};
        let tree = if let Ok(v) = rootfile.get_tree(&self.tree) {v} else {return rvec![];};
        let branch = if let Some(v) = tree.branch(&self.branch) {v} else {return rvec![];};
        let branch_iter = if let Ok(v) = branch.as_iter::<f64>() {v} else {return rvec![];};
        let branch_iter = branch_iter.skip(start).take(end-start);
        // println!("BRANCH OK");
        let ticks:Vec<f64> = branch_iter.collect();
        ticks.into()


    }
}
