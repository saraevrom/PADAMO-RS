
use std::fmt::Debug;

use super::LazyArrayOperationBox;
use super::LazyArrayOperation;

#[derive(Clone,Debug,thiserror::Error)]
pub enum CutError{
    #[error("Start index {0} is out of bounds")]
    StartOutOfBounds(usize),

    #[error("End index {0} is out of bounds")]
    EndOutOfBounds(usize),

    #[error("Start index {0} must be smaller than end index {1}")]
    StartEndMessedBounds(usize,usize),

}

#[derive(Debug,Clone)]
pub struct LazyArrayOperationCutter<T>
where
    T:Clone+Debug
{
    pub source:LazyArrayOperationBox<T>,
    pub start:usize,
    pub end:usize
}

impl<T> LazyArrayOperationCutter<T>
where
    T:Clone+Debug
{
    pub fn new(source:LazyArrayOperationBox<T>,start:usize, end:usize)->Result<Self,CutError>{
        let l = source.length();
        if start>l{
            Err(CutError::StartOutOfBounds(start))
        }
        else if end>l{
            Err(CutError::EndOutOfBounds(end))
        }
        else if start>end{
            Err(CutError::StartEndMessedBounds(start, end))
        }
        else{
            Ok(
                Self{
                    source,start,end
                }
            )
        }
    }
}

impl<T> LazyArrayOperation<T> for LazyArrayOperationCutter<T>
where
    T:Clone+Debug
{
    #[allow(clippy::let_and_return)]
    fn calculate_overhead(&self,start:usize,end:usize,) -> usize where {
        self.source.calculate_overhead(start+self.start,end+self.start)
    }

    #[allow(clippy::let_and_return)]
    fn length(&self,) -> usize where {
        self.end-self.start
    }

    #[allow(clippy::let_and_return)]
    fn request_range(&self,start:usize,end:usize,) -> T{
        self.source.request_range(self.start+start,self.start+end)
    }
}
