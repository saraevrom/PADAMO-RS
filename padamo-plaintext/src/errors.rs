#[derive(thiserror::Error,Debug)]
pub enum CSVError{
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Invalid length. File contains {total_length} lines. Start is set to {start_line}. Length is set to {length:?}.")]
    InvalidLength{
        total_length:usize,
        start_line:usize,
        length:Option<usize>
    },
    #[error("Invalid row slice: ({0}, {1}) from array of size {2}")]
    InvalidSlice(usize,usize,usize),

}
