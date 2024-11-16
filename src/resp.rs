use pest::iterators::Pair;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "resp.pest"]
pub struct RESPParser;

#[derive(Debug, PartialEq)]
pub enum Entry {
    Int(i32),
    Text(String),
    SimpleText(String),
    Nil,
}

impl ToString for Entry {
    fn to_string(&self) -> String {
        match self {
            Entry::Text(text) => format!("${}\r\n{}\r\n", text.len(), text),
            Entry::SimpleText(text) => format!("+{}\r\n", text),
            Entry::Int(text) => format!(":{}\r\n", text),
            Entry::Nil => format!("$-1\r\n"),
        }
    }
}

pub struct Array(pub Vec<Entry>);

impl ToString for Array {
    fn to_string(&self) -> String {
        let mut result = format!("*{}\r\n", self.0.len());
        for entry in self.0.iter() {
            result.push_str(&entry.to_string());
        }
        result
    }
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

pub fn extract_array_entries(pair: Pair<Rule>) -> Vec<Entry> {
    pair.into_inner()
        .filter_map(|p| match p.as_rule() {
            Rule::int => Some(Entry::Int(extract_int_value(p))),
            Rule::string => Some(Entry::Text(extract_string_value(p).to_string())),
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
        let expected = vec![Entry::Int(2), Entry::Text("three".to_string())];
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
            Entry::Int(2),
            Entry::Text("three".to_string()),
            Entry::Array(vec![Entry::Int(4)]),
        ];
        let actual = RESPParser::parse(Rule::array, input)
            .expect("failed to parse input")
            .next()
            .expect("expected a single pair");

        let entries = extract_array_entries(actual);

        assert_eq!(entries, expected);
    }
}
