use abi_stable::std_types::RVec;

use padamo_api::lazy_array_operations::LazyArrayOperation;

use crate::errors::CSVError;


#[derive(Clone,Debug)]
pub struct CSVTimeColumnReader{
    pub reader:crate::ops::CSVReader,
}

impl CSVTimeColumnReader{
    pub fn new(separator:String, filename:String, start_line:usize, length:Option<usize>, column:usize)->Result<Self, CSVError>{
        let reader = crate::ops::CSVReader::new(separator, filename, start_line, length, Some(column), Some(column+1))?;
        Ok(Self{reader})
    }
}

impl LazyArrayOperation<RVec<f64>> for CSVTimeColumnReader{
    fn length(&self,) -> usize where {
        self.reader.length()
    }

    fn request_range(&self,start:usize,end:usize,) -> RVec<f64>where {
        let res = self.reader.request_range(start, end);
        if res.flat_data.len()!= end-start {
            panic!("CSV time column reader inner error: items length mismatch");
        }
        res.flat_data
    }

    fn calculate_overhead(&self,start:usize,end:usize,) -> usize where {
        self.reader.calculate_overhead(start, end)
    }
}

#[derive(Clone,Debug)]
pub struct CSVTimeRowReader{
    pub reader:crate::ops_transposed::CSVReaderTransposed,
}

impl CSVTimeRowReader{
    pub fn new(separator:String, filename:String, line:usize, lower_bound:Option<usize>, upper_bound:Option<usize>)->Result<Self, CSVError>{
        let reader = crate::ops_transposed::CSVReaderTransposed::new(separator, filename, line, Some(1), lower_bound, upper_bound)?;
        Ok(Self{reader})
    }
}

impl LazyArrayOperation<RVec<f64>> for CSVTimeRowReader{
    fn length(&self,) -> usize where {
        self.reader.length()
    }

    fn request_range(&self,start:usize,end:usize,) -> RVec<f64>where {
        let res = self.reader.request_range(start, end);
        if res.flat_data.len()!= end-start {
            panic!("CSV time column reader inner error: items length mismatch");
        }
        res.flat_data
    }

    fn calculate_overhead(&self,start:usize,end:usize,) -> usize where {
        self.reader.calculate_overhead(start, end)
    }
}
