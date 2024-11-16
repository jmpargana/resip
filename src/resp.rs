use pest::iterators::Pair;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "resp.pest"]
pub struct RESPParser;

#[derive(Debug, PartialEq)]
pub enum ArrayEntry {
    Int(i32),
    Text(String),
    // Array(Vec<ArrayEntry>),
}

pub fn extract_string_value(pair: Pair<Rule>) -> &str {
    pair.into_inner()
        .find(|p| p.as_rule() == Rule::text)
        .expect("Expected at least one string")
        .as_str()
}

// Helper function to extract the integer from a `Pair` for `int`
pub fn extract_int_value(pair: Pair<Rule>) -> i32 {
    pair.into_inner()
        .next()
        .expect("Expected number after ':'")
        .as_str()
        .parse::<i32>()
        .expect("failed to parse number")
}

pub fn extract_array_entries(pair: Pair<Rule>) -> Vec<ArrayEntry> {
    pair.into_inner()
        .filter_map(|p| match p.as_rule() {
            Rule::int => Some(ArrayEntry::Int(extract_int_value(p))),
            Rule::string => Some(ArrayEntry::Text(extract_string_value(p).to_string())),
            // Rule::array => Some(ArrayEntry::Array(extract_array_entries(p))),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_int() {
        let input = ":123\r\n";
        let expected = 123;
        let actual = RESPParser::parse(Rule::int, input)
            .expect("failed to parse input")
            .next()
            .expect("expected a single pair");

        assert_eq!(extract_int_value(actual), expected);
    }

    #[test]
    fn can_parse_string() {
        let input = "$3\r\nhey\r\n";
        let expected = "hey";
        let actual = RESPParser::parse(Rule::string, input)
            .expect("failed to parse input")
            .next()
            .expect("expected a single pair");

        assert_eq!(extract_string_value(actual), expected);
    }

    #[test]
    fn can_parse_array() {
        let input = "*2\r\n:2\r\n$5\r\nthree\r\n";
        let expected = vec![ArrayEntry::Int(2), ArrayEntry::Text("three".to_string())];
        let actual = RESPParser::parse(Rule::array, input)
            .expect("failed to parse input")
            .next()
            .expect("expected a single pair");

        let entries = extract_array_entries(actual);

        assert_eq!(entries, expected);
    }

    #[test]
    fn can_parse_array_recursively() {
        let input = "*2\r\n:2\r\n$5\r\nthree\r\n*1\r\n:4\r\n";
        let expected = vec![
            ArrayEntry::Int(2),
            ArrayEntry::Text("three".to_string()),
            ArrayEntry::Array(vec![ArrayEntry::Int(4)]),
        ];
        let actual = RESPParser::parse(Rule::array, input)
            .expect("failed to parse input")
            .next()
            .expect("expected a single pair");

        let entries = extract_array_entries(actual);

        assert_eq!(entries, expected);
    }
}
