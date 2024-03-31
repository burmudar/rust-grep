use std::env;
use std::io;
use std::process;

pub mod dfa;

enum Pattern {
    Digit(String),
    Literal(String),
    AlphaNumeric(String),
}

impl Pattern {
    fn from(pattern: &str) -> Pattern {
        match pattern {
            "\\d" => Pattern::Digit(pattern.to_string()),
            "\\w" => Pattern::AlphaNumeric(pattern.to_string()),
            _ => Pattern::Literal(pattern.to_string()),
        }
    }
    fn match_on(&self, line: &str) -> Result<bool, String> {
        match self {
            Pattern::Digit(pattern) => Ok(handle_digit(line, pattern)),
            Pattern::AlphaNumeric(pattern) => Ok(handle_alpha_numeric(line, pattern)),
            Pattern::Literal(pattern) => {
                if pattern.chars().count() == 1 {
                    Ok(line.contains(pattern))
                } else {
                    Err(format!("unknown literal pattern: {}", pattern))
                }
            }
        }
    }
}

fn handle_digit(input_line: &str, _pattern: &str) -> bool {
    input_line.chars().filter(|c| c.is_digit(10)).count() > 0
}

fn handle_alpha_numeric(input_line: &str, _pattern: &str) -> bool {
    input_line.chars().filter(|c| c.is_alphanumeric()).count() > 0
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    Pattern::from(pattern)
        .match_on(input_line)
        .expect("Pattern match failure")
}

// Usage: echo <input_text> | your_grep.sh -E <pattern>
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    // Uncomment this block to pass the first stage
    if match_pattern(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}

#[cfg(test)]
mod tests {
    use crate::Pattern;

    #[test]
    fn pattern_from_returns_correct_enum() {
        assert!(matches!(Pattern::from("\\d"), Pattern::Digit(_)));
        assert!(matches!(Pattern::from("\\w"), Pattern::AlphaNumeric(_)));
        assert!(matches!(Pattern::from("f"), Pattern::Literal(_)));
    }

    #[test]
    fn digit_character_class() {
        let p = Pattern::Digit("\\d".to_string());
        assert!(matches!(p.match_on("apple123"), Ok(true)));
        assert!(matches!(p.match_on("apple"), Ok(false)));
        assert!(matches!(p.match_on("---"), Ok(false)));
    }

    #[test]
    fn alphanumeric_character_class() {
        let p = Pattern::AlphaNumeric("\\w".to_string());
        assert!(matches!(p.match_on("apple123"), Ok(true)));
        assert!(matches!(p.match_on("apple"), Ok(true)));
        // just punctuation should fail
        assert!(matches!(p.match_on("---"), Ok(false)));
        // letters, numbers and some punctuation should pass
        assert!(matches!(p.match_on("alph4-num3ric"), Ok(true)));
    }

    #[test]
    fn literal_match_on() {
        let p = Pattern::Literal("f".to_string());
        assert!(matches!(p.match_on("f"), Ok(true)));
        assert!(matches!(p.match_on("a"), Ok(false)));
        assert!(matches!(p.match_on(""), Ok(false)));
    }
}
