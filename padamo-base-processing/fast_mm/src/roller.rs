

pub struct SwapperRollerArray{
    elements:Vec<f64>,
    order:Vec<usize>,
    conj_order:Vec<usize>,
    start:usize,
    end:usize,
    max_size:usize,
}

impl SwapperRollerArray{
    pub fn new(max_size:usize)->Self{
        let mut order = Vec::with_capacity(max_size);
        order.resize(max_size,0);
        let mut conj_order = Vec::with_capacity(max_size);
        conj_order.resize(max_size,0);
        let mut elements = Vec::with_capacity(max_size);
        elements.resize(max_size,0.0);
        Self{
            max_size,
            start:0,
            end:0,
            order,
            conj_order,
            elements
        }
    }

    pub fn length(&self)->usize{
        self.end-self.start
    }

    // fn remap_index(&self,i:usize)->usize{
    //     (i+start)%self.max_size
    // }

    pub fn roll(&mut self, item:f64)->Option<(f64,usize)>{
        if self.length()<self.max_size{
            //Prefill
            let last_i = self.end%self.max_size;
            self.elements[last_i] = item;
            self.order[last_i] = self.end;
            self.conj_order[last_i] = self.end;
            self.end += 1;
            None
        }
        else{
            //Roll
            let index = self.start%self.max_size;
            let res = self.elements[index];
            self.elements[index] = item;
            let number = self.conj_order[index];

            self.start += 1;
            self.end += 1;
            Some((res,number))
        }
    }

    pub fn swap(&mut self,i:usize,j:usize){
        let conj_i = self.order[i];
        let conj_j = self.order[j];
        (self.conj_order[conj_i],self.conj_order[conj_j])=(self.conj_order[conj_j],self.conj_order[conj_i]);
        (self.order[i],self.order[j])=(self.order[j],self.order[i]);
    }

    pub fn print_order(&self){
        println!("{:?}",self.order);
        println!("{:?}",self.conj_order);
    }

    pub fn print_data(&self){
        for i in 0..self.length(){
            let j = self.order[i];
            print!("{} ",self.elements[j]);
        }
        println!()
    }

    pub fn get(&self,i:usize)->f64{
        let conj_i = self.order[i];
        self.elements[conj_i]
    }

}

#[cfg(test)]
mod roller_tests {
    use super::*;

    #[test]
    fn fill() {
        let mut roller = SwapperRollerArray::new(5);
        assert_eq!(roller.roll(0.0),None);
        assert_eq!(roller.roll(1.0),None);
        assert_eq!(roller.roll(2.0),None);
        assert_eq!(roller.roll(3.0),None);
        assert_eq!(roller.roll(4.0),None);
        assert_eq!(roller.roll(5.0),Some((0.0, 0)));
    }

    #[test]
    fn sequence() {
        let mut roller = SwapperRollerArray::new(5);
        assert_eq!(roller.roll(2.0),None);
        assert_eq!(roller.roll(5.0),None);
        assert_eq!(roller.roll(5.0),None);
        assert_eq!(roller.roll(6.0),None);
        assert_eq!(roller.roll(5.0),None);
        assert_eq!(roller.roll(8.0),Some((2.0,0)));
        assert_eq!(roller.roll(42.0),Some((5.0,1)));
        assert_eq!(roller.roll(6.0),Some((5.0,2)));
        assert_eq!(roller.roll(0.0),Some((6.0,3)));
    }

    #[test]
    fn multiroll() {
        let mut roller = SwapperRollerArray::new(5);
        //Observing

        //Prefill (Roll 1)
        assert_eq!(roller.roll(2.0),None);
        assert_eq!(roller.roll(5.0),None);
        assert_eq!(roller.roll(5.0),None);
        assert_eq!(roller.roll(6.0),None);
        assert_eq!(roller.roll(5.0),None);

        //Roll 2
        assert_eq!(roller.roll(8.0), Some((2.0,0)));
        assert_eq!(roller.roll(42.0),Some((5.0,1)));
        assert_eq!(roller.roll(6.0), Some((5.0,2)));
        assert_eq!(roller.roll(0.0), Some((6.0,3)));
        assert_eq!(roller.roll(1.0), Some((5.0,4)));


        //Roll 3
        assert_eq!(roller.roll(24.0),Some((8.0,0)));
        assert_eq!(roller.roll(33.0),Some((42.0,1)));
        assert_eq!(roller.roll(56.0),Some((6.0,2)));
        assert_eq!(roller.roll(9.0), Some((0.0,3)));
        assert_eq!(roller.roll(1.5), Some((1.0,4)));

        //Roll 4
        assert_eq!(roller.roll(3.3),Some((24.0,0)));
        assert_eq!(roller.roll(8.5),Some((33.0,1)));
        assert_eq!(roller.roll(1.3),Some((56.0,2)));
        assert_eq!(roller.roll(4.2),Some((9.0,3)));
        assert_eq!(roller.roll(6.0),Some((1.5,4)));
    }

    #[test]
    fn swap_multiroll() {
        let mut roller = SwapperRollerArray::new(5);
        //Observing

        //Prefill (Roll 1)
        assert_eq!(roller.roll(2.0),None);
        assert_eq!(roller.roll(5.0),None);
        roller.swap(0,1);
        assert_eq!(roller.roll(5.0),None);
        assert_eq!(roller.roll(6.0),None);
        assert_eq!(roller.roll(5.0),None);


        roller.swap(1,2);
        roller.swap(3,4);

        //Roll 2
        assert_eq!(roller.roll(8.0), Some((2.0,2)));
        assert_eq!(roller.roll(42.0),Some((5.0,0)));
        assert_eq!(roller.roll(6.0), Some((5.0,1)));
        assert_eq!(roller.roll(0.0), Some((6.0,4)));
        assert_eq!(roller.roll(1.0), Some((5.0,3)));


        //Roll 3
        assert_eq!(roller.roll(24.0),Some((8.0,2)));
        assert_eq!(roller.roll(33.0),Some((42.0,0)));
        assert_eq!(roller.roll(56.0),Some((6.0,1)));
        assert_eq!(roller.roll(9.0), Some((0.0,4)));
        assert_eq!(roller.roll(1.5), Some((1.0,3)));

    }

    #[test]
    fn ordering(){

        let mut roller = SwapperRollerArray::new(5);
        //Observing

        //Prefill (Roll 1)
        assert_eq!(roller.roll(1.0),None);
        assert_eq!(roller.roll(2.0),None);
        assert_eq!(roller.roll(3.0),None);
        assert_eq!(roller.roll(4.0),None);
        assert_eq!(roller.roll(5.0),None);


        roller.swap(0,1);
        roller.swap(1,2);
        roller.swap(3,4);

        roller.print_order();

        assert_eq!(roller.get(0),2.0);
        assert_eq!(roller.get(1),3.0);
        assert_eq!(roller.get(2),1.0);
        assert_eq!(roller.get(3),5.0);
        assert_eq!(roller.get(4),4.0);
    }

    #[test]
    fn shuffler(){

        let mut roller = SwapperRollerArray::new(5);
        //Observing

        //Prefill (Roll 1)
        assert_eq!(roller.roll(1.0),None);
        assert_eq!(roller.roll(2.0),None);
        assert_eq!(roller.roll(3.0),None);
        assert_eq!(roller.roll(4.0),None);
        assert_eq!(roller.roll(5.0),None);


        roller.swap(0,1);
        roller.swap(1,2);
        roller.swap(3,4);
        roller.swap(2,3);

        roller.print_order();


        assert_eq!(roller.get(3),1.0);
        assert_eq!(roller.get(0),2.0);
        assert_eq!(roller.get(1),3.0);
        assert_eq!(roller.get(4),4.0);
        assert_eq!(roller.get(2),5.0);


        assert_eq!(roller.roll(6.0), Some((1.0,3)));
        assert_eq!(roller.roll(7.0), Some((2.0,0)));
        assert_eq!(roller.roll(8.0), Some((3.0,1)));
        assert_eq!(roller.roll(9.0), Some((4.0,4)));
        assert_eq!(roller.roll(10.0),Some((5.0,2)));
    }
}
