use std::f64::consts::E;

pub fn lambertw0(x:f64)->f64{
    let mut left = -1.0;
    let mut right = if x<=E {E} else {x};
    while right-left>1e-12{
        let mid = (left+right)/2.0;
        if mid*mid.exp()>x{
            right = mid;
        }
        else{
            left = mid;
        }
    }
    right
}
