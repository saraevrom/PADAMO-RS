use super::utils::IndexCalculator;
use nom::{
  bytes::complete::tag, character, combinator::cut, IResult, Parser
};


#[derive(Debug)]
pub struct SliceDefFull{
    start:Option<Box<dyn IndexCalculator>>,
    end:Option<Box<dyn IndexCalculator>>,
    stride:Option<Box<dyn IndexCalculator>>,
}

impl SliceDefFull{
    pub fn calculate(&self, operand: &Box<dyn IndexCalculator>, index_src:&[usize], index_lens:&[usize]) -> Option<usize>{
        let start = try_calculate(&self.start, index_src, index_lens).unwrap_or(0);
        let end =  try_calculate(&self.end, index_src, index_lens);
        let stride = try_calculate(&self.stride, index_src, index_lens).unwrap_or(1);
        let x = operand.calculate(index_src, index_lens)?;
        let x_off = usize::checked_sub(x, start)?;
        if let Some(e) = end{
            if x_off>=e{
                return None;
            }
        }
        return usize::checked_div(x_off, stride);
    }
}


#[derive(Debug)]
pub struct SliceDefEq{
    value:Box<dyn IndexCalculator>,
}

impl SliceDefEq{
    pub fn calculate(&self, operand: &Box<dyn IndexCalculator>, index_src:&[usize], index_lens:&[usize]) -> Option<usize>{
        let x0 = self.value.calculate(index_src, index_lens)?;
        let x = operand.calculate(index_src, index_lens)?;
        if x != x0{
            Some(0)
        }
        else{
            None
        }
    }
}


#[derive(Debug)]
pub enum SliceDef{
    Full(SliceDefFull),
    Eq(SliceDefEq)
}


#[derive(Debug)]
pub struct Slice{
    def:SliceDef,
    operand:Box<dyn IndexCalculator>,
}

impl Slice {
    pub fn new(def: SliceDef, operand: Box<dyn IndexCalculator>) -> Self {
        Self { def, operand }
    }
}

impl SliceDefFull{
    fn new() -> Self {
        Self { start: None, end: None, stride: None}
    }

    fn with_start(mut self, start:Box<dyn IndexCalculator>)->Self{
        self.start = Some(start);
        self
    }

    fn with_end(mut self, end:Box<dyn IndexCalculator>)->Self{
        self.end = Some(end);
        self
    }

    fn with_stride(mut self, stride:Box<dyn IndexCalculator>)->Self{
        self.stride = Some(stride);
        self
    }
}

fn try_calculate(x:&Option<Box<dyn IndexCalculator>>, index_src:&[usize], index_lens:&[usize])->Option<usize>{
    if let Some(op) = x{
        op.calculate(index_src, index_lens)
    }
    else{
        None
    }
}

impl IndexCalculator for Slice{
    fn calculate(&self, index_src:&[usize], index_lens:&[usize])->Option<usize> {
        // let start = try_calculate(&self.start, index_src, index_lens).unwrap_or(0);
        // let end =  try_calculate(&self.end, index_src, index_lens);
        // let stride = try_calculate(&self.stride, index_src, index_lens).unwrap_or(1);
        // let x = self.operand.calculate(index_src, index_lens)?;
        // let x_off = usize::checked_sub(x, start)?;
        // if let Some(e) = end{
        //     if x_off>=e{
        //         return None;
        //     }
        // }
        // return usize::checked_div(x_off, stride);
        match &self.def{
            SliceDef::Full(f)=>f.calculate(&self.operand, index_src, index_lens),
            SliceDef::Eq(e) => e.calculate(&self.operand, index_src, index_lens)
        }
    }
}



pub fn parse_one_num(input: &str) -> IResult<&str, SliceDef> {
    let (input, i) = crate::expression::parse_expression(input)?;
    let res = SliceDef::Eq(SliceDefEq { value: i });
    Ok((input,res))
}


pub fn parse_two_num(input: &str) -> IResult<&str, SliceDef> {
    let (input, start) = crate::expression::parse_expression(input)?;
    let (input, _) = crate::utils::parse_colon_sep(input)?;

    let (input, end) = crate::expression::parse_expression.parse(input)?;

    let res = SliceDef::Full(SliceDefFull::new().with_start(start).with_end(end));
    Ok((input,res))

}

pub fn parse_three_num(input: &str) -> IResult<&str, SliceDef> {
    let (input, start) = crate::expression::parse_expression(input)?;
    let (input, _) = crate::utils::parse_colon_sep(input)?;

    let (input, end) = crate::expression::parse_expression.parse(input)?;
    let (input, _) = crate::utils::parse_colon_sep(input)?;

    let (input, stride) = crate::expression::parse_expression.parse(input)?;

    let res = SliceDef::Full(SliceDefFull::new().with_start(start).with_end(end).with_stride(stride));
    Ok((input,res))
}

pub fn parse_slice(input: &str) -> IResult<&str, SliceDef> {
    let (input, _) = tag("[").parse(input)?;
    let (input, res) = nom::branch::alt((parse_three_num, parse_two_num, parse_one_num)).parse(input)?;
    let (input, _) = cut(tag("]")).parse(input)?;
    Ok((input,res))
}
