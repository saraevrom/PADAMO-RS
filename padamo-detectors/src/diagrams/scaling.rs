use padamo_arraynd::ArrayND;

#[derive(Clone,Copy,Debug,serde::Serialize, serde::Deserialize)]
pub enum Scaling{
    Autoscale,
    Fixed(f64,f64)
}

impl Scaling{
    pub fn get_bounds(&self, frame:&ArrayND<f64>, alive_pixels:&ArrayND<bool>)->(f64,f64){
        match self{
            Self::Autoscale=>{

                // let (min,max) = frame.flat_data.iter()
                //     .enumerate()
                //     .filter(|x| alive_pixels.flat_data.get(x.0).map(|y|*y).unwrap_or(false))
                //     .map(|x| x.1)
                //     .fold((first,first), |a,b| (a.0.min(*b),a.1.max(*b)));
                let min = frame.flat_data
                .iter()
                .enumerate()
                .filter(|x| alive_pixels.flat_data.get(x.0).map(|y|*y).unwrap_or(false))
                .min_by(|a,b| a.1.total_cmp(b.1))
                .map(|x| x.1);
                let max = frame.flat_data
                .iter()
                .enumerate()
                .filter(|x| alive_pixels.flat_data.get(x.0).map(|y|*y).unwrap_or(false))
                .max_by(|a,b| a.1.total_cmp(b.1))
                .map(|x| x.1);
                match (min, max){
                    (Some(l),Some(u)) => {
                        if max<=min{
                            (l-0.1,l+0.1)
                        }
                        else{
                            (*l,*u)
                        }
                    }
                    _ =>{
                        (-0.1, 0.1)
                    }

                }
            }
            Self::Fixed(min, max)=>{
                if max<=min{
                    (*min,*min+0.001)
                }
                else{
                    (*min,*max)
                }
            }
        }
    }
}
