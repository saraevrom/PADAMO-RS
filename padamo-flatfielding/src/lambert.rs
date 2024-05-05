use std::f64::consts::E;

fn lambertw0_binary(x:f64, precision:f64)->f64{
    if x < -1.0/E{
        return -1.0;
    }
    let mut left = -1.0;
    let mut right = if x<=E {E} else {x};
    while right-left>precision{
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


fn lambertw0_newtonian(y:f64,precision:f64)->f64{
    if y <= -1.0/E{
        return -1.0;
    }

    //f(x)=x exp(x) (x<1.0)
    //f'(x)=(1 + x)*exp(x) (x<1.0)

    let mut prev_x = y+precision*10.0;
    let mut x = y;
    while (x-prev_x).abs()>precision{
        let grad = (1.0 + x)*x.exp();
        if grad==0.0{
            panic!("LambertW0: How did we get here?");
        }
        let y0 = x*x.exp();
        let new_x = x+(y-y0)/grad;
        prev_x = x;
        x = new_x;
    }
    x

}


pub fn lambertw0(x:f64)->f64{
    //lambertw0_binary(x, 1e-12)
    lambertw0_newtonian(x, 1e-12)
}
