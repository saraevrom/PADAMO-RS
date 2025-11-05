use std::collections::HashMap;
use std::fmt::Debug;

use nom::multi::separated_list1;
use nom::{IResult, Parser};
use padamo_api::lazy_array_operations::ArrayND;
use crate::errors::ReindexError;
use crate::expression::parse_expression;
use crate::utils::{parse_comma_sep, parse_semicolon_sep, parse_shape_array};

use crate::utils::IndexCalculator;
use padamo_api::lazy_array_operations::ndim_array::ShapeIterator;


fn index_is_inside(index:&[usize], shape:&[usize])->Result<bool, ReindexError>{
    if index.len()!=shape.len(){
        return Err(ReindexError::IncompatibleIndexWithShape(index.len(), shape.len()))
    }
    for (i, s) in index.iter().zip(shape.iter()){
        if i>=s{
            return Ok(false);
        }
    }

    Ok(true)
}


pub struct Transformer<T:Clone+Debug+abi_stable::StableAbi>{
    pub transformers:Vec<Box<dyn IndexCalculator>>,
    pub fill_value:T,
    pub target_shape_override:Option<Vec<usize>>,
}

impl<T:Clone+Debug+abi_stable::StableAbi> Transformer<T> {
    pub fn new(transformers: Vec<Box<dyn IndexCalculator>>, fill_value: T, target_shape_override:Option<Vec<usize>>) -> Self {
        Self { transformers, fill_value, target_shape_override }
    }

    pub fn transform_indices(&self, indices:&[usize], indices_len:&[usize])->Option<Vec<usize>>{
        let mut res = Vec::with_capacity(self.transformers.len());
        for t in self.transformers.iter(){
            let item:i64 = t.calculate(indices, indices_len)?;
            let item:usize = item.try_into().ok()?;
            res.push(item);
        }
        Some(res)
    }
}

#[derive(Clone, Debug)]
pub struct BakedTransformer<T:Clone+Debug+abi_stable::StableAbi>{
    pub source_shape:Vec<usize>,
    pub target_shape:Vec<usize>,
    pub mapping:HashMap<Vec<usize>, Vec<usize>>,
    pub fill_value:T,
}

impl<T:Clone+Debug+abi_stable::StableAbi> BakedTransformer<T>{
    pub fn new(t:Transformer<T>, source_shape:Vec<usize>)->Result<Self, crate::errors::ReindexError>{
        let mut mapping = HashMap::new();
        let mut target_shape:Vec<usize> = Vec::new();
        let estimating_target_shape = if let Some(tso) = &t.target_shape_override{
            target_shape = tso.clone();
            false
        }
        else{
            true
        };

        'maincycle: for index in ShapeIterator::new(source_shape.clone()){
            if let Some(target_index) = t.transform_indices(&index, &source_shape){
                // println!("{:?}->{:?}", index, target_index);
                if !estimating_target_shape{
                    if !(index_is_inside(&target_index, &target_shape)?){
                        continue 'maincycle;
                    }
                }

                if let Some(p1) = mapping.insert(target_index.clone(), index.clone()){
                    return Err(crate::errors::ReindexError::PixelRaceError(p1, index, target_index));
                }

                if estimating_target_shape{
                    if target_shape.is_empty(){
                        target_shape = target_index.iter().map(|x| x+1).collect();
                    }
                    else{
                        target_shape.iter_mut().enumerate().for_each(|(i, tgt)|{
                            *tgt = usize::max(*tgt, target_index[i]+1);
                        });
                    }
                }
            }
            // else{
            //     println!("{:?}->OOB", index);
            // }
        }
        if target_shape.is_empty(){
            Err(crate::errors::ReindexError::TargetIsEmpty)
        }
        else{
            // if let Some(ts) = target_shape_override{
            //     target_shape = ts;
            // }
            // println!("Target shape: {:?}", target_shape);
            Ok(Self{source_shape, target_shape, mapping, fill_value:t.fill_value})
        }
    }

    pub fn apply(&self, src:&ArrayND<T>)->Result<ArrayND<T>,crate::errors::ReindexError>{
        if src.form_compatible(&self.source_shape){
            let mut res = ArrayND::new(self.target_shape.clone(), self.fill_value.clone());
            // println!("{:?}", self.mapping);

            for (target_index, source_index) in &self.mapping{
                // println!("Applying {:?} -> {:?}", source_index, target_index);
                if let Some(got) = src.try_get(source_index){
                    res.set(target_index, got.clone());
                }
            }
            // println!("Apply success");
            Ok(res)
        }
        else{
            Err(ReindexError::IncompatibleShapes(src.shape.clone().into_vec(), self.source_shape.clone()))
        }
    }
}


fn parse_default_transformer<T:Clone+Debug+abi_stable::StableAbi>(default_value:T, target_shape_override:Option<Vec<usize>>, input:&str)->IResult<&str, Transformer<T>>{
    let (input, preres) = separated_list1(parse_comma_sep, parse_expression).parse(input)?;
    let res = Transformer::new(preres, default_value, target_shape_override);
    Ok((input, res))
}

// fn parse_full_transformer<'a,T,U>(mut parser:U, input:&'a str)->IResult<&'a str, Transformer<T>,nom::error::Error<&'a str>>
// where
//     T:Clone+Debug+abi_stable::StableAbi,
//     U:Parser<&'a str, Output=T, Error=nom::error::Error<&'a str>>,
// {
//     let (input, default_value) = parser.parse(input)?;
//     let (input, _) = cut(parse_semicolon_sep).parse(input)?;
//     parse_default_transformer(default_value, input)
// }

pub fn parse_complete_transformer<'a,T,U>(default_value:T, default_parser:U, input:&'a str)->IResult<&'a str, Transformer<T>,nom::error::Error<&'a str>>
where
    T:Clone+Debug+abi_stable::StableAbi,
    U:Parser<&'a str, Output=T, Error=nom::error::Error<&'a str>>,
{
    let mut input = input;
    let mut actual_default_value = default_value;

    if let Ok((i, v)) = (default_parser, parse_semicolon_sep).map(|x| x.0).parse(input){
        input = i;
        actual_default_value = v;
    }

    let target_override;
    if let Ok((i, v)) = (parse_shape_array, parse_semicolon_sep).map(|x| x.0).parse(input){
        input = i;
        target_override = Some(v);
    }
    else{
        target_override = None;
    }

    parse_default_transformer(actual_default_value, target_override, input)

}
