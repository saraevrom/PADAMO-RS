use std::fmt::Debug;
use nom::{branch::alt, bytes::complete::{tag, tag_no_case}, IResult, Parser};

mod indices;
mod utils;
mod binary_ops;
mod expression;
mod slicing;
pub mod transformer;
pub mod errors;


pub use errors::ReindexError;
pub use transformer::BakedTransformer;

pub fn parse_remapper<'a,T,U>(input:&'a str, source_shape:Vec<usize>, default_fill_value:T, default_parser:U)->Result<transformer::BakedTransformer<T>,errors::ReindexError>
where
    T:Clone+Debug+abi_stable::StableAbi,
    U:nom::Parser<&'a str, Output=T, Error=nom::error::Error<&'a str>>,
{
    let (_, parsed) = transformer::parse_complete_transformer(default_fill_value, default_parser, &input)?;
    transformer::BakedTransformer::new(parsed, source_shape)
}

pub fn parse_f64_remapper<'a>(input:&'a str, source_shape:Vec<usize>, default_fill_value:f64)->Result<transformer::BakedTransformer<f64>,errors::ReindexError>{
    parse_remapper(&input, source_shape, default_fill_value, nom::number::complete::double)
}

fn parse_bool(input:& str)->IResult<&str, bool>{
    let true_value = alt((tag_no_case("true"),tag("1"))).map(|_| true);
    let false_value = alt((tag_no_case("false"),tag("0"))).map(|_| false);
    alt((true_value,false_value)).parse(input)
}

pub fn parse_bool_remapper<'a>(input:&'a str, source_shape:Vec<usize>, default_fill_value:bool)->Result<transformer::BakedTransformer<bool>,errors::ReindexError>{
    parse_remapper(&input, source_shape, default_fill_value, parse_bool)
}
