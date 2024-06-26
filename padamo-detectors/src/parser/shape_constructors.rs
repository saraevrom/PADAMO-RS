use std::f64::consts::PI;

use nom::multi::separated_list1;
use nom::number::complete::{double, };
//use nom::number::streaming::double;
use nom::IResult;
use nom::error::{ErrorKind, ParseError};
use nom::bytes::complete::{escaped, tag, tag_no_case, take_while};
use nom::character::complete::{alphanumeric1 as alphanumeric, char as char_t, digit1, multispace1, one_of};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated};
use nom::combinator::{cut, map_res,};
use nom::branch::alt;
use nom::Parser;

use crate::polygon::DetectorPixel;

use super::base_parsers::{parse_index,parse_point,sp, parse_grid_point};
use super::detector_building_data::{PixelGrid, PixelGridError, PolygonArray, SinglePixel, Transformable, TransformablePixelMaker, IndexExtend,IndexExtension};

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

fn parse_polygon<'a>(i:&'a str)-> IResult<&'a str, PolygonArray, nom::error::Error<&'a str>>{
    let vertices = separated_list1(sp_sep, parse_point);
    let mut merged = separated_pair(tag_no_case("polygon"), sp_sep, vertices).map(|x| x.1.into());
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

fn parse_rect<'a>(i:&'a str)-> IResult<&'a str, PolygonArray, nom::error::Error<&'a str>>{
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
    }).map(|x| x.into());
    mapped.parse(i)
}

fn parse_square<'a>(i:&'a str)-> IResult<&'a str, PolygonArray, nom::error::Error<&'a str>>{

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
    }).map(|x| x.into());
    mapped.parse(i)
}


fn parse_hexagon<'a>(i:&'a str)-> IResult<&'a str, PolygonArray, nom::error::Error<&'a str>>{


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
    }).map(|x| x.into());
    mapped.parse(i)
}


fn parse_angle<'a>(i:&'a str)-> IResult<&'a str, f64, nom::error::Error<&'a str>>{
    let degrees = separated_pair(double, sp_sep, tag_no_case("deg")).map(|x|x.0*PI/180.0);
    alt((degrees,double)).parse(i)
}

fn parse_rotate<'a,T,U>(i:&'a str, f:U)-> IResult<&'a str, T, nom::error::Error<&'a str>>
where
    T:Transformable,
    U:Fn(&'a str) -> IResult<&'a str, T, nom::error::Error<&'a str>>,
{
    let rotator = separated_pair(tag_no_case("rotate"),sp_sep , parse_angle).map(|x| x.1);
    separated_pair(rotator, sp_sep, f)
        .map(|x|{
            let ang = x.0;
            let mut res = x.1;
            res.rotate(ang);
            res
        }).parse(i)
}

fn parse_rotate_polygon<'a>(i:&'a str)-> IResult<&'a str, PolygonArray, nom::error::Error<&'a str>>

{
    parse_rotate(i, parse_vertices)
}

fn parse_translate<'a,T,U>(i:&'a str, f:U)-> IResult<&'a str, T, nom::error::Error<&'a str>>
where
    T:Transformable,
    U:Fn(&'a str) -> IResult<&'a str, T, nom::error::Error<&'a str>>,
{
    let mover = separated_pair(tag_no_case("move"),sp_sep , parse_point).map(|x| x.1);
    separated_pair(mover, sp_sep, f)
        .map(|x|{
            let offset = x.0;
            let mut res = x.1;
            res.offset(offset);
            res
        }).parse(i)
}

fn parse_translate_polygon<'a>(i:&'a str)-> IResult<&'a str, PolygonArray, nom::error::Error<&'a str>>{
    parse_translate(i, parse_vertices)
}

fn parse_vertices<'a>(i:&'a str)-> IResult<&'a str, PolygonArray, nom::error::Error<&'a str>>{
    alt((parse_rotate_polygon,parse_translate_polygon,parse_polygon,parse_rect,parse_square, parse_hexagon)).parse(i)
}

pub struct BoxedPixelMaker(pub Box<dyn TransformablePixelMaker>);
impl BoxedPixelMaker{
    pub fn new<T:TransformablePixelMaker+'static>(x:T)->Self{
        Self(Box::new(x))
    }
}

impl Transformable for BoxedPixelMaker {
    fn offset(&mut self,offset:(f64,f64)) {
        self.0.offset(offset)
    }
    fn rotate(&mut self,angle:f64) {
        self.0.rotate(angle)
    }
}

pub fn parse_pixel<'a>(i:&'a str)-> IResult<&'a str, BoxedPixelMaker, nom::error::Error<&'a str>>{
    separated_pair(pixel_definition, sp_sep, cut(parse_vertices))
    .map(|x| BoxedPixelMaker::new(SinglePixel{index:x.0, polygon:x.1}))
    .parse(i)
}

pub fn parse_rotate_pixelable<'a>(i:&'a str)-> IResult<&'a str, BoxedPixelMaker, nom::error::Error<&'a str>>{
    parse_rotate(i, parse_pixelable_pre)
}

pub fn parse_offset_pixelable<'a>(i:&'a str)-> IResult<&'a str, BoxedPixelMaker, nom::error::Error<&'a str>>{
    parse_translate(i, parse_pixelable_pre)
}

pub fn parse_pixelable_pre<'a>(i:&'a str)-> IResult<&'a str, BoxedPixelMaker, nom::error::Error<&'a str>>{
    alt((parse_pixel,parse_grid,parse_rotate_pixelable,parse_offset_pixelable,parse_prepend,parse_append, parse_grid_nd)).parse(i)
}

pub fn parse_pixelable<'a>(i:&'a str)-> IResult<&'a str, Box<dyn TransformablePixelMaker>, nom::error::Error<&'a str>>{
    parse_pixelable_pre.map(|x| x.0).parse(i)
}

pub fn parse_grid<'a>(i:&'a str)-> IResult<&'a str, BoxedPixelMaker, nom::error::Error<&'a str>>{
    let parser = separated_pair(preceded(sp, tag_no_case("grid")), sp_sep, parse_grid_point).map(|x| x.1); //lowers
    let parser = separated_pair(parser, sp_sep, parse_grid_point); //uppers
    let parser = separated_pair(parser, sp_sep, parse_grid_point); //steps
    let parser = separated_pair(parser, sp_sep, parse_point); //AX
    let parser = separated_pair(parser, sp_sep, parse_point); //AY
    let parser = separated_pair(parser, sp_sep, parse_pixelable); //AY
    let mut parser = parser.map(|x|{
        let (((((lowers,uppers),steps),ax),ay),obj) = x;
        PixelGrid::new(lowers, uppers, steps, (0,1), ax, ay, obj)
    });

    let res:(&str, Result<PixelGrid,PixelGridError>) = parser.parse(i)?;
    match res.1{
        Ok(v)=> IResult::Ok((res.0,BoxedPixelMaker::new(v))),
        Err(_)=>{
            IResult::Err(nom::Err::Failure(nom::error::Error { input: i, code: ErrorKind::Fail }))
        },
    }
}

pub fn parse_grid_nd<'a>(i:&'a str)-> IResult<&'a str, BoxedPixelMaker, nom::error::Error<&'a str>>{
    let parser = separated_pair(preceded(sp, tag_no_case("ndgrid")), sp_sep, parse_grid_point).map(|x| x.1); //lowers
    let parser = separated_pair(parser, sp_sep, parse_grid_point); //uppers
    let parser = separated_pair(parser, sp_sep, parse_grid_point); //steps
    let parser = separated_pair(parser, sp_sep, parse_grid_point); //mutated indices
    let parser = separated_pair(parser, sp_sep, parse_point); //AX
    let parser = separated_pair(parser, sp_sep, parse_point); //AY
    let parser = separated_pair(parser, sp_sep, parse_pixelable); //AY
    let mut parser = parser.map(|x|{
        let ((((((lowers,uppers),steps),mutated_indices),ax),ay),obj) = x;
        PixelGrid::new(lowers, uppers, steps, mutated_indices, ax, ay, obj)
    });

    let res:(&str, Result<PixelGrid,PixelGridError>) = parser.parse(i)?;
    match res.1{
        Ok(v)=> IResult::Ok((res.0,BoxedPixelMaker::new(v))),
        Err(_)=>{
            IResult::Err(nom::Err::Failure(nom::error::Error { input: i, code: ErrorKind::Fail }))
        },
    }
}


pub fn parse_prepend<'a>(i:&'a str)-> IResult<&'a str, BoxedPixelMaker, nom::error::Error<&'a str>>{
    let parser = separated_pair(preceded(sp,tag("prepend") ),sp_sep , parse_index).map(|x| x.1);
    let parser = separated_pair(parser, sp_sep, parse_pixelable);
    let mut parser = parser.map(|(index,parseable)|{
        BoxedPixelMaker::new(IndexExtend::prepend(index, parseable))
    });
    parser.parse(i)
}


pub fn parse_append<'a>(i:&'a str)-> IResult<&'a str, BoxedPixelMaker, nom::error::Error<&'a str>>{
    let parser = separated_pair(preceded(sp,tag("append") ),sp_sep , parse_index).map(|x| x.1);
    let parser = separated_pair(parser, sp_sep, parse_pixelable);
    let mut parser = parser.map(|(index,parseable)|{
        BoxedPixelMaker::new(IndexExtend::append(index, parseable))
    });
    parser.parse(i)
}

//Tests are outdated
