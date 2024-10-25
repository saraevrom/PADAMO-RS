
pub fn hsv_to_rgb(h:f32,s:f32,v:f32)->(f32,f32,f32){

    // '''
    // 0<=h<=360
    // 0<=s<=1
    // 0<=v<=1
    // '''


    let h = h % 360.0;
    let c = v*s;
    let x = c*(1.0-((h/60.0)%2.0-1.0).abs());
    let m = v-c;

    let r:f32;
    let g:f32;
    let b:f32;


    if h<60.0{
        r = c;
        g = x;
        b = 0.0;
    }
    else if h<120.0{
        r = x;
        g = c;
        b = 0.0;
    }
    else if h<180.0{
        r = 0.0;
        g = c;
        b = x;
    }
    else if h<240.0{
        r = 0.0;
        g = x;
        b = c;
    }
    else if h<300.0{
        r = x;
        g = 0.0;
        b = c;
    }
    else{
        r = c;
        g = 0.0;
        b = x;
    }
    (r+m, g+m, b+m)
}

pub fn h_color(i:f32, hue_shift:f32,s_shift:f32, v_shift:f32)->(f32,f32,f32){
    let h = ((i/8.0))*360.0+hue_shift;
    let s = 1.0-s_shift;
    let v = 1.0-v_shift;
    hsv_to_rgb(h,s,v)
}

pub fn floormod(a:f32,b:f32)->f32{
    let base = (a/b).floor()*b;
    return a-base;
}

pub fn get_color(i:usize,j:usize)->(f32,f32,f32){
    let j1;
    if i%2==0{
        j1 = j;
    }
    else{
        j1 = j + 1
    }
    let shift_id = floormod(floormod(i as f32-(j1 as f32)*4.0,16.0),16.0);
    let mut gray_shift = 0.0;
    if j%2==0 && (i as isize-((j/2) as isize))%2==0{
        gray_shift = 1.0;
    }
    h_color(shift_id,(j as f32)*180.0/16.0,gray_shift*0.3,gray_shift*0.5)
}

