use std::error::Error;
use std::fmt;

#[derive(Debug)]
enum ParseError {
    NoMatchFound,
    InvalidNumber,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidNumber => write!(f, "Failed to parse number"),
            ParseError::NoMatchFound => write!(f, "No match found"),
        }
    }
}

impl Error for ParseError {}
// Read more here: https://kean.blog/post/regex-parser
pub trait Parse {
    type Output;

    fn parse(&self, input: &str) -> Result<Option<(Self::Output, &str)>, ParseError>;
}

struct HelloParser {}

impl Parse for HelloParser {
    type Output = &'static str;

    fn parse(&self, input: &str) -> Result<Option<(Self::Output, &str)>, ParseError> {
        if input.starts_with("hello") {
            Ok(Some(("hello", &input[5..])))
        } else {
            Ok(None)
        }
    }
}
