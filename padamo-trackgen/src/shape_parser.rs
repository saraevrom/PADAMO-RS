
#[allow(dead_code)]
pub fn parse_usize_vec(s:&str)->Option<Vec<usize>>{
    let mut s = s.trim();
    s = s.trim_start_matches("[");
    s = s.trim_end_matches("]");
    let mut data:Vec<usize> = Vec::new();
    for x in s.split(","){
        let part = x.trim();
        if let Ok(v) = part.parse(){
            data.push(v);
        }
        else{
            return None;
        }
    }
    Some(data)
}


#[cfg(test)]
mod parse_test{
    use super::parse_usize_vec;

    #[test]
    fn test_ordinary(){
        assert_eq!(parse_usize_vec("[16,16]"),Some(vec![16usize,16usize]));
    }

    #[test]
    fn test_bracketless(){
        assert_eq!(parse_usize_vec("16,16"),Some(vec![16usize,16usize]));
    }

    #[test]
    fn test_weeeee(){
        assert_eq!(parse_usize_vec("[16 ,    16"),Some(vec![16usize,16usize]));
    }

    #[test]
    fn test_usual(){
        assert_eq!(parse_usize_vec("[16, 16]"),Some(vec![16usize,16usize]));
    }

    #[test]
    fn test_bad(){
        assert_eq!(parse_usize_vec("[16, 16asfb]"),None);
    }

    #[test]
    fn test_drunk(){
        assert_eq!(parse_usize_vec("imma be vector 16, 16"),None);
    }
}
