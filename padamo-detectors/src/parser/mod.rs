use nom::multi::{separated_list0, separated_list1};
use nom::number::complete::{double, };
use nom::IResult;
use nom::error::{context, ParseError};
use nom::bytes::complete::{escaped, tag, tag_no_case, take_while};
use nom::character::complete::{alphanumeric1 as alphanumeric, char as char_t, digit1, multispace0, one_of};
use nom::sequence::{delimited, preceded, separated_pair, terminated};
use nom::combinator::{cut, map_res};
use nom::Parser;
use nom::branch::alt;

use self::shape_constructors::sp_sep;
use base_parsers::sp;
use crate::polygon::{DetectorPixel,DetectorContent};

pub mod base_parsers;
pub mod shape_constructors;
pub mod detector_building_data;

enum DetectorDataMod{
    PixelData(Box<dyn detector_building_data::PixelMaker>),
    Name(String),
    CompatShape(Vec<usize>)
}


fn parse_str<'a>(i: &'a str) -> IResult<&'a str, &'a str, nom::error::Error<&'a str>> {
  escaped(alphanumeric, '\\', one_of("\"n\\"))(i)
}

fn parse_string<'a>(i:&'a str)-> IResult<&'a str, &'a str, nom::error::Error<&'a str>>{
    preceded(char_t('\"'), cut(terminated(parse_str, char_t('\"')))).parse(i)
}

fn parse_pixel<'a>(i:&'a str)-> IResult<&'a str, DetectorDataMod, nom::error::Error<&'a str>>{
    shape_constructors::parse_pixel.map(|x| DetectorDataMod::PixelData(Box::new(x))).parse(i)
}


fn parse_name<'a>(i:&'a str)-> IResult<&'a str, DetectorDataMod, nom::error::Error<&'a str>>{
    let name_parser = separated_pair(tag_no_case("name"), sp_sep, parse_string).map(|x|x.1);
    name_parser.map(|x| DetectorDataMod::Name(x.into())).parse(i)
}

fn parse_compat_shape<'a>(i:&'a str)-> IResult<&'a str, DetectorDataMod, nom::error::Error<&'a str>>{
    let shape_parser = separated_pair(tag_no_case("shape"), sp_sep, base_parsers::parse_index).map(|x|x.1);
    shape_parser.map(|x| DetectorDataMod::CompatShape(x)).parse(i)
}

fn parse_instruction<'a>(i:&'a str)-> IResult<&'a str, DetectorDataMod, nom::error::Error<&'a str>>{
    alt((parse_pixel,parse_name,parse_compat_shape)).parse(i)
}

pub fn parse_detector<'a>(i:&'a str)-> IResult<&'a str, DetectorContent, nom::error::Error<&'a str>>{
    let splitted = separated_list0(sp_sep, parse_instruction);
    let pre = preceded(sp, splitted);
    let mut detector_parser = pre.map(|x|{
        let mut name = "Unknown_detector".to_string();
        let mut compat_shape:Vec<usize> = vec![0];
        let mut pixels:Vec<DetectorPixel> = Vec::new();
        let mut x = x;
        for item in x.drain(..){
            match item {
                //DetectorDataMod::Pixel(p)=>{pixels.push(p)},
                DetectorDataMod::PixelData(data)=>{
                    pixels.extend(data.get_pixels());
                }
                DetectorDataMod::Name(s)=>{name = s},
                DetectorDataMod::CompatShape(s)=>{compat_shape = s},
            }
        }
        DetectorContent{compat_shape,name,content:pixels}
    });
    detector_parser.parse(i)
}

/*

pub fn parse_detector<'a>(i:&'a)-> IResult<&'a str, usize, nom::error::Error<&'a str>>{
    preceded(base_parsers::sp, separated_list0(shape_constructors::sp_sep, shape_constructors::parse_pixel).map(|x|).parse(i)
}
*/


#[cfg(test)]
mod parser_tests{
    use super::*;
    #[test]
    fn test_vtl(){
        //Let's generate Tuloma source and parse it back
        let vtl = DetectorContent::default_vtl();
        let src = vtl.into_src(None);
        let detector = parse_detector(&src).unwrap().1;
        assert_eq!(vtl,detector)
    }
}
