use super::{LazyArrayOperation,LazyArrayOperationBox};
use std::{fmt::Debug, sync::{Arc, Mutex}};


#[derive(Debug,Clone)]
pub struct CacheContent<T>
where
    T:Cache
{
    pub cache:Option<T>,
    pub last_bounds:(usize,usize)
}

impl<T:Cache> CacheContent<T>{
    pub fn new()->Self{
        Self{cache:None, last_bounds:(0,0)}
    }
}


#[derive(Debug,Clone)]
pub struct LazyArrayOperationLocalCache<T>
where
    T:Cache
{
    src:LazyArrayOperationBox<T>,
    cache:Arc<Mutex<CacheContent<T>>>,
    // last_bounds:Arc<Mutex<(usize,usize)>>,
}

impl<T:Cache> LazyArrayOperationLocalCache<T>{
    pub fn new(src:LazyArrayOperationBox<T>)->Self{
        Self{src, cache:Arc::new(Mutex::new(CacheContent::new()))}
    }
}

impl<T:Cache+Clone+Debug+Send+Sync> LazyArrayOperation<T> for LazyArrayOperationLocalCache<T>{
    #[allow(clippy::let_and_return)]
    fn length(&self,) -> usize where {
        self.src.length()
    }

    fn calculate_overhead(&self,start:usize,end:usize,) -> usize where {
        (end-start)+self.src.calculate_overhead(start,end)
    }

    #[allow(clippy::let_and_return)]
    fn request_range(&self,start:usize,end:usize,) -> T where {
        let mut cache_mut = self.cache.lock().unwrap();


        //let mut last_bounds = self.last_bounds.borrow_mut();
        if let Some(v) = cache_mut.cache.take(){
            let (old_start,old_end) = cache_mut.last_bounds;
            cache_mut.last_bounds = (start,end);
            let mut res = v;
            //println!("CACHE REQUEST {}-{} ({}-{})",start,end,old_start,old_end);

            if start>=old_end || end<=old_start{
                let data = self.src.request_range(start,end);
                cache_mut.cache = Some(data.clone());
                cache_mut.last_bounds = (start,end);

                data
            }
            else{

                if start>=old_start{
                    //println!("CUTLEFT {}",start-old_start);
                    res = res.cut_front(start-old_start);
                }
                else{
                    //println!("ADDLEFT {}-{}",start,old_start);
                    let part = self.src.request_range(start,old_start);
                    res = res.prepend(part);
                }

                if end<=old_end{
                    //println!("CUTRIGHT {}",old_end-end);
                    res = res.cut_end(old_end-end);
                }
                else{
                    //println!("ADDRIGHT {}-{}",old_end,end);
                    let part = self.src.request_range(old_end,end);
                    res = res.append(part);
                }

                res
            }
        }
        else {
            let data = self.src.request_range(start,end);
            cache_mut.cache = Some(data.clone());
            cache_mut.last_bounds = (start,end);
            //println!("CACHE REQUEST {}-{}",start,end);
            data
        }
    }
}



pub trait Cache{
    fn cut_front(self,count:usize)->Self;
    fn cut_end(self,count:usize)->Self;
    fn prepend(self,data:Self)->Self;
    fn append(self,data:Self)->Self;
}

impl<T> Cache for Vec<T>{
    fn cut_front(self,count:usize)->Self {
        let mut res = self;
        res.drain(..count);
        res
    }

    fn cut_end(self,count:usize)->Self {
        let dr = self.len()-count;
        let mut res = self;
        res.drain(dr..);
        res
    }

    fn prepend(self,data:Self)->Self {
        let mut res = data;
        res.extend(self);
        res
    }

    fn append(self,data:Self)->Self {
        let mut res = self;
        res.extend(data);
        res
    }
}

