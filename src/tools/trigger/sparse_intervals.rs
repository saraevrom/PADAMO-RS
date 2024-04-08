use serde::{Serialize,Deserialize};
use std::fmt::Display;
use padamo_api::lazy_array_operations::LazyTimeSignal;


#[derive(Clone, Copy,Debug)]
pub struct Interval{
    pub start:usize,
    pub end:usize,
}

#[derive(Clone, Copy,Debug,Serialize,Deserialize)]
pub struct UnixInterval{
    pub start:f64,
    pub end:f64,
}

#[derive(Clone, Copy,Debug)]
pub enum IntervalSubtraction{
    Empty,
    Single(Interval),
    Dual(Interval,Interval),
}

impl UnixInterval{
    pub fn new(start:f64,end:f64)->Self{
        if end<start{
            panic!("Wrong interval parameters");
        }
        Self { start, end}
    }
}

impl Interval{
    pub fn new(start:usize,end:usize)->Self{
        if end<start{
            panic!("Wrong interval parameters");
        }
        Self { start, end}
    }

    pub fn offset(&self, offset:usize)->Self{
        Self { start: self.start+offset, end: self.end+offset }
    }

    pub fn length(&self)->usize{
        self.end-self.start
    }

    pub fn is_trivial(&self)->bool{
        self.start==self.end
    }

    pub fn contains(&self, inner:&Self)->bool{
        inner.start>=self.start && inner.end<=self.end
    }

    pub fn contains_point(&self, inner:usize)->bool{
        inner>=self.start && inner<self.end
    }

    pub fn intersection(&self,other:&Self)->Option<Self>{
        if self.start>other.end || other.end>self.end{
            None
        }
        else {
            let start = usize::max(self.start,other.start);
            let end = usize::min(self.end,other.end);
            Some(Interval::new(start,end))
        }
    }

    pub fn unify(&self,other:&Self)->Option<Self>{
        if self.start>other.end || other.end>self.end{
            None
        }
        else {
            let start = usize::min(self.start,other.start);
            let end = usize::max(self.end,other.end);
            Some(Interval::new(start,end))
        }
    }

    pub fn subtract(&self, other:&Self)->IntervalSubtraction{
        if self.contains(other){
            let start_interval = Interval::new(self.start, other.start);
            let end_interval = Interval::new(other.end, self.end);
            match (start_interval.is_trivial(),end_interval.is_trivial()){
                (true,true)=> IntervalSubtraction::Empty,
                (false,true)=>IntervalSubtraction::Single(start_interval),
                (true,false)=>IntervalSubtraction::Single(end_interval),
                (false,false)=>IntervalSubtraction::Dual(Interval::new(self.start, other.start), Interval::new(other.end, self.end))
            }


        }
        else if let Some(i) = self.intersection(other){
            if i.start==self.start{
                if i.end==self.end{
                    IntervalSubtraction::Empty
                }
                else{
                    IntervalSubtraction::Single(Interval::new(i.end, self.end))
                }
            }
            else {
                IntervalSubtraction::Single(Interval::new(self.start, i.start))
            }
        }
        else {
            IntervalSubtraction::Empty
        }
    }

    pub fn to_utc_interval(&self, reference:&LazyTimeSignal)->UnixInterval {
        let start = reference.request_range(self.start,self.start+1)[0];
        let end = reference.request_range(self.end,self.end+1)[0];
        UnixInterval::new(start, end)
    }

    pub fn from_unixtime_interval(src: &UnixInterval, reference:&LazyTimeSignal)->Option<Self>{
        let start = crate::time_search::find_unixtime(reference, src.start);
        let end = crate::time_search::find_unixtime(reference, src.end);
        if end>start{
            Some(Self::new(start, end))
        }
        else{
            None
        }
    }
}


impl Display for Interval{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"[{}, {}) ({})",self.start,self.end,self.end-self.start)
    }
}

#[derive(Clone,Debug)]
pub struct IntervalStorage{
    pub container:Vec<Interval>
}


#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct UnixIntervalStorage{
    pub container:Vec<UnixInterval>
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct BinaryUnixIntervalStorage{
    pub positives:UnixIntervalStorage,
    pub negatives:UnixIntervalStorage,
}


impl IntervalStorage{
    pub fn new_full(length:usize)->Self{
        Self { container: vec![Interval::new(0,length)] }
    }

    pub fn new_empty()->Self{
        Self { container: vec![] }
    }

    pub fn get_first_available(&self)->Option<Interval>{
        if self.container.len()==0{
            None
        }
        else{
            Some(self.container[0])
        }
    }

    pub fn print_contents(&self){
        for i in self.container.iter(){
            print!("{} ",i);
        }
        println!();
    }

    fn find_close_index(&self, pos:usize)->Option<usize>{
        if self.container.is_empty(){
            return None;
        }
        let mut start:usize = 0;
        let mut end:usize = self.container.len();
        let mut middle:usize = (start+end)/2;
        if pos>self.container[end-1].start{
            return Some(end-1);
        }
        if pos<self.container[0].start{
            return Some(0);
        }
        while start != middle{
            let item = self.container[middle].start;
            if item<=pos{
                start = middle;
            }
            if item>=pos{
                end = middle;
            }
            middle = (start+end)/2;
        }
        //println!("Datetime search result. req: {}, actual: {}",unixtime, op.request_item(middle));
        Some(middle)
    }

    fn interval_in_point(&self, pos:usize)->Option<(Interval,usize)>{
        if let Some(ptr) = self.find_close_index(pos){
            if self.container[ptr].contains_point(pos){
                return Some((self.container[ptr],ptr));
            }
            if ptr>0{
                if self.container[ptr-1].contains_point(pos){
                    return Some((self.container[ptr-1],ptr-1));
                }
            }
            if ptr<self.container.len()-1{
                if self.container[ptr+1].contains_point(pos){
                    return Some((self.container[ptr+1],ptr+1));
                }
            }
            None
        }
        else{
            None
        }
    }

    fn append_interval(&mut self, interval:Interval){
        self.container.push(interval);
        let mut i = self.container.len()-1;
        while i>0{
            if self.container[i].start<self.container[i-1].start{
                (self.container[i].start,self.container[i-1].start)=(self.container[i-1].start,self.container[i].start);
                i-=1;
            }
            else {
                break;
            }
        }
    }

    fn simplify(&mut self){
        if self.container.is_empty(){
            // Empty container is perfect!
            return;
        }
        let mut i:usize = 0;
        while i < self.container.len()-1{
            if self.container[i].length()==0{
                self.container.remove(i);
            }
            else if self.container[i].end==self.container[i+1].start{
                self.container[i] = Interval::new(self.container[i].start, self.container[i+1].end);
                self.container.remove(i+1);
            }
            else{
                i+=1;
            }
        }
    }

    pub fn take_interval(&mut self,interval:Interval)->bool{
        if let Some((closest,i)) = self.interval_in_point(interval.start){
            if closest.contains(&interval){
                match closest.subtract(&interval){
                    IntervalSubtraction::Empty => {
                        self.container.remove(i);
                        return true;
                    },
                    IntervalSubtraction::Single(v) => {
                        self.container[i] = v;
                        return true;
                    },
                    IntervalSubtraction::Dual(a, b) => {
                        self.container[i] = a;
                        self.append_interval(b);
                        return true;
                    },
                }
            }
        }
        false
    }

    pub fn is_available(&self, interval:Interval)->bool{
        if let Some((closest,i)) = self.interval_in_point(interval.start){
            return closest.contains(&interval);
        }
        false
    }

    pub fn insert_interval(&mut self,interval:Interval)->bool{
        if let Some(_) = self.interval_in_point(interval.start){
            return false;
        }
        if let Some(_) = self.interval_in_point(interval.end){
            return false;
        }
        else{
            self.append_interval(interval);
            self.simplify();
            return true;
        }
    }

    pub fn to_unixtime_storage(&self,reference:&LazyTimeSignal)->UnixIntervalStorage{
        let unixtimes = self.container.iter().map(|x| x.to_utc_interval(reference)).collect();
        UnixIntervalStorage { container: unixtimes }
    }

    pub fn from_unixtime_storage(src:&UnixIntervalStorage,reference:&LazyTimeSignal)->Self{
        let intervals = src.container.iter().map(|x| Interval::from_unixtime_interval(x, reference)).filter(|x| x.is_some()).map(|x| x.unwrap()).collect();
        //println!("{:?}",intervals);
        let mut res = Self { container: intervals };
        res.simplify();
        res
    }
}


pub fn split_intervals(trigger_result:&Vec<bool>)->(Vec<Interval>,Vec<Interval>){
    let mut positives = Vec::new();
    let mut negatives = Vec::new();

    let mut interval_start:usize = 0;
    let mut current_end:usize = 0;
    let mut current = trigger_result[0];
    for (i,x) in trigger_result.iter().enumerate(){
        current_end = i;
        if *x != current{
            if current{
                positives.push(Interval::new(interval_start,current_end));
            }
            else{
                negatives.push(Interval::new(interval_start,current_end));
            }
            current = *x;
            interval_start = current_end;
        }
    }
    current_end = trigger_result.len();

    // Last interval
    if current{
        positives.push(Interval::new(interval_start,current_end));
    }
    else{
        negatives.push(Interval::new(interval_start,current_end));
    }

    (positives,negatives)
}
