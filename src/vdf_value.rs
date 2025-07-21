use nu_protocol::{Span, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VdfValue {
    Table(BTreeMap<String, VdfValue>),
    Value(String),
}

impl VdfValue {
    pub fn into_value(self, span: Span) -> Value {
        match self {
            VdfValue::Table(map) => {
                let mut record = nu_protocol::Record::new();
                for (k, v) in map {
                    record.push(k, v.into_value(span));
                }
                Value::record(record, span)
            }
            VdfValue::Value(s) => Value::string(s, span),
        }
    }
}

pub fn parse(input: &str, lossy: bool) -> Result<VdfValue, String> {
    let mut chars = input.chars().peekable();
    let mut table = BTreeMap::new();

    while let Some(key) = parse_string(&mut chars, lossy)? {
        if let Some(value) = parse_value(&mut chars, lossy)? {
            table.insert(key, value);
        } else {
            return Err("Unexpected end of input: missing value".to_string());
        }
    }

    Ok(VdfValue::Table(table))
}

fn parse_string<I>(chars: &mut std::iter::Peekable<I>, lossy: bool) -> Result<Option<String>, String>
where
    I: Iterator<Item = char>,
{
    skip_whitespace(chars);
    if chars.peek() != Some(&'"') {
        return Ok(None);
    }
    chars.next(); // Consume opening quote

    let mut s = String::new();
    let mut escaped = false;
    while let Some(&c) = chars.peek() {
        match c {
            '"' if !escaped => {
                chars.next(); // Consume closing quote
                return Ok(Some(s));
            }
            '\\' if !escaped => {
                escaped = true;
                chars.next();
            }
            _ => {
                s.push(c);
                escaped = false;
                chars.next();
            }
        }
    }

    if lossy {
        Ok(Some(s))
    } else {
        Err("Unexpected end of input: unclosed string".to_string())
    }
}

fn parse_value<I>(chars: &mut std::iter::Peekable<I>, lossy: bool) -> Result<Option<VdfValue>, String>
where
    I: Iterator<Item = char>,
{
    skip_whitespace(chars);
    match chars.peek() {
        Some('{') => {
            chars.next(); // Consume opening brace
            let mut table = BTreeMap::new();
            loop {
                skip_whitespace(chars);
                if chars.peek() == Some(&'}') {
                    chars.next(); // Consume closing brace
                    break;
                }
                if let Some(key) = parse_string(chars, lossy)? {
                    if let Some(value) = parse_value(chars, lossy)? {
                        table.insert(key, value);
                    } else {
                        return Err("Unexpected end of input: missing value".to_string());
                    }
                } else {
                    break; // No more keys
                }
            }
            Ok(Some(VdfValue::Table(table)))
        }
        Some('"') => {
            parse_string(chars, lossy).map(|s| s.map(VdfValue::Value))
        }
        _ => Ok(None),
    }
}

fn skip_whitespace<I>(chars: &mut std::iter::Peekable<I>)
where
    I: Iterator<Item = char>,
{
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}
