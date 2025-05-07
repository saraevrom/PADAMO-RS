use std::fmt::Debug;

use nom::{character, IResult, Parser};
use nom::combinator::{map_res, fail};
use nom::sequence::delimited;
use nom::bytes::complete::tag;
use nom::branch::alt;

pub trait IndexCalculator: Debug{
    fn calculate(&self, index_src:&[usize], index_sizes:&[usize])->Option<usize>;
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
