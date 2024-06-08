use core::result::Result;
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

type ParseResult<'a, Output> = Result<(&'a str, Output), &'a str>;

trait Parser<'a, Output> {
    type Output;

    fn parse(&self, input: &'a str) -> ParseResult<'a, Output>;
}

pub fn char_parser() -> impl Fn(&str) -> ParseResult<char> {
    move |input: &str| {
        if input.is_empty() {
            return Err(input);
        }
        let c = input.chars().next().unwrap();
        Ok((&input[1..], c))
    }
}

pub fn filter<P, F, O>(parser: P, pred: F) -> impl Fn(&str) -> Result<(&str, O), &str>
where
    P: Fn(&str) -> Result<(&str, O), &str>,
    F: Fn(&O) -> bool,
{
    move |input: &str| {
        let result = (parser)(input);
        match result {
            Ok((next, c)) => {
                if pred(&c) {
                    Ok((next, c))
                } else {
                    Err(input)
                }
            }
            Err(e) => Err(e),
        }
    }
}

pub fn map<P, F, A, B>(parser: P, map_fn: F) -> impl Fn(&str) -> Result<(&str, B), &str>
where
    P: Fn(&str) -> Result<(&str, A), &str>,
    F: Fn(A) -> B,
{
    move |input| match parser(input) {
        Ok((next, r)) => Ok((next, map_fn(r))),
        Err(err) => Err(err),
    }
}

pub fn digit_parser() -> impl Fn(&str) -> ParseResult<String> {
    map(
        filter(char_parser(), |c: &char| c.is_digit(10)),
        |c: char| c.to_string(),
    )
}

pub fn one_or_more<P, O>(parser: P) -> impl Fn(&str) -> ParseResult<Vec<O>>
where
    P: Fn(&str) -> ParseResult<O>,
{
    move |input| {
        let mut s = input;
        let mut matches = vec![];

        if let Ok((next, r)) = parser(input) {
            matches.push(r);
            s = next;
        } else {
            return Err(s);
        }

        while !s.is_empty() {
            match parser(s) {
                Ok((next, r)) => {
                    matches.push(r);
                    s = next;
                }
                Err(_) => break,
            }
        }
        Ok((s, matches))
    }
}

pub fn zero_or_more<P>(parser: P) -> impl Fn(&str) -> ParseResult<Vec<&str>>
where
    P: Fn(&str) -> ParseResult<&str>,
{
    move |input| {
        let mut s = input;
        let mut matches = vec![];
        while !s.is_empty() {
            match parser(s) {
                Ok((next, r)) => {
                    matches.push(r);
                    s = next;
                }
                Err(_) => break,
            }
        }
        Ok((s, matches))
    }
}

pub fn number_parser() -> impl Fn(&str) -> ParseResult<String> {
    map(one_or_more(digit_parser()), |r: Vec<String>| {
        let s: String = r.concat();
        s
    })
}

#[cfg(test)]
mod tests {

    use crate::parser;

    #[test]
    fn char_parser() {
        let parse = parser::char_parser();
        match parse("william") {
            Ok((rest, c)) => {
                assert!(c == 'w');
                assert!(rest == "illiam");
            }
            Err(_) => panic!("unexpected error!"),
        }
    }

    #[test]
    fn digit_parser() {
        let parser = parser::digit_parser();
        match parser("1william") {
            Ok((rest, d)) => {
                assert!(d == "1");
                assert!(rest == "william")
            }
            Err(e) => panic!("unexpected error! {}", e),
        }
    }

    #[test]
    fn number_parser() {
        let parse = parser::number_parser();
        match parse("12a3william") {
            Ok((rest, num)) => {
                assert!(num == "12");
                assert!(rest == "a3william");
            }
            Err(_) => panic!("number parser unexpected error!"),
        }
    }
}
