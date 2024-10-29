use crate::{eval::Value, line_count::LineCountable};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ParserState {
    Array,
    Quote,
}

#[allow(clippy::unnecessary_wraps)]
/// # Errors
pub fn parse(src: &str) -> Result<Vec<Value>, String> {
    let mut chars = src.chars().line_count().peekable();
    // the stack of arrays at higher depths
    let mut parse_stack: Vec<Vec<Value>> = Vec::new();
    let mut states: Vec<ParserState> = Vec::new();
    // the array of atoms at the current depth
    let mut current_array: Vec<Value> = Vec::new();
    'main: while let Some((row, col, c)) = chars.next() {
        // println!("{parse_stack:#?}\n{current_array:#?}\n{row}:{col} = {c:?}");
        // begin a comment
        if c == '#' {
            // go to the end of the line
            for (_, _, c) in chars.by_ref() {
                if c == '\n' {
                    continue 'main;
                }
            }
        } else if c == '(' {
            // begin a new array
            parse_stack.push(current_array);
            states.push(ParserState::Array);
            current_array = Vec::new();
        } else if c == ')' {
            // end the current array
            if let Some(previous_level) = parse_stack.pop() {
                let arr = Value::Array(current_array);
                current_array = previous_level;
                states.pop();
                if states.last() == Some(&ParserState::Quote) {
                    current_array.push(Value::Array(vec![
                        Value::Identifier("quote".to_string()),
                        arr,
                    ]));

                    states.pop();
                } else {
                    current_array.push(arr);
                }
            } else {
                return Err(format!("Unmatched closing parenthesis at {row}:{col}"));
            }
        } else if c == '\'' {
            states.push(ParserState::Quote);
        } else if c.is_whitespace() {
        } else if c.is_numeric() {
            let mut int_value = c as u32 - '0' as u32;
            while let Some((row, col, c)) = chars.peek() {
                if c.is_whitespace() || *c == '(' || *c == ')' || *c == '#' || *c == '\'' {
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
            if states.last() == Some(&ParserState::Quote) {
                current_array.push(Value::Array(vec![
                    Value::Identifier("quote".to_string()),
                    (Value::Int(int_value as i32)),
                ]));
                states.pop();
            } else {
                current_array.push(Value::Int(int_value as i32));
            }
        } else {
            let mut id_buffer = String::from(c);
            while let Some((_, _, c)) = chars.peek() {
                if c.is_whitespace() || *c == '(' || *c == ')' || *c == '#' {
                    break;
                }
                id_buffer.push(*c);
                chars.next();
            }
            if states.last() == Some(&ParserState::Quote) {
                current_array.push(Value::Array(vec![
                    Value::Identifier("quote".to_string()),
                    Value::Identifier(id_buffer),
                ]));
                states.pop();
            } else {
                current_array.push(Value::Identifier(id_buffer));
            }
        }
    }
    if parse_stack.is_empty() {
        Ok(current_array)
    } else {
        Err("Unmatched opening parenthesis".to_string())
    }
}
