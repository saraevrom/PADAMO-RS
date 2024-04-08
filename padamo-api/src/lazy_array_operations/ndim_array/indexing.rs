use super::ArrayND;

pub struct ShapeIterator{
    source_shape:Vec<usize>,
    current_data:Vec<usize>,
    initialized:bool
}

impl ShapeIterator{
    pub fn new(source_shape:Vec<usize>)->Self{
        let current_data = source_shape.iter().map(|_| 0).collect();
        Self { source_shape, current_data, initialized:false}
    }
}

impl Iterator for ShapeIterator{
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.initialized{
            self.initialized = true;
            return Some(self.current_data.clone());
        }
        let mut i = 0;
        while i<self.source_shape.len(){
            self.current_data[i] +=1;
            if self.current_data[i]<self.source_shape[i]{
                return Some(self.current_data.clone());
            }
            else{
                self.current_data[i] = 0;
                i +=1;
            }
        }
        None
    }
}

