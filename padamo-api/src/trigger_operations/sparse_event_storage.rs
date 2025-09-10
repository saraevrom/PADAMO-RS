use abi_stable::{std_types::{RString,RVec}, StableAbi};
use crate::lazy_array_operations::merge::Merge;

#[repr(C)]
#[derive(StableAbi,Clone)]
pub struct SparseTag{
    pub tag:RString,
    pub position:usize,
    pub duration:usize,
}

impl SparseTag {
    fn new(tag: RString, position: usize, duration:usize) -> Self {
        Self { tag, position, duration }
    }
}

#[repr(C)]
#[derive(StableAbi,Clone)]
pub struct SparseTagArray{
    tags:RVec<SparseTag>
}

fn check_first(arr:&[SparseTag], lower:usize)->bool{
    if let Some(x) = arr.first(){
        x.position<lower
    }
    else{
        false
    }
}

fn check_last(arr:&[SparseTag], upper:usize)->bool{
    if let Some(x) = arr.last(){
        x.position>=upper
    }
    else{
        false
    }
}

impl SparseTagArray{
    pub fn new()->Self{
        Self { tags: RVec::new() }
    }

    pub fn with_capacity(capacity:usize)->Self{
        Self { tags: RVec::with_capacity(capacity) }
    }

    pub fn len(&self)->usize{
        self.tags.len()
    }

    pub fn push<T:Into<RString>>(&mut self, tag:T, position:usize, duration:usize){
        let new_item = SparseTag::new(tag.into(), position, duration);
        self.tags.push(new_item);
        for i in (1..self.tags.len()).rev(){
            if self.tags[i-1].position>self.tags[i].position{
                self.tags.swap(i-1, i);
            }
            else{
                break;
            }
        }
    }

    pub fn retain_interval(mut self, start:usize, end:usize)->Self{
        //Filter start
        while check_first(&self.tags, start){
            self.tags.remove(0);
        }

        //Filter end
        while check_last(&self.tags, end){
            self.tags.pop();
        }
        self
    }

    pub fn view_tags<'a>(&'a self) -> Vec<&'a str>{
        self.tags.iter().map(|x| x.tag.as_str()).collect()
    }
}

impl Merge for SparseTagArray{
    fn merge(mut self,mut other:Self)->Self {
        let mut res = Self::with_capacity(self.len()+other.len());
        // Merging arrays using merge-sort-like stuff
        //Step 1: Merge magic
        while !(self.tags.is_empty() || other.tags.is_empty()){
            let choice = if self.tags[0].position<=other.tags[0].position{
                &mut self
            }
            else {
                &mut other
            };
            res.tags.extend(choice.tags.drain(0..1));
        }

        //Step 2: add rest
        res.tags.extend(self.tags.drain(..));
        res.tags.extend(other.tags.drain(..));

        res
    }
}

#[cfg(test)]
mod tests{
    use crate::lazy_array_operations::merge::Merge;

    use super::SparseTagArray;

    #[test]
    fn test_creation(){
        let mut events = SparseTagArray::new();
        events.push("A", 0, 10);
        events.push("C", 2, 10);
        events.push("B", 1, 10);
        assert_eq!(events.view_tags(), vec!["A", "B", "C"])
    }

    #[test]
    fn test_stability(){
        let mut events = SparseTagArray::new();
        events.push("A", 0, 10);
        events.push("C", 2, 10);
        events.push("B", 1, 10);
        events.push("D", 2, 10);
        events.push("E", 3, 10);


        assert_eq!(events.view_tags(), vec!["A", "B", "C", "D", "E"])
    }

    #[test]
    fn test_merging(){
        let mut events1 = SparseTagArray::new();
        events1.push("A", 0, 10);
        events1.push("C", 2, 10);
        events1.push("B", 1, 10);

        let mut events2 = SparseTagArray::new();
        events2.push("D", 2, 10); // Shares same position as "C". The 'merge' will prioritize left parameter during call.
        events2.push("E", 4, 10);
        events2.push("F", 5, 10);
        let events = events1.merge(events2);
        assert_eq!(events.view_tags(), vec!["A", "B", "C", "D", "E", "F"])
    }

    #[test]
    fn test_retaining(){
        let mut events = SparseTagArray::new();
        events.push("A", 0, 10);
        events.push("C", 2, 10);
        events.push("B", 1, 10);

        events.push("D", 3, 10);
        events.push("E", 4, 10);
        events.push("F", 5, 10);

        events = events.retain_interval(2, 5);
        assert_eq!(events.view_tags(), vec!["C", "D", "E"])
    }
}
