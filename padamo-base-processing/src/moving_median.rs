use atomic_float::AtomicF64;
use fast_mm::DoubleHeap;
use padamo_api::lazy_array_operations::ArrayND;
use std::sync::{Arc,Mutex};
use std::collections::VecDeque;
use std::thread;
//use std::thread::available_parallelism;

fn free_threads(threads: &mut VecDeque<thread::JoinHandle<()>>, threadcount:usize){
    while threads.len()>=threadcount{
        //println!("Working threads: {}",threads.len());
        if let Some(handle)= threads.pop_front() {
            if handle.is_finished(){
                //println!("Freeing one handle");
                if let Err(e) = handle.join(){
                    println!("{:?}",e);
                }
                //println!("Freed one handle");
            }
            else{
                threads.push_back(handle)
            }
        }
        else{
            break;
        }
    }
}

pub fn temporal_moving_median(array:ArrayND<f64>,window:usize)->ArrayND<f64>{
    let shape = array.shape.clone();
    if shape[0]<window{
        panic!("Cannot use sliding median with array smaller than window");
    }
    let frame_size:usize = shape.iter().skip(1).fold(1, |a,b| a*b);

    let mut target_shape = shape.clone();
    target_shape[0] = shape[0] - window + 1;
    let target_length = target_shape[0];

    // let target = Arc::new(Mutex::new(ArrayND::<f64>::new(target_shape.into(),-666.0)));
    let target_flat_len:usize = target_shape.iter().map(|x|*x).product();
    let mut target_flat:Vec<AtomicF64> = Vec::with_capacity(target_flat_len);
    target_flat.resize_with(target_flat_len, || AtomicF64::new(0.0));

    let target_flat = Arc::new(target_flat);

    let source = Arc::new(array);

    let threadcount = num_cpus::get();
    let mut threads:VecDeque<thread::JoinHandle<()>> = VecDeque::with_capacity(10);

    for pixel in 0..frame_size{
        free_threads(&mut threads, threadcount);
        // let tgt = target.clone();
        let tgt_f = target_flat.clone();
        let src = source.clone();
        // let pixel_index = pixel;
        let offset = pixel;
        //println!("Starting pixel {}", pixel);
        let handle = thread::spawn(move || {
            // if target_length==1{
            //     let med = src.flat_data[0..window].medf_checked().unwrap();
            //     tgt.lock().unwrap().flat_data[offset] = med;
            // }
            // else{
            let mut roller = DoubleHeap::new(window);
            //println!("Prefilling pixel {}", pixel_index);
            for i in 0..window-1{
                let index = i*frame_size+offset;
                let value = src.flat_data[index];
                //println!("Pixel {} preroll {}", pixel_index, i);
                roller.roll(value);
                //println!("Pixel {} preroll_end {}", pixel_index, i);
            }
            //println!("Calculating pixel {}", pixel_index);
            for i in 0..target_length{
                let index = (i+window-1)*frame_size+offset;
                //println!("Pixel {} roll {}", pixel_index, i);
                roller.roll(src.flat_data[index]);
                //println!("Pixel M={}", roller.median());
                //roller.print_stats();
                //println!("Pixel {} afterroll {}", pixel_index, i);
                tgt_f[i*frame_size+offset].fetch_add(roller.median(), std::sync::atomic::Ordering::Relaxed);
                // tgt.lock().unwrap().flat_data[i*frame_size+offset] = roller.median();
                //println!("Pixel {} write {}", pixel_index, i+window-1);
            }
            // }


        });
        threads.push_back(handle);
    }


    free_threads(&mut threads, 1);

    // let lock = Arc::try_unwrap(target).unwrap();
    // lock.into_inner().unwrap()

    let mut target_flat = Arc::try_unwrap(target_flat).unwrap();
    let result = ArrayND {shape:target_shape.into(), flat_data: target_flat.drain(..).map(|x| x.into_inner()).collect()};
    result.assert_shape();
    result
}
