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
