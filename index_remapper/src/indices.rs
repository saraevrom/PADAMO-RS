use super::utils::IndexCalculator;
use nom::{
  bytes::complete::tag, character::{self, complete::digit1}, combinator::{cut, map_res}, Finish, IResult, Parser
};


#[derive(Debug)]
pub struct IndexSource(pub usize);

#[derive(Debug)]
pub struct IndexWidthSource(pub usize);

#[derive(Debug)]
pub struct ConstantSource(pub i64);

#[derive(Debug)]
pub struct NothingSource;



impl IndexCalculator for IndexSource{
    fn calculate(&self, index_src:&[usize],_index_sizes:&[usize])->Option<i64> {
        index_src.get(self.0).map(|x| *x as i64)
    }
}

impl IndexCalculator for ConstantSource{
    fn calculate(&self, _index_src:&[usize], _index_sizes:&[usize])->Option<i64> {
        Some(self.0)
    }
}

impl IndexCalculator for IndexWidthSource{
    fn calculate(&self, _index_src:&[usize], index_sizes:&[usize])->Option<i64> {
        index_sizes.get(self.0).map(|x| *x as i64)
    }
}

impl IndexCalculator for NothingSource{
    fn calculate(&self, _index_src:&[usize], _index_sizes:&[usize])->Option<i64> {
        None
    }
}



pub fn parse_index(input: &str) -> IResult<&str, Box<dyn IndexCalculator>> {
    let (input,_) = tag("i")(input)?;
    let (input, id) = cut(character::complete::usize).parse(input)?;
    Ok((input, Box::new(IndexSource(id))))
}

pub fn parse_index_width(input: &str) -> IResult<&str, Box<dyn IndexCalculator>> {
    let (input,_) = tag("i")(input)?;
    let (input, id) = character::complete::usize.parse(input)?;
    let (input,_) = tag("w").parse(input)?;

    Ok((input, Box::new(IndexWidthSource(id))))
}

pub fn parse_constant(input: &str) -> IResult<&str, Box<dyn IndexCalculator>> {
    let (input, id) = map_res(digit1, |s: &str| s.parse::<i64>()).parse(input)?;
    Ok((input, Box::new(ConstantSource(id))))
}
