use std::{collections::HashMap, rc::Rc};

use crate::{line_count::LineCountable, types::Value};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ParserState {
    Array,
    Quote,
    QuasiQuote,
    Unquote,
    Table,
}

#[allow(clippy::too_many_lines)]
/// # Errors
pub fn parse(src: &str) -> Result<Value, String> {
    let mut chars = src.chars().line_count().peekable();
    // the stack of arrays at higher depths
    let mut parse_stack: Vec<Vec<Value>> = Vec::new();
    let mut states: Vec<ParserState> = Vec::new();
    // the array of atoms at the current depth
    let mut current_array: Vec<Value> = Vec::new();
    'main: loop {
        let next_thing = 'inner: {
            'by_char: while let Some((row, col, c)) = chars.next() {
                // println!("{parse_stack:#?}\n{current_array:#?}\n{row}:{col} = {c:?}");
                // begin a comment
                if c == '#' {
                    // go to the end of the line
                    for (_, _, c) in chars.by_ref() {
                        if c == '\n' {
                            continue 'by_char;
                        }
                    }
                } else if c == '{' {
                    // begin a new table
                    parse_stack.push(current_array);
                    states.push(ParserState::Table);
                    states.push(ParserState::Array);
                    current_array = Vec::new();
                } else if c == '}' {
                    // end the current table
                    if let (Some(previous_level), Some(ParserState::Array)) =
                        (parse_stack.pop(), states.pop())
                    {
                        let arr = Value::List(current_array.into());
                        current_array = previous_level;
                        break 'inner arr;
                    }
                    return Err(format!("Unmatched closing curly bracket at {row}:{col}"));
                } else if c == '(' {
                    // begin a new array
                    parse_stack.push(current_array);
                    states.push(ParserState::Array);
                    current_array = Vec::new();
                } else if c == ')' {
                    // end the current array
                    if let (Some(previous_level), Some(ParserState::Array)) =
                        (parse_stack.pop(), states.pop())
                    {
                        let arr = Value::List(current_array.into());
                        current_array = previous_level;
                        break 'inner arr;
                    }
                    return Err(format!("Unmatched closing parenthesis at {row}:{col}"));
                } else if c == '\'' {
                    states.push(ParserState::Quote);
                } else if c == '`' {
                    states.push(ParserState::QuasiQuote);
                } else if c == '~' {
                    states.push(ParserState::Unquote);
                } else if c.is_whitespace() {
                } else if c.is_numeric() {
                    let mut int_value = c as u32 - '0' as u32;
                    while let Some((row, col, c)) = chars.peek() {
                        if c.is_whitespace()
                            || *c == '('
                            || *c == ')'
                            || *c == '#'
                            || *c == '\''
                            || *c == '{'
                            || *c == '}'
                        {
                            break;
                        }
                        if !c.is_numeric() {
                            return Err(format!(
                                "Invalid character in int literal: `{c:?}`; {row}:{col}"
                            ));
                        }
                        int_value *= 10;
                        int_value += *c as u32 - '0' as u32;
                        chars.next();
                    }
                    #[allow(clippy::cast_possible_wrap)]
                    break 'inner Value::Int(int_value as i32);
                } else if c == '"' {
                    let mut string_buf = String::new();
                    while let Some((_, _, c)) = chars.next() {
                        if c == '"' {
                            break 'inner Value::String(
                                string_buf.replace("\\n", "\n").replace("\\\\", "\\"),
                            );
                        }
                        string_buf.push(c);
                        if c == '\\' {
                            let Some((_, _, c)) = chars.next() else {
                                return Err("Unexpected EOF".to_string());
                            };
                            string_buf.push(c);
                        }
                    }
                    return Err(format!("Unmatched quote; {row}:{col}"));
                } else {
                    let mut id_buffer = String::from(c);
                    while let Some((_, _, c)) = chars.peek() {
                        if c.is_whitespace()
                            || *c == '('
                            || *c == ')'
                            || *c == '#'
                            || *c == '{'
                            || *c == '}'
                        {
                            break;
                        }
                        id_buffer.push(*c);
                        chars.next();
                    }
                    break 'inner Value::symbol(&id_buffer);
                }
            }
            break 'main;
        };
        let next_thing = match states.last() {
            Some(ParserState::Quote) => {
                states.pop();
                Value::List(vec![Value::symbol("quote"), next_thing].into())
            }
            Some(ParserState::QuasiQuote) => {
                states.pop();
                Value::List(vec![Value::symbol("quasiquote"), next_thing].into())
            }
            Some(ParserState::Unquote) => {
                states.pop();
                Value::List(vec![Value::symbol("unquote"), next_thing].into())
            }
            Some(ParserState::Table) => {
                states.pop();
                let Value::List(l) = next_thing else {
                    unreachable!()
                };
                if l.len() % 2 != 0 {
                    return Err("Invalid table syntax".to_string());
                }
                let mut hm = HashMap::new();
                for i in 0..(l.len() / 2) {
                    hm.insert(l[2 * i].clone(), l[2 * i + 1].clone());
                }
                Value::Table(Rc::new(hm))
            }
            Some(ParserState::Array) | None => next_thing,
        };
        current_array.push(next_thing);
    }
    if parse_stack.is_empty() {
        current_array.insert(0, Value::symbol("do"));
        Ok(Value::List(current_array.into()))
    } else {
        Err("Unmatched opening parenthesis".to_string())
    }
}
