use std::f64::consts::PI;

use nom::multi::separated_list1;
use nom::number::complete::{double, };
//use nom::number::streaming::double;
use nom::IResult;
use nom::error::ParseError;
use nom::bytes::complete::{escaped, tag, tag_no_case, take_while};
use nom::character::complete::{alphanumeric1 as alphanumeric, char as char_t, digit1, multispace1, one_of};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated};
use nom::combinator::{cut, map_res,};
use nom::branch::alt;
use nom::Parser;

use crate::polygon::DetectorPixel;

use super::base_parsers::{parse_index,parse_point,sp};

#[derive(Clone, Copy)]
enum RectSpec{
    TopRight(f64,f64),
    Size(f64,f64),
}

impl RectSpec{
    pub fn to_tr(self, bl:(f64,f64))->(f64,f64){
        match self {
            Self::TopRight(a, b)=>(a,b),
            Self::Size(a, b)=>(a+bl.0,b+bl.1)
        }
    }
}



pub fn sp_sep<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
  //let chars = " \t\r\n";

  multispace1(i)
  // nom combinators like `take_while` return a function. That function is the
  // parser,to which we can pass the input
  //take_while(move |c| chars.contains(c))(i)
}

fn pixel_definition<'a>(i:&'a str)-> IResult<&'a str,Vec<usize>, nom::error::Error<&'a str>>{
    //let splitted = ;
    let index = preceded(sp, parse_index);
    let mut merged = separated_pair(tag_no_case("pixel"), sp_sep, index).map(|x| x.1);
    merged.parse(i)
}

fn parse_polygon<'a>(i:&'a str)-> IResult<&'a str, Vec<(f64,f64)>, nom::error::Error<&'a str>>{
    let vertices = separated_list1(sp_sep, parse_point);
    let mut merged = separated_pair(tag_no_case("polygon"), sp_sep, vertices).map(|x| x.1);
    merged.parse(i)
}

fn parse_rect_size<'a>(i:&'a str)-> IResult<&'a str, RectSpec, nom::error::Error<&'a str>>{
    let pair_parser = separated_pair(tag_no_case("size"), sp_sep, parse_point).map(|x| x.1);
    let mut enwrapped = pair_parser.map(|x| RectSpec::Size(x.0, x.1) );
    enwrapped.parse(i)
}

fn parse_rect_tr<'a>(i:&'a str)-> IResult<&'a str, RectSpec, nom::error::Error<&'a str>>{
    let pair_parser = separated_pair(tag_no_case("top_right"), sp_sep, parse_point).map(|x| x.1);
    let mut enwrapped = pair_parser.map(|x| RectSpec::TopRight(x.0, x.1) );
    enwrapped.parse(i)
}

fn parse_rect<'a>(i:&'a str)-> IResult<&'a str, Vec<(f64,f64)>, nom::error::Error<&'a str>>{
    let spec = alt((parse_rect_size,parse_rect_tr));
    let full_rect_spec = separated_pair(parse_point, sp_sep, spec);

    let part = separated_pair(tag_no_case("rect"),sp ,cut(full_rect_spec) ).map(|x|x.1);
    let mut mapped = part.map(|x| {
        let bl = x.0;
        let spec = x.1;
        let tr = spec.to_tr(bl);
        let (x1,y1) = bl;
        let (x2,y2) = tr;
        vec![(x1,y1),(x2,y1),(x2,y2),(x1,y2)]
    });
    mapped.parse(i)
}

fn parse_square<'a>(i:&'a str)-> IResult<&'a str, Vec<(f64,f64)>, nom::error::Error<&'a str>>{

    let full_square_spec = separated_pair(parse_point, sp_sep, double);
    let partial_square_spec = double.map(|x| ((-x/2.0,-x/2.0),x));
    let alt_square_spec = alt((full_square_spec, partial_square_spec));

    let part = separated_pair(tag_no_case("square"),sp ,cut(alt_square_spec) ).map(|x|x.1);
    let mut mapped = part.map(|x| {
        let bl = x.0;
        let w = x.1;
        let (x1,y1) = bl;
        let x2 = x1+w;
        let y2 = y1+w;
        vec![(x1,y1),(x2,y1),(x2,y2),(x1,y2)]
    });
    mapped.parse(i)
}


fn parse_hexagon<'a>(i:&'a str)-> IResult<&'a str, Vec<(f64,f64)>, nom::error::Error<&'a str>>{


    let part = separated_pair(tag_no_case("hexagon"),sp ,cut(double) ).map(|x|x.1);
    let mut mapped = part.map(|r| {
        let mut res = Vec::with_capacity(6);
        for i in 0..6{
            let phi = (i as f64)/3.0*PI;
            let x = phi.cos()*r;
            let y = phi.sin()*r;
            res.push((x,y))
        }
        res
    });
    mapped.parse(i)
}


fn parse_angle<'a>(i:&'a str)-> IResult<&'a str, f64, nom::error::Error<&'a str>>{
    let degrees = separated_pair(double, sp_sep, tag_no_case("deg")).map(|x|x.0*PI/180.0);
    alt((degrees,double)).parse(i)
}

fn parse_rotate<'a>(i:&'a str)-> IResult<&'a str, Vec<(f64,f64)>, nom::error::Error<&'a str>>{
    let rotator = separated_pair(tag_no_case("rotate"),sp_sep , parse_angle).map(|x| x.1);
    separated_pair(rotator, sp_sep, parse_vertices)
        .map(|x|{
            let ang = x.0;
            let a11 = ang.cos();
            let a12 = -ang.sin();
            let a21 = ang.sin();
            let a22 = ang.cos();
            x.1.iter().map(|(x,y)| (x*a11+y*a12,x*a21+y*a22)).collect()
        }).parse(i)
}

fn parse_translate<'a>(i:&'a str)-> IResult<&'a str, Vec<(f64,f64)>, nom::error::Error<&'a str>>{
    let rotator = separated_pair(tag_no_case("move"),sp_sep , parse_point).map(|x| x.1);
    separated_pair(rotator, sp_sep, parse_vertices)
        .map(|x|{
            let pos = x.0;
            x.1.iter().map(|(x,y)| (x+pos.0,y+pos.1)).collect()
        }).parse(i)
}

fn parse_vertices<'a>(i:&'a str)-> IResult<&'a str, Vec<(f64,f64)>, nom::error::Error<&'a str>>{
    alt((parse_rotate,parse_translate,parse_polygon,parse_rect,parse_square, parse_hexagon)).parse(i)
}

pub fn parse_pixel<'a>(i:&'a str)-> IResult<&'a str, DetectorPixel, nom::error::Error<&'a str>>{
    separated_pair(pixel_definition, sp_sep, cut(parse_vertices))
    .map(|x|DetectorPixel::new(x.0, x.1))
    .parse(i)
}


#[cfg(test)]
mod test_parsers{
    use super::*;

    #[test]
    fn test_definition(){
        assert_eq!(pixel_definition("pixel [ 1, 2 ]"),Ok(("",vec!(1usize,2usize))))
    }

    #[test]
    fn test_definition_common(){
        assert_eq!(pixel_definition("pixel [1, 2]"),Ok(("",vec!(1usize,2usize))))
    }

    #[test]
    fn test_incorrect_definition(){
        assert_ne!(pixel_definition("reeee [ 1, 2 ]"),Ok(("",vec!(1usize,2usize))))
    }

    #[test]
    fn test_poly(){
        assert_eq!(parse_vertices("polygon (0.0,0.0) (0.0,1.0) (1.0,0.0)"),Ok(("",vec![(0.0,0.0),(0.0,1.0),(1.0,0.0)])))
    }

    #[test]
    fn test_rect(){
        assert_eq!(parse_vertices("rect (0.0,0.0) size (1.0,1.0)"),Ok(("",vec![(0.0,0.0),(1.0,0.0),(1.0,1.0),(0.0,1.0)])))
    }
    #[test]
    fn test_square(){
        assert_eq!(parse_vertices("square (0.0,0.0) 1.0"),Ok(("",vec![(0.0,0.0),(1.0,0.0),(1.0,1.0),(0.0,1.0)])))
    }

    #[test]
    fn test_pixel(){
        assert_eq!(parse_pixel("pixel [1,2]\n\t square (0.0,0.0) 1.0"),Ok(("",DetectorPixel::new(vec![1,2],vec![(0.0,0.0),(1.0,0.0),(1.0,1.0),(0.0,1.0)]))));
    }

    // #[test]
    // fn test_square_alt(){
    //     assert_eq!(parse_vertices("move (0.5,0.5) square 1.0"),Ok(("",vec![(0.0,0.0),(1.0,0.0),(1.0,1.0),(0.0,1.0)])))
    // }

    // #[test]
    // fn test_roll(){
    //     assert_eq!(parse_vertices("rotate 90 deg square (0.0,0.0) 1.0"),Ok(("",vec![(0.0,0.0),(0.0,1.0),(-1.0,1.0),(-1.0,0.0)])))
    // }
}

