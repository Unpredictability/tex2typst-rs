use crate::definitions::{TexToken, TexTokenType};

fn eat_command_name(latex: &Vec<char>, start: usize) -> String {
    let mut pos = start;
    while pos < latex.len() && latex[pos].is_alphabetic() {
        pos += 1;
    }
    latex[start..pos].iter().collect::<String>()
}

fn find_closing_curly_bracket_char(latex: &Vec<char>, start: usize) -> Result<usize, &'static str> {
    assert_eq!(latex[start], '{');
    let mut count = 1;
    let mut pos = start + 1;

    while count > 0 {
        if pos >= latex.len() {
            return Err("Unmatched curly brackets");
        }
        if pos + 1 < latex.len() && ["\\{", "\\}"].contains(&latex[pos..pos + 2].iter().collect::<String>().as_str()) {
            pos += 2;
            continue;
        }
        match latex[pos] {
            '{' => count += 1,
            '}' => count -= 1,
            _ => {}
        }
        pos += 1;
    }

    Ok(pos - 1)
}

pub fn tokenize(latex: &str) -> Result<Vec<TexToken>, String> {
    let latex: Vec<char> = latex.chars().collect();
    let mut tokens: Vec<TexToken> = Vec::new();
    let mut pos = 0;

    while pos < latex.len() {
        let first_char = latex[pos];
        let token: TexToken;
        match first_char {
            '%' => {
                let mut new_pos = pos + 1;
                while new_pos < latex.len() && latex[new_pos] != '\n' {
                    new_pos += 1;
                }
                token = TexToken::new(TexTokenType::Comment, latex[pos + 1..new_pos].iter().collect());
                pos = new_pos;
            }
            '{' | '}' | '_' | '^' | '&' => {
                token = TexToken::new(TexTokenType::Control, first_char.to_string());
                pos += 1;
            }
            '\n' => {
                token = TexToken::new(TexTokenType::Newline, first_char.to_string());
                pos += 1;
            }
            '\r' => {
                if pos + 1 < latex.len() && latex[pos + 1] == '\n' {
                    token = TexToken::new(TexTokenType::Newline, "\n".to_string());
                    pos += 2;
                } else {
                    token = TexToken::new(TexTokenType::Newline, "\n".to_string());
                    pos += 1;
                }
            }
            ' ' => {
                let mut new_pos = pos;
                while new_pos < latex.len() && latex[new_pos] == ' ' {
                    new_pos += 1;
                }
                token = TexToken::new(TexTokenType::Space, latex[pos..new_pos].iter().collect());
                pos = new_pos;
            }
            '\\' => {
                if pos + 1 >= latex.len() {
                    return Err("Expecting command name after '\\'".to_string());
                }
                let first_two_chars = latex[pos..pos + 2].iter().collect::<String>();
                if ["\\\\", "\\,"].contains(&&*first_two_chars) {
                    token = TexToken::new(TexTokenType::Control, first_two_chars.to_string());
                } else if ["\\{", "\\}", "\\%", "\\$", "\\&", "\\#", "\\_", "\\|"].contains(&&*first_two_chars) {
                    token = TexToken::new(TexTokenType::Element, first_two_chars.to_string());
                } else {
                    let command = eat_command_name(&latex, pos + 1);
                    token = TexToken::new(TexTokenType::Command, format!("\\{}", command));
                }
                pos += token.value.len();
            }
            _ => {
                if first_char.is_digit(10) {
                    let mut new_pos = pos;
                    while new_pos < latex.len() && latex[new_pos].is_digit(10) {
                        new_pos += 1;
                    }
                    token = TexToken::new(TexTokenType::Element, latex[pos..new_pos].iter().collect());
                } else if first_char.is_alphabetic() {
                    token = TexToken::new(TexTokenType::Element, first_char.to_string());
                } else if "+-*/='<>!.,;:?()[]|".contains(first_char) {
                    token = TexToken::new(TexTokenType::Element, first_char.to_string());
                } else if "~".contains(first_char) {
                    token = TexToken::new(TexTokenType::NoBreakSpace, "space.nobreak".to_string());
                } else {
                    token = TexToken::new(TexTokenType::Unknown, first_char.to_string());
                }
                pos += token.value.len();
            }
        }

        tokens.push(token.clone());

        if token.token_type == TexTokenType::Command
            && matches!(token.value.as_str(), r"\text" | r"\operatorname" | r"\begin" | r"\end")
        {
            if pos >= latex.len() || latex[pos] != '{' {
                if let Some(nn) = latex[pos..].iter().position(|&c| c == '{') {
                    pos += nn;
                } else {
                    return Err(format!("No content for {} command", token.value));
                }
            }
            tokens.push(TexToken::new(TexTokenType::Control, "{".to_string()));
            let pos_closing_bracket = find_closing_curly_bracket_char(&latex, pos)?;
            pos += 1;
            let mut text_inside: String = latex[pos..pos_closing_bracket].iter().collect();
            let chars = ['{', '}', '\\', '$', '&', '#', '_', '%'];
            for &char in &chars {
                text_inside = text_inside.replace(&format!("\\{}", char), &char.to_string());
            }
            tokens.push(TexToken::new(TexTokenType::Text, text_inside));
            tokens.push(TexToken::new(TexTokenType::Control, "}".to_string()));
            pos = pos_closing_bracket + 1;
        }
    }
    Ok(tokens)
}