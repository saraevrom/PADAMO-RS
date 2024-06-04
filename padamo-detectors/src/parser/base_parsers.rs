use nom::multi::{separated_list0, separated_list1};
use nom::number::complete::{double, };
use nom::IResult;
use nom::error::{context, ParseError};
use nom::bytes::complete::{escaped, tag, take_while};
use nom::character::complete::{alphanumeric1 as alphanumeric, char as char_t, digit1, multispace0, one_of};
use nom::sequence::{delimited, preceded, separated_pair, terminated};
use nom::combinator::{cut, map_res};
use nom::Parser;




fn parse_usize<'a>(i: &'a str) -> IResult<&'a str, usize, nom::error::Error<&'a str>> {
    map_res(digit1, |s:&str| s.parse::<usize>()).parse(i)
}

pub fn sp<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
  //let chars = " \t\r\n";

  multispace0(i)
  // nom combinators like `take_while` return a function. That function is the
  // parser,to which we can pass the input
  //take_while(move |c| chars.contains(c))(i)
}

pub fn parse_point<'a>(i:&'a str)-> IResult<&'a str, (f64,f64), nom::error::Error<&'a str>>{
    let pair = separated_pair(
        preceded(sp, double),
        cut(preceded(sp, char_t(','))),
        preceded(sp, double)
    );
    let mut inbracket_pair = terminated(preceded(char_t('('),pair),preceded(sp,char_t(')')));
    inbracket_pair.parse(i)

}

pub fn parse_grid_point<'a>(i:&'a str)-> IResult<&'a str, (usize,usize), nom::error::Error<&'a str>>{
    let pair = separated_pair(
        preceded(sp, parse_usize),
        cut(preceded(sp, char_t(','))),
        preceded(sp, parse_usize)
    );
    let mut inbracket_pair = terminated(preceded(char_t('['),pair),preceded(sp,char_t(']')));
    inbracket_pair.parse(i)

}

pub fn parse_index<'a>(i:&'a str)-> IResult<&'a str, Vec<usize>, nom::error::Error<&'a str>>{
    // let pair = separated_pair(
    //     preceded(sp, parse_usize),
    //     cut(preceded(sp, char_t(','))),
    //     preceded(sp, parse_usize)
    // );
    let separator = terminated(preceded(sp, char_t(',')),sp);
    let array = separated_list1(separator, parse_usize);
    //let mut inbracket_pair = terminated(preceded(char_t('['),array),preceded(sp,char_t(']')));
    let mut inbracket_pair = context(
        "index",
        preceded(
                char_t('['),
                 cut(
                     terminated(
                         separated_list0(
                             preceded(sp, char_t(',')),
                             preceded(sp,parse_usize),
                         ),
                        preceded(sp, char_t(']'))
                      )
                     )
                 )
        );
    inbracket_pair.parse(i)

}

#[cfg(test)]
mod test_point_parser{
    use crate::parser::base_parsers::parse_point;

    #[test]
    fn test_dense_point(){
        assert_eq!(parse_point("(1.0,2.0)"),Ok(("",(1.0,2.0))))
    }

    #[test]
    fn test_normal_point(){
        assert_eq!(parse_point("(1.0, 2.0)"),Ok(("",(1.0,2.0))))
    }

    #[test]
    fn test_somespace_point(){
        assert_eq!(parse_point("( 1.0, 2.0)"),Ok(("",(1.0,2.0))))
    }

    #[test]
    fn test_sparse_point(){
        assert_eq!(parse_point("( 1.0, 2.0 )"),Ok(("",(1.0,2.0))))
    }

    #[test]
    fn test_sick_sparse_point(){
        assert_eq!(parse_point("( 1.0 , 2.0 )"),Ok(("",(1.0,2.0))))
    }

    #[test]
    fn test_sickest_sparse_point(){
        assert_eq!(parse_point("(  1.0  ,  2.0 ) "),Ok((" ",(1.0,2.0))))
    }


    #[test]
    fn test_integers(){
        assert_eq!(parse_point("(1, 2) "),Ok((" ",(1.0,2.0))))
    }
}


#[cfg(test)]
mod test_index_parser{
    use crate::parser::base_parsers::parse_index;

    #[test]
    fn test_dense_point(){
        assert_eq!(parse_index("[1,2]"),Ok(("",vec!(1usize,2usize))))
    }

    #[test]
    fn test_normal_point(){
        assert_eq!(parse_index("[1, 2]"),Ok(("",vec!(1usize,2usize))))
    }

    #[test]
    fn test_somespace_point(){
        assert_eq!(parse_index("[ 1, 2]"),Ok(("",vec!(1usize,2usize))))
    }

    #[test]
    fn test_sparse_point(){
        assert_eq!(parse_index("[ 1, 2 ]"),Ok(("",vec!(1usize,2usize))))
    }

    #[test]
    fn test_sick_sparse_point(){
        assert_eq!(parse_index("[ 1 , 2 ]"),Ok(("",vec!(1usize,2usize))))
    }

    #[test]
    fn test_sickest_sparse_point(){
        assert_eq!(parse_index("[  1  ,  2 ] "),Ok((" ",vec!(1usize,2usize))))
    }

    #[test]
    fn test_triple(){
        assert_eq!(parse_index("[1, 2, 3] "),Ok((" ",vec!(1usize,2usize,3usize))))
    }

}
