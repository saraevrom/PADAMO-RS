use super::roller::SwapperRollerArray;
use atomic_refcell::AtomicRefCell;
use std::sync::Arc;

type Comparator = &'static dyn Fn(f64,f64)->bool;

pub struct HeapContainer{
    roller: Arc<AtomicRefCell<SwapperRollerArray>>,
    offset:usize,
    upper_fn:Comparator,

    //coefficient:usize, // = 2 we need only 2 subarrays
}

pub fn gt_compare(a:f64,b:f64)->bool{
    a>b
}

pub fn lt_compare(a:f64,b:f64)->bool{
    a<b
}




fn get_parent_index(child:usize)->Option<usize>{
    if child==0{
        None
    }
    else{
        Some((child-1)/2)
    }
}

impl HeapContainer{
    pub fn new(roller:Arc<AtomicRefCell<SwapperRollerArray>>,offset:usize, upper_fn:Comparator)->Self{
        Self{roller, offset,upper_fn}
    }

    fn remap_index(&self, i:usize)->usize{
        i*2+self.offset
    }

    fn get_best_child_index(&self,parent:usize)->Option<usize>{
        let base = 2*parent;
        let i1 = base+1;
        let i2 = base+2;
        let l = self.length();

        if i1>=l && i2>=l{
            return None;
        }

        if i1<l && i2>=l{
            return Some(i1);
        }

        if i1>=l && i2<l{
            return Some(i2);
        }

        if self.compare_indices(i1,i2){
            Some(i1)
        }
        else{
            Some(i2)
        }
    }

    fn compare_indices(&self,i:usize,j:usize)->bool{
        (self.upper_fn)(self.get(i),self.get(j))
    }

    pub fn inner_index(&self,i:usize)->Option<usize>{
        if i%2==self.offset{
            Some((i-self.offset)/2)
        }
        else{
            None
        }
    }

    pub fn length(&self)->usize{
        let src_len = self.roller.borrow().length();
        let available_nonscarce_len = src_len - self.offset;
        let half = available_nonscarce_len/2;
        if available_nonscarce_len%2==0{
            half
        }
        else{
            half+1
        }
    }

    pub fn swap(&mut self, i:usize,j:usize){
        let i = self.remap_index(i);
        let j = self.remap_index(j);
        self.roller.borrow_mut().swap(i,j);
    }
    pub fn get(&self, i:usize)->f64{
        let i = self.remap_index(i);
        self.roller.borrow().get(i)
    }

    fn float_element(&mut self, index:usize)->bool{
        let mut i = index;
        while let Some(parent) = get_parent_index(i){
            if self.compare_indices(i,parent){
                self.swap(i,parent);
                i = parent;
            }
            else{
                return false;
            }
        }
        true
    }

    fn sink_element(&mut self, index:usize){
        let mut i = index;
        while let Some(c) = self.get_best_child_index(i){
            if self.compare_indices(c,i){
                self.swap(i,c);
                i = c;
            }
            else{
                return
            }
        }
    }

    pub fn move_element(&mut self, inner:usize)->bool{
        //let me = self.get(inner).unwrap();
        //let c = self.get(self.get_best_child_index(inner).unwrap());
        if let Some(c_id) = self.get_best_child_index(inner){
            if self.compare_indices(c_id,inner){
                //println!("Sinking element {}", inner);
                self.sink_element(inner);
                return false;
            }
        }

        if let Some(p) = get_parent_index(inner){
            //let pval = self.get(p).unwrap();
            if self.compare_indices(inner,p){
                return self.float_element(inner);
            }
            else{
                return false;
            }
        }
        else{
            return true;
        }
    }

    pub fn print_heap(&self){
        for i in 0..self.length(){
            print!("{} ",self.get(i));
        }
        println!()
    }
}


pub struct DoubleHeap{
    container:Arc<AtomicRefCell<SwapperRollerArray>>,
    lower:HeapContainer,
    upper:HeapContainer,
}

impl DoubleHeap{
    pub fn new(window_size:usize)->Self{
        let container = Arc::new(AtomicRefCell::new(SwapperRollerArray::new(window_size)));
        //heap of elements with size lower than median. Max heap
        let lower = HeapContainer::new(container.clone(),0,&gt_compare);
        //heap of elements with size larger than median. Min heap
        let upper = HeapContainer::new(container.clone(),1,&lt_compare);
        Self{lower,upper,container}
    }

    pub fn median(&self)->f64{
        let a = self.lower.length();
        let b = self.upper.length();
        if a==0 && b==0{
            panic!("Heaps are empty");
        }

        if a>b{
            self.lower.get(0)
        }
        else if b>a{
            self.upper.get(0)
        }
        else{
            (self.lower.get(0)+self.upper.get(0))*0.5
        }
    }


    fn move_element_nointer(&mut self, index:usize)->Option<usize>{
        if let Some(i) = self.upper.inner_index(index){
            //println!("Element {} in upper heap",i);
            if self.upper.move_element(i){

                Some(0)
            }
            else{
                None
            }
        }
        else if let Some(i) = self.lower.inner_index(index){
            //println!("Element {} in lower heap",i);
            if self.lower.move_element(i){
                Some(1)
            }
            else{
                None
            }
        }
        else{
            panic!("Unexpected index location");
        }
    }

    fn move_element(&mut self, index:usize){
        if let Some(i) = self.move_element_nointer(index){
            //println!("Element reached top");
            //println!("Comparing {} and {}",self.lower.get(0),self.upper.get(0));
            if self.lower.get(0)>self.upper.get(0){
                //println!("SWAP");
                self.container.borrow_mut().swap(0,1);
                self.move_element_nointer(i);
            }
        }
    }

    pub fn roll(&mut self, item:f64){
        //println!("INCOMING {}", item);
        let roll_result = {
            self.container.borrow_mut().roll(item)
        };
        match roll_result{

            None=>{
                //Unfilled
                let index = self.container.borrow().length()-1;

                self.move_element(index);
            }
            Some((_,i))=>{
                self.move_element(i);
            }
        }
    }

    pub fn print_stats(&self){
        println!("-------------------------");
        self.container.borrow().print_data();
        self.lower.print_heap();
        self.upper.print_heap();
        println!("-------------------------");
    }

}


#[cfg(test)]
mod heap_tests {
    use super::*;

    #[test]
    fn sorted_odd() {
        let mut dheap = DoubleHeap::new(5);
        dheap.roll(0.0);
        dheap.roll(1.0);
        dheap.roll(2.0);
        dheap.roll(3.0);
        dheap.roll(4.0);

        dheap.print_stats();
        assert_eq!(dheap.median(),2.0);
    }

    #[test]
    fn sorted_even() {
        let mut dheap = DoubleHeap::new(4);
        dheap.roll(0.0);
        dheap.roll(1.0);
        dheap.roll(2.0);
        dheap.roll(3.0);

        dheap.print_stats();
        assert_eq!(dheap.median(),1.5);
    }

    #[test]
    fn sorted_even_unfilled() {
        let mut dheap = DoubleHeap::new(10);
        dheap.roll(0.0);
        dheap.roll(1.0);
        dheap.roll(2.0);
        dheap.roll(3.0);

        dheap.print_stats();
        assert_eq!(dheap.median(),1.5);
    }

    #[test]
    fn unsorted_odd() {
        let mut dheap = DoubleHeap::new(5);
        dheap.roll(2.0);
        dheap.roll(1.0);
        dheap.roll(4.0);
        dheap.roll(0.0);
        dheap.roll(3.0);

        dheap.print_stats();
        assert_eq!(dheap.median(),2.0);
    }

    #[test]
    fn unsorted_even() {
        let mut dheap = DoubleHeap::new(6);
        dheap.roll(2.0);
        dheap.roll(1.0);
        dheap.roll(4.0);
        dheap.roll(5.0);
        dheap.roll(0.0);
        dheap.roll(3.0);
        dheap.print_stats();
        assert_eq!(dheap.median(),2.5);
    }

    #[test]
    fn unsorted_overflow() {
        let mut dheap = DoubleHeap::new(5);
        dheap.roll(2.0);
        dheap.roll(1.0);
        dheap.roll(4.0);
        dheap.roll(5.0);
        dheap.roll(0.0);
        dheap.roll(3.0);
        dheap.roll(6.0);
        dheap.roll(9.0);
        dheap.roll(5.0);
        dheap.print_stats();
        assert_eq!(dheap.median(),5.0);
    }
}
