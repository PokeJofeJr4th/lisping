use crate::line_count::LineCountable;

#[derive(Debug)]
pub enum Syntax {
    Int(i32),
    Identifier(String),
    Array(Vec<Syntax>),
}

#[allow(clippy::unnecessary_wraps)]
pub fn parse(src: &str) -> Result<Vec<Syntax>, String> {
    let mut chars = src.chars().line_count().peekable();
    // the stack of arrays at higher depths
    let mut parse_stack: Vec<Vec<Syntax>> = Vec::new();
    // the array of atoms at the current depth
    let mut current_array: Vec<Syntax> = Vec::new();
    'main: while let Some((row, col, c)) = chars.next() {
        println!("{parse_stack:#?}\n{current_array:#?}\n{row}:{col} = {c:?}");
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
            current_array = Vec::new();
        } else if c == ')' {
            // end the current array
            if let Some(previous_level) = parse_stack.pop() {
                let arr = Syntax::Array(current_array);
                current_array = previous_level;
                current_array.push(arr);
            } else {
                return Ok(current_array);
            }
        } else if c.is_whitespace() {
        } else if c.is_numeric() {
            let mut int_value = c as u32 - '0' as u32;
            for (row, col, c) in chars.by_ref() {
                if c.is_whitespace() {
                    break;
                }
                if !c.is_numeric() {
                    return Err(format!(
                        "Invalid character in int literal: `{c:?}`; {row}:{col}"
                    ));
                }
                int_value *= 10;
                int_value += c as u32 - '0' as u32;
            }
            current_array.push(Syntax::Int(int_value as i32));
        } else {
            let mut id_buffer = String::from(c);
            while let Some((_, _, c)) = chars.peek() {
                if c.is_whitespace() || *c == '(' || *c == ')' || *c == '#' {
                    break;
                }
                id_buffer.push(*c);
                chars.next();
            }
            current_array.push(Syntax::Identifier(id_buffer));
        }
    }
    Err("Uh oh!".to_string())
}
