use std::fmt::Debug;



mod indices;
mod utils;
mod binary_ops;
mod expression;
mod slicing;
pub mod transformer;
pub mod errors;

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
