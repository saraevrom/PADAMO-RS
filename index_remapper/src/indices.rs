use super::utils::IndexCalculator;
use nom::{
  bytes::complete::tag, character, combinator::cut, Finish, IResult, Parser
};


pub struct IndexSource(pub usize);
pub struct IndexWidthSource(pub usize);
pub struct ConstantSource(pub usize);
pub struct NothingSource;



impl IndexCalculator for IndexSource{
    fn calculate(&self, index_src:&[usize],_index_sizes:&[usize])->Option<usize> {
        index_src.get(self.0).copied()
    }
}

impl IndexCalculator for ConstantSource{
    fn calculate(&self, _index_src:&[usize], _index_sizes:&[usize])->Option<usize> {
        Some(self.0)
    }
}

impl IndexCalculator for IndexWidthSource{
    fn calculate(&self, _index_src:&[usize], index_sizes:&[usize])->Option<usize> {
        index_sizes.get(self.0).copied()
    }
}

impl IndexCalculator for NothingSource{
    fn calculate(&self, _index_src:&[usize], _index_sizes:&[usize])->Option<usize> {
        None
    }
}



pub fn parse_index(input: &str) -> IResult<&str, Box<dyn IndexCalculator>> {
    let (input,_) = tag("i")(input)?;
    let (input, id) = cut(character::complete::usize).parse(input)?;
    Ok((input, Box::new(IndexSource(id))))
}

pub fn parse_constant(input: &str) -> IResult<&str, Box<dyn IndexCalculator>> {
    let (input, id) = character::complete::usize(input)?;
    Ok((input, Box::new(ConstantSource(id))))
}
