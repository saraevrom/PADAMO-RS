use std::collections::VecDeque;

fn partition(list:&mut [f64], left:usize, right:usize, pivot:usize)->usize{
    let pivot_value = list[pivot];
    list.swap(pivot, right);
    let mut store_index = left;
    for i in left..right{
        if list[i]< pivot_value{
            list.swap(store_index, i);
            store_index+=1;
        }
    }
    list.swap(right, store_index);
    store_index
}

fn select(list:&mut [f64], mut left:usize, mut right:usize, k:usize) ->f64{
    while left != right{
        let mut pivot = (left+right)/2;
        pivot = partition(list, left, right, pivot);
        if k==pivot{
            return list[k];
        }
        else if k<pivot{
            right = pivot-1;
        }
        else{
            left = pivot+1;
        }
    }
    list[left]
}

fn quickselect(list:&mut [f64], k:usize) -> f64{
    select(list, 0, list.len()-1, k)
}

pub fn quantile(list:&mut [f64], mut q:f64) -> f64{
    if q<0.0{
        q = 0.0;
    }
    if q>1.0{
        q = 1.0;
    }
    // let k1 = list.len()*q;
    let total_length = (list.len()-1) as f64;
    let partial_length = total_length*q;



    let mut interpolation = partial_length - partial_length.floor();

    let mut lower_k = partial_length.floor() as usize;
    if lower_k+2>list.len(){
        lower_k = list.len() - 2;
        interpolation += 1.0;
    }
    let upper_k = lower_k+1;

    if interpolation==0.0{
        return quickselect(list, lower_k);
    }

    if interpolation>=1.0{
        return quickselect(list, upper_k);
    }

    let a = quickselect(list, lower_k);
    let b = quickselect(list, upper_k);
    a+(b-a)*interpolation
}

struct WindowIterator<T:Iterator<Item=f64>>{
    source:T,
    current_window:VecDeque<f64>,
}

impl<T: Iterator<Item=f64>> WindowIterator<T> {
    fn new(mut source: T, window: usize) -> Self {
        let mut current_window:VecDeque<f64> = VecDeque::new();
        current_window.push_back(f64::NAN); // This value will be quickly discarded
        for _ in 0..window-1{
            current_window.push_back(source.next().unwrap());
        }

        Self { source, current_window}
    }
}

impl<T: Iterator<Item=f64>> Iterator for WindowIterator<T>{
    type Item = Vec<f64>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(v) = self.source.next(){
            self.current_window.pop_front();
            self.current_window.push_back(v);
            Some(self.current_window.iter().map(|x| *x).collect())
        }
        else{
            None
        }
    }

}

pub fn slide_quantile<T:Iterator<Item=f64>>(seq:T, window:usize, q:f64)->Vec<f64>{
    let mut res = Vec::new();
    for mut subsequence in WindowIterator::new(seq, window){
        res.push(quantile(&mut subsequence, q));

    }
    res
}


#[cfg(test)]
mod tests{
    use super::quantile;
    #[test]
    fn test_median(){
        let mut arr = vec![5., 4., 1., 2., 3. ];
        let med = quantile(&mut arr, 0.5);
        assert_eq!(med, 3.0);
    }

    #[test]
    fn test_median_even(){
        let mut arr = vec![4., 1., 2., 3. ];
        let med = quantile(&mut arr, 0.5);
        assert_eq!(med, 2.5);
    }

    #[test]
    fn test_min(){
        let mut arr = vec![5., 4., 1., 2., 3. ];
        let med = quantile(&mut arr, 0.0);
        assert_eq!(med, 1.0);
    }

    #[test]
    fn test_max(){
        let mut arr = vec![5., 4., 1., 2., 3. ];
        let med = quantile(&mut arr, 1.0);
        assert_eq!(med, 5.0);
    }

    #[test]
    fn test_weird(){
        let mut arr = vec![5., 4., 1., 2., 3. ];
        let med = quantile(&mut arr, 0.25);
        assert_eq!(med, 2.0);
    }
}
