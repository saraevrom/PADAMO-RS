use std::fmt::Debug;

use nom::multi::separated_list1;
use nom::{character, IResult, Parser};
use nom::combinator::cut;
use nom::bytes::complete::tag;

pub trait IndexCalculator: Debug{
    fn calculate(&self, index_src:&[usize], index_sizes:&[usize])->Option<i64>;
}

pub fn parse_comma_sep(input:&str)->IResult<&str, ()>{
    let (input, _) = tag(",").parse(input)?;
    let (input, _) = character::complete::space0.parse(input)?;
    Ok((input, ()))
}

pub fn parse_semicolon_sep(input:&str)->IResult<&str, ()>{
    let (input, _) = tag(";").parse(input)?;
    let (input, _) = character::complete::space0.parse(input)?;
    Ok((input, ()))
}

pub fn parse_colon_sep(input:&str)->IResult<&str, ()>{
    let (input, _) = tag(":").parse(input)?;
    let (input, _) = character::complete::space0.parse(input)?;
    Ok((input, ()))
}

pub fn parse_shape_array(input:&str)->IResult<&str, Vec<usize>>{
    let array = separated_list1(parse_comma_sep, character::complete::usize);

    let mut wrapped = (tag("("), cut(array), cut(tag(")"))).map(|x| x.1);
    wrapped.parse(input)
}
