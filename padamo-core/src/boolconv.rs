use std::collections::VecDeque;
use std::{sync::Arc, thread};
use std::sync::Mutex;

use padamo_api::lazy_array_operations::ArrayND;



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

pub fn temporal_conv_bool(array:ArrayND<bool>,expansion:usize)->ArrayND<bool>{
    let shape = array.shape.clone();
    let frame_size:usize = shape.iter().skip(1).fold(1, |a,b| a*b);

    let target_shape = shape.clone();
    let target_length = target_shape[0];

    let target = Arc::new(Mutex::new(ArrayND::<bool>::new(target_shape.into(),false)));
    let source = Arc::new(array);

    let threadcount = num_cpus::get();
    let mut threads:VecDeque<thread::JoinHandle<()>> = VecDeque::with_capacity(10);

    for pixel in 0..frame_size{
        free_threads(&mut threads, threadcount);
        let tgt = target.clone();
        let src = source.clone();
        let pixel_index = pixel;
        let offset = pixel;
        //println!("Starting pixel {}", pixel);
        let handle = thread::spawn(move || {

            //let mut roller = DoubleHeap::new(window);


            //println!("Calculating pixel {}", pixel_index);
            for i in 0..target_length{
                //let index = (i+window-1)*frame_size+offset;
                //println!("Pixel {} roll {}", pixel_index, i);
                //roller.roll(src.flat_data[index]);
                //println!("Pixel M={}", roller.median());
                //roller.print_stats();
                //println!("Pixel {} afterroll {}", pixel_index, i);
                let start_i = if i<expansion {0} else{i-expansion};
                let end_i = if i>=target_length-expansion {target_length} else{i+expansion};

                let mut conv = false;
                for j in start_i..end_i{
                    conv = conv || src.flat_data[j*frame_size+offset];
                }

                tgt.lock().unwrap().flat_data[i*frame_size+offset] = conv;
                //println!("Pixel {} write {}", pixel_index, i+window-1);
            }


        });
        threads.push_back(handle);
    }


    free_threads(&mut threads, 1);

    let lock = Arc::try_unwrap(target).unwrap();
    lock.into_inner().unwrap()

}
