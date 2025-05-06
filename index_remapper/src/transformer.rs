use std::collections::HashMap;
use std::fmt::Debug;

use nom::multi::separated_list1;
use nom::{character, IResult, Parser};
use nom::combinator::{cut, fail, map_res};
use nom::sequence::delimited;
use nom::bytes::complete::tag;
use nom::branch::alt;
use padamo_api::lazy_array_operations::ArrayND;
use crate::errors::ReindexError;
use crate::expression::parse_expression;
use crate::utils::{parse_comma_sep, parse_semicolon_sep};

use crate::utils::IndexCalculator;
use padamo_api::lazy_array_operations::ndim_array::indexing::ShapeIterator;

pub struct Transformer<T:Clone+Debug+abi_stable::StableAbi>{
    pub transformers:Vec<Box<dyn IndexCalculator>>,
    pub fill_value:T,
}

impl<T:Clone+Debug+abi_stable::StableAbi> Transformer<T> {
    pub fn new(transformers: Vec<Box<dyn IndexCalculator>>, fill_value: T) -> Self {
        Self { transformers, fill_value }
    }

    pub fn transform_indices(&self, indices:&[usize], indices_len:&[usize])->Option<Vec<usize>>{
        let mut res = Vec::with_capacity(self.transformers.len());
        for t in self.transformers.iter(){
            let item = t.calculate(indices, indices_len)?;
            res.push(item);
        }
        Some(res)
    }
}

pub struct BakedTransformer<T:Clone+Debug+abi_stable::StableAbi>{
    source_shape:Vec<usize>,
    target_shape:Vec<usize>,
    mapping:HashMap<Vec<usize>, Vec<usize>>,
    fill_value:T,
}

impl<T:Clone+Debug+abi_stable::StableAbi> BakedTransformer<T>{
    pub fn new(t:Transformer<T>, source_shape:Vec<usize>)->Result<Self, crate::errors::ReindexError>{
        let mut mapping = HashMap::new();
        let mut target_shape:Vec<usize> = Vec::new();
        for index in ShapeIterator::new(source_shape.clone()){
            if let Some(target_index) = t.transform_indices(&index, &source_shape){
                if let Some(p1) = mapping.insert(target_index.clone(), index.clone()){
                    return Err(crate::errors::ReindexError::PixelRaceError(p1, index, target_index));
                }
                if target_index.is_empty(){
                    target_shape = target_index.iter().map(|x| x+1).collect();
                }
                else{
                    target_shape.iter_mut().enumerate().for_each(|(i, tgt)|{
                        *tgt = usize::max(*tgt, target_index[i]+1);
                    });
                }
            }
        }
        if target_shape.is_empty(){
            Err(crate::errors::ReindexError::TargetIsEmpty)
        }
        else{
            Ok(Self{source_shape, target_shape, mapping, fill_value:t.fill_value})
        }
    }

    pub fn apply(&self, src:&ArrayND<T>)->Result<ArrayND<T>,crate::errors::ReindexError>{
        if src.form_compatible(&self.source_shape){
            let mut res = ArrayND::new(self.target_shape.clone(), self.fill_value.clone());
            for (target_index, source_index) in self.mapping.iter(){
                if let Some(got) = src.try_get(source_index){
                    res.set(target_index, got.clone());
                }
            }
            Ok(res)
        }
        else{
            Err(ReindexError::IncompatibleShapes(src.shape.clone().into_vec(), self.source_shape.clone()))
        }
    }
}


fn parse_default_transformer<T:Clone+Debug+abi_stable::StableAbi>(default_value:T, input:&str)->IResult<&str, Transformer<T>>{
    let (input, preres) = separated_list1(parse_comma_sep, parse_expression).parse(input)?;
    let res = Transformer::new(preres, default_value);
    Ok((input, res))
}

fn parse_full_transformer<'a,T,U>(mut parser:U, input:&'a str)->IResult<&'a str, Transformer<T>,nom::error::Error<&'a str>>
where
    T:Clone+Debug+abi_stable::StableAbi,
    U:Parser<&'a str, Output=T, Error=nom::error::Error<&'a str>>,
{
    let (input, default_value) = parser.parse(input)?;
    let (input, _) = cut(parse_semicolon_sep).parse(input)?;
    parse_default_transformer(default_value, input)
}

pub fn parse_complete_transformer<'a,T,U>(default_value:T, default_parser:U, input:&'a str)->IResult<&'a str, Transformer<T>,nom::error::Error<&'a str>>
where
    T:Clone+Debug+abi_stable::StableAbi,
    U:Parser<&'a str, Output=T, Error=nom::error::Error<&'a str>>,
{
    let res1 = parse_full_transformer(default_parser, input);
    match res1{
        Ok(v)=> Ok(v),
        Err(e)=>{
            if let nom::Err::Failure(f) = e{
                Err(nom::Err::Failure(f))
            }
            else{
                parse_default_transformer(default_value, input)
            }
        }
    }
    // let mut parser_full = move |x| parse_full_transformer(default_parser, input);
    // let mut parser_simple = move |x| parse_default_transformer(default_value, input);
    // alt((parser_full, parser_simple)).parse(input)
}
