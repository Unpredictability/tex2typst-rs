use crate::definitions::{TexNode, TexNodeType, TexToken, TexTokenType};
use std::sync::LazyLock;

pub static EMPTY_NODE: LazyLock<TexNode> =
    LazyLock::new(|| TexNode::new(TexNodeType::Empty, String::new(), None, None));

pub static LEFT_CURLY_BRACKET: LazyLock<TexToken> =
    LazyLock::new(|| TexToken::new(TexTokenType::Control, "{".to_string()));
pub static RIGHT_CURLY_BRACKET: LazyLock<TexToken> =
    LazyLock::new(|| TexToken::new(TexTokenType::Control, "}".to_string()));

pub static LEFT_SQUARE_BRACKET: LazyLock<TexToken> =
    LazyLock::new(|| TexToken::new(TexTokenType::Element, "[".to_string()));
pub static RIGHT_SQUARE_BRACKET: LazyLock<TexToken> =
    LazyLock::new(|| TexToken::new(TexTokenType::Element, "]".to_string()));

pub fn eat_whitespaces(tokens: &[TexToken], start: usize) -> usize {
    let mut pos = start;
    while pos < tokens.len() && matches!(tokens[pos].token_type, TexTokenType::Space | TexTokenType::Newline) {
        pos += 1;
    }
    tokens[start..pos].len()
}

pub fn eat_parenthesis(tokens: &[TexToken], start: usize) -> Option<&TexToken> {
    let first_token = &tokens[start];
    if first_token.token_type == TexTokenType::Element
        && ["(", ")", "[", "]", "|", "\\{", "\\}", "."].contains(&first_token.value.as_str())
    {
        Some(first_token)
    } else if first_token.token_type == TexTokenType::Command
        && ["lfloor", "rfloor", "lceil", "rceil", "langle", "rangle"].contains(&&first_token.value[1..])
    {
        Some(first_token)
    } else {
        None
    }
}

pub fn eat_primes(tokens: &[TexToken], start: usize) -> usize {
    let mut pos = start;
    while pos < tokens.len() && tokens[pos] == TexToken::new(TexTokenType::Element, "'".to_string()) {
        pos += 1;
    }
    pos - start
}

pub fn find_closing_match(tokens: &[TexToken], start: usize, left_token: &TexToken, right_token: &TexToken) -> isize {
    assert!(tokens[start].eq(left_token));
    let mut count = 1;
    let mut pos = start + 1;

    while count > 0 {
        if pos >= tokens.len() {
            return -1;
        }
        if tokens[pos].eq(left_token) {
            count += 1;
        } else if tokens[pos].eq(right_token) {
            count -= 1;
        }
        pos += 1;
    }

    (pos - 1) as isize
}

pub static LEFT_COMMAND: LazyLock<TexToken> =
    LazyLock::new(|| TexToken::new(TexTokenType::Command, "\\left".to_string()));
pub static RIGHT_COMMAND: LazyLock<TexToken> =
    LazyLock::new(|| TexToken::new(TexTokenType::Command, "\\right".to_string()));

pub fn find_closing_right_command(tokens: &[TexToken], start: usize) -> isize {
    find_closing_match(tokens, start, &LEFT_COMMAND, &RIGHT_COMMAND)
}

pub static BEGIN_COMMAND: LazyLock<TexToken> =
    LazyLock::new(|| TexToken::new(TexTokenType::Command, "\\begin".to_string()));
pub static END_COMMAND: LazyLock<TexToken> =
    LazyLock::new(|| TexToken::new(TexTokenType::Command, "\\end".to_string()));

pub fn find_closing_end_command(tokens: &[TexToken], start: usize) -> isize {
    find_closing_match(tokens, start, &BEGIN_COMMAND, &END_COMMAND)
}

pub static SUB_SYMBOL: LazyLock<TexToken> = LazyLock::new(|| TexToken::new(TexTokenType::Control, "_".to_string()));
pub static SUP_SYMBOL: LazyLock<TexToken> = LazyLock::new(|| TexToken::new(TexTokenType::Control, "^".to_string()));
