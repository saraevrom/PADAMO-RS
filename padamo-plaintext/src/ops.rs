use std::{fs::File, io::{self, BufRead, BufReader}};
use abi_stable::rvec;

use padamo_api::{lazy_array_operations::{merge::Merge, ArrayND, LazyArrayOperation}, prelude::*};
use regex::Regex;

use crate::errors::CSVError;

#[derive(Clone,Debug)]
pub struct CSVReader{
    pub separator:Regex,
    pub filename:String,
    pub start_line:usize,
    pub length:usize,
    pub frame_size:usize,
}

impl CSVReader{
    pub fn new(separator: String, filename: String, start_line: usize, length: Option<usize>) -> Result<Self, CSVError> {
        let separator = Regex::new(&separator)?;
        let f = File::open(&filename)?;
        let reader = BufReader::new(f);
        let total_length = reader.lines().count();
        let length = if let Some(l) = length {
            l
        }
        else if start_line>=total_length{
            return Err(CSVError::InvalidLength{total_length,start_line,length});
        }
        else{
            total_length-start_line
        };

        if start_line>=total_length || start_line+length>total_length || length==0{
            return Err(CSVError::InvalidLength{total_length,start_line,length:Some(length)});
        }
        let mut res = Self { separator, filename, start_line, length, frame_size:0};

        let frame_size = res.read_lines_csv(start_line,1)?[0].len();
        res.frame_size = frame_size;
        Ok(res)
    }

    fn read_lines_csv(&self, line_start:usize, amount:usize)->Result<Vec<Vec<f64>>, CSVError>{
        let f = File::open(&self.filename).map_err(CSVError::IOError)?;
        let reader = BufReader::new(f);
        let mut res = vec![];
        for line in reader.lines().skip(line_start).take(amount){
            let line = line.map_err(CSVError::IOError)?;
            let line = line.trim();
            let items:Vec<f64> = self.separator.split(line).map(|x| x.parse()).filter(|x| x.is_ok()).map(|x| x.unwrap()).collect();
            res.push(items);
        }
        Ok(res)
        // else{
        //     Ok(vec![])
        // }
    }
}


impl LazyArrayOperation<ArrayND<f64>> for CSVReader{
    fn length(&self,) -> usize where {
        self.length
    }

    fn calculate_overhead(&self,start:usize,end:usize,) -> usize where {
        end-start
    }

    fn request_range(&self,start:usize,end:usize,) -> ArrayND<f64> where {
        let mut res = self.read_lines_csv(start+self.start_line, end-start).unwrap();
        let res = res
            .drain(..)
            .map(|x| ArrayND{flat_data: x.into(), shape:rvec![1,self.frame_size]})
            .fold(ArrayND::new(vec![0,self.frame_size], 0.0),|a,b| a.merge(b));
        res
    }
}
