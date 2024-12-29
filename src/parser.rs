use std::{collections::HashMap, iter::Peekable, rc::Rc};

use crate::{line_count::LineCountable, types::Value};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ParserState {
    Array,
    List,
    Quote,
    QuasiQuote,
    Unquote,
    Table,
}

/// # Errors
pub fn parse(src: &str) -> Result<Value, String> {
    let mut chars = src.chars().line_count().peekable();
    // the stack of arrays at higher depths
    let mut parse_stack: Vec<Vec<Value>> = Vec::new();
    let mut states: Vec<ParserState> = Vec::new();
    // the array of atoms at the current depth
    let mut current_array: Vec<Value> = Vec::new();
    'main: loop {
        let Some(mut next_thing) = read_value(
            &mut chars,
            &mut current_array,
            &mut states,
            &mut parse_stack,
        )?
        else {
            break 'main;
        };
        loop {
            match states.last() {
                Some(ParserState::Quote) => {
                    states.pop();
                    next_thing = Value::List(vec![Value::symbol("quote"), next_thing].into());
                }
                Some(ParserState::QuasiQuote) => {
                    states.pop();
                    next_thing = Value::List(vec![Value::symbol("quasiquote"), next_thing].into());
                }
                Some(ParserState::Unquote) => {
                    states.pop();
                    next_thing = Value::List(vec![Value::symbol("unquote"), next_thing].into());
                }
                Some(ParserState::Table) => {
                    states.pop();
                    let Value::List(l) = next_thing else {
                        return Err("Invalid table syntax".to_string());
                    };
                    if l.len() % 2 != 0 {
                        return Err("Invalid table syntax".to_string());
                    }
                    let mut hm = HashMap::new();
                    for i in 0..(l.len() / 2) {
                        hm.insert(l[2 * i].clone(), l[2 * i + 1].clone());
                    }
                    next_thing = Value::Table(Rc::new(hm));
                }
                Some(ParserState::Array | ParserState::List) | None => break,
            }
        }
        current_array.push(next_thing);
    }
    if parse_stack.is_empty() {
        current_array.insert(0, Value::symbol("do"));
        Ok(Value::List(current_array.into()))
    } else {
        Err("Unmatched opening parenthesis".to_string())
    }
}

#[allow(clippy::too_many_lines)]
fn read_value(
    chars: &mut Peekable<impl Iterator<Item = (usize, usize, char)>>,
    current_array: &mut Vec<Value>,
    states: &mut Vec<ParserState>,
    parse_stack: &mut Vec<Vec<Value>>,
) -> Result<Option<Value>, String> {
    'by_char: while let Some((row, col, c)) = chars.next() {
        // println!("{parse_stack:#?}\n{current_array:#?}\n{row}:{col} = {c:?}");
        // begin a comment
        if c == '#' {
            if chars.peek().is_some_and(|(_, _, x)| *x == '#') {
                chars.next();
                let mut doc_buf = String::new();
                chars.next_if(|(_, _, c)| *c == ' ');
                for (_, _, c) in chars.by_ref() {
                    if c == '\n' {
                        break;
                    }
                    doc_buf.push(c);
                }
                current_array.push(Value::List(Rc::new([
                    Value::Symbol("doc".to_string()),
                    Value::String(doc_buf),
                ])));
                continue 'by_char;
            }
            // go to the end of the line
            for (_, _, c) in chars.by_ref() {
                if c == '\n' {
                    continue 'by_char;
                }
            }
        } else if c == '{' {
            // begin a new table
            parse_stack.push(core::mem::take(current_array));
            states.push(ParserState::Table);
            states.push(ParserState::Array);
        } else if c == '}' {
            // end the current table
            if let (Some(previous_level), Some(ParserState::Array)) =
                (parse_stack.pop(), states.pop())
            {
                let arr = Value::List(core::mem::replace(current_array, previous_level).into());
                return Ok(Some(arr));
            }
            return Err(format!("Unmatched closing curly bracket at {row}:{col}"));
        } else if c == '(' {
            // begin a new array
            parse_stack.push(core::mem::take(current_array));
            states.push(ParserState::Array);
        } else if c == ')' {
            // end the current array
            if let (Some(previous_level), Some(ParserState::Array)) =
                (parse_stack.pop(), states.pop())
            {
                let arr = Value::List(core::mem::replace(current_array, previous_level).into());
                return Ok(Some(arr));
            }
            return Err(format!("Unmatched closing parenthesis at {row}:{col}"));
        } else if c == '[' {
            // begin a new list
            parse_stack.push(core::mem::take(current_array));
            states.push(ParserState::List);
        } else if c == ']' {
            // end the current array
            if let (Some(previous_level), Some(ParserState::List)) =
                (parse_stack.pop(), states.pop())
            {
                current_array.insert(0, Value::symbol("list"));
                let arr = Value::List(core::mem::replace(current_array, previous_level).into());
                return Ok(Some(arr));
            }
            return Err(format!("Unmatched closing square bracket at {row}:{col}"));
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
                    || *c == '['
                    || *c == ']'
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
            return Ok(Some(Value::Int(int_value as i32)));
        } else if c == '"' {
            let mut string_buf = String::new();
            while let Some((_, _, c)) = chars.next() {
                if c == '"' {
                    return Ok(Some(Value::String(
                        string_buf.replace("\\n", "\n").replace("\\\\", "\\"),
                    )));
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
                    || *c == '['
                    || *c == ']'
                    || *c == '#'
                    || *c == '{'
                    || *c == '}'
                {
                    break;
                }
                id_buffer.push(*c);
                chars.next();
            }
            return Ok(Some(Value::symbol(&id_buffer)));
        }
    }
    Ok(None)
}
