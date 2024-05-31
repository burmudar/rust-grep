use std::error::Error;
use std::fmt;
use std::str::FromStr;

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

    fn parse<'a>(&self, input: &'a str) -> Result<Option<(Self::Output, &'a str)>, ParseError>;
}

struct HelloParser {}

impl Parse for HelloParser {
    type Output = &'static str;

    fn parse<'a>(&self, input: &'a str) -> Result<Option<(Self::Output, &'a str)>, ParseError> {
        // if we have the prefix, rest will be the string with the prefix stripped, otherwise None
        if let Some(rest) = input.strip_prefix("hello") {
            Ok(Some(("hello", rest)))
        } else {
            Ok(None)
        }
    }
}

struct CharParser {}
impl Parse for CharParser {
    type Output = &'static str;

    fn parse<'a>(&self, input: &'a str) -> Result<Option<(Self::Output, &'a str)>, ParseError> {
        // find the first non-numberic chat position
        let pos = input
            .find(|c: char| !c.is_alphabetic())
            .unwrap_or(input.len());
        let s = &input[0..pos];
        let rest = &input[pos..];
        if pos < 0 {
            Ok(None)
        } else {
            Ok(Some((s, rest)))
        }
    }
}

struct NumberParser {}
impl Parse for NumberParser {
    type Output = i32;

    fn parse<'a>(&self, input: &'a str) -> Result<Option<(Self::Output, &'a str)>, ParseError> {
        // find the first non-numberic chat position
        let pos = input.find(|c: char| !c.is_numeric()).unwrap_or(input.len());
        let num_str = &input[0..pos];
        let rest = &input[pos..];
        if let Ok(num) = num_str.parse::<i32>() {
            Ok(Some((num, rest)))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::NumberParser;
    use crate::parser::Parse;
    #[test]
    fn number_parser() {
        let parser = NumberParser {};

        match parser.parse("123") {
            Ok(Some((num, rest))) => {
                assert_eq!(123, num);
                assert_eq!("", rest)
            }
            Ok(None) => panic!("got none but wanted '123' number"),
            Err(e) => {
                panic!("got error: {}", e)
            }
        }
        match parser.parse("777 abc") {
            Ok(Some((num, rest))) => {
                assert_eq!(777, num);
                assert_eq!(" abc", rest)
            }
            Ok(None) => panic!("got none but wanted '123' number"),
            Err(e) => {
                panic!("got error: {}", e)
            }
        }
        match parser.parse("def 777 abc") {
            Ok(Some(_)) => {
                panic!("parsing should fail because the str starts with non numberic chars")
            }
            Ok(None) => assert!(true, "should not parse anything"),
            Err(e) => {
                panic!("got error: {}", e)
            }
        }
    }
}
