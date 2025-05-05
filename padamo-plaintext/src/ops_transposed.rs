use std::{fs::File, io::{BufRead, BufReader}};
use abi_stable::rvec;

use padamo_api::lazy_array_operations::{merge::Merge, ArrayND, LazyArrayOperation};
use regex::Regex;

use crate::errors::CSVError;


// Each COLUMN is separate frame

#[derive(Clone,Debug)]
pub struct CSVReaderTransposed{
    pub separator:Regex,
    pub filename:String,
    pub start_line:usize,
    pub length:usize,
    pub frame_size:usize,
    pub row_bounds:(usize,usize)
}

impl CSVReaderTransposed{
    pub fn new(separator: String, filename: String, start_line: usize, length: Option<usize>, lower_bound:Option<usize>, upper_bound:Option<usize>) -> Result<Self, CSVError> {
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
        let mut res = Self { separator, filename, start_line, length, frame_size:0, row_bounds:(0,0)};

        let frame_size = res.read_lines_csv(start_line,1,false)?[0].len();
        res.frame_size = frame_size;
        //let row_bounds = rows_bounds.unwrap_or((0,frame_size));
        let lower_bound = lower_bound.unwrap_or(0);
        let upper_bound = upper_bound.unwrap_or(frame_size);
        if lower_bound>=frame_size || upper_bound>frame_size || upper_bound<=lower_bound{
            return Err(CSVError::InvalidSlice(lower_bound, upper_bound, frame_size));
        }
        res.row_bounds = (lower_bound, upper_bound);
        res.frame_size = upper_bound-lower_bound;
        Ok(res)
    }

    fn read_lines_csv(&self, line_start:usize, amount:usize, limit:bool)->Result<Vec<Vec<f64>>, CSVError>{
        //println!("Reading lines {} - {}", line_start, line_start+amount);
        //println!("Reading columns {:?}",self.row_bounds);
        let f = File::open(&self.filename).map_err(CSVError::IOError)?;
        let reader = BufReader::new(f);
        let mut res = vec![];
        //println!("Init OK");
        for line in reader.lines().skip(line_start).take(amount){
            let line = line.map_err(CSVError::IOError)?;
            let line = line.trim();
            //println!("LINE: {}",line);
            let items:Vec<f64> = if limit {self.separator.split(line)
                .skip(self.row_bounds.0).take(self.row_bounds.1-self.row_bounds.0).map(|x| x.parse::<f64>()).filter(|x| x.is_ok()).map(|x| x.unwrap()).collect()
            }
            else {
                self.separator.split(line).map(|x| x.parse::<f64>()).filter(|x| x.is_ok()).map(|x| x.unwrap()).collect()
            };
            //println!("ITEMS {:?}", items);
            res.push(items);
        }
        Ok(res)
        // else{
        //     Ok(vec![])
        // }
    }
    fn read_columns_csv(&self, column_start:usize, amount:usize)->Result<Vec<Vec<f64>>, CSVError>{
        let f = File::open(&self.filename).map_err(CSVError::IOError)?;
        let reader = BufReader::new(f);
        let mut res = vec![vec![];amount];
        // res.fill
        for line in reader.lines().skip(self.start_line).take(self.length){
            let line = line.map_err(CSVError::IOError)?;
            let line = line.trim();
            // println!("LINE: {}",line);
            let items:Vec<f64> = self.separator.split(line).skip(column_start).take(amount).map(|x| x.parse::<f64>()).filter(|x| x.is_ok()).map(|x| x.unwrap()).collect();
            // println!("ITEMS: {:?}",items);
            items.iter().enumerate().for_each(|(i,x)| {
                res[i].push(*x);
            });
        }
        //println!("RES {:?}", res);
        Ok(res)
    }
}


impl LazyArrayOperation<ArrayND<f64>> for CSVReaderTransposed{
    fn length(&self,) -> usize where {
        self.frame_size
    }

    fn calculate_overhead(&self,start:usize,end:usize,) -> usize where {
        end-start
    }

    fn request_range(&self,start:usize,end:usize,) -> ArrayND<f64> where {
        let mut res = self.read_columns_csv(start+self.row_bounds.0, end-start).unwrap();
        let res = res
            .drain(..)
            .map(|x| ArrayND{flat_data: x.into(), shape:rvec![1,self.length]})
            .fold(ArrayND::new(vec![0,self.length], 0.0),|a,b| a.merge(b));
        res
    }
}
